import { execFileSync } from "node:child_process";
import { randomUUID } from "node:crypto";

import { expect, test, type Page } from "@playwright/test";

/**
 * EVALUATION-32 — real dev-auth Evaluation lifecycle proof.
 *
 * The browser drives the shipped generated-client UI against the real
 * dev-auth backend and PostgreSQL. SQL is restricted to prerequisites which
 * the Evaluation UI deliberately cannot create: two real employee records and
 * their local-dev persona identity links. Every Evaluation transition is made
 * through the browser. There are no route interceptions, mocked API replies,
 * or in-process fake success values.
 *
 * This requires BOTH local-only gates:
 *
 *   MNT_DEV_AUTH_E2E=1 MNT_CONSOLE_PREVIEW_E2E=1 \
 *   VITE_CONSOLE_DEV_PREVIEW=1 \
 *     node scripts/dev-up.mjs bootstrap
 *   MNT_DEV_AUTH_E2E=1 MNT_CONSOLE_PREVIEW_E2E=1 \
 *   VITE_CONSOLE_DEV_PREVIEW=1 npx playwright test \
 *     --project=dev-auth-console-preview-known-red \
 *     e2e/specs/evaluation-32-live-user-story.spec.ts
 *
 * `VITE_CONSOLE_DEV_PREVIEW` makes the dark Evaluation inventory visible only
 * in a Vite development server. It cannot expose the route in a production
 * build; `EXPOSED_SCREEN_KEYS` remains the production authority.
 */

const ORG_ID = "00000000-0000-0000-0000-0000000000a1";
const DATABASE_URL =
  process.env.MNT_DEV_DATABASE_URL ??
  "postgres://mnt_rt:mnt-dev-runtime-change-me@127.0.0.1:55432/mnt_dev";
const OWNER_DATABASE_URL =
  process.env.MNT_DEV_DATABASE_OWNER_URL ??
  "postgres://mnt_app:mnt-dev-owner-change-me@127.0.0.1:55432/mnt_dev";

type Scenario = {
  readonly runId: string;
  readonly cycleName: string;
  readonly subjectEmployeeId: string;
  readonly subjectEmployeeName: string;
  readonly isolatedOrgId: string;
  readonly isolatedOrgSlug: string;
};

type DevAuthActors = {
  readonly adminUserId: string;
  readonly executiveUserId: string;
};

let executiveOriginalEmployeeId: string | null | undefined;
let executiveUserIdForRestore: string | undefined;

function newScenario(): Scenario {
  const runId = randomUUID();
  return {
    runId,
    cycleName: `E2E 평가 ${runId.slice(0, 8)}`,
    subjectEmployeeId: randomUUID(),
    subjectEmployeeName: `E2E 평가 대상 ${runId.slice(0, 8)}`,
    isolatedOrgId: randomUUID(),
    isolatedOrgSlug: `e2e-evaluation-${runId.replaceAll("-", "").slice(0, 16)}`,
  };
}

function assertDevOnlyEnvironment(): void {
  if (
    process.env.MNT_DEV_AUTH_E2E !== "1" ||
    process.env.MNT_CONSOLE_PREVIEW_E2E !== "1" ||
    process.env.VITE_CONSOLE_DEV_PREVIEW !== "1"
  ) {
    throw new Error(
      "EVALUATION-32 requires the explicit dev-auth + console-preview selector.",
    );
  }
}

function sqlLiteral(value: string): string {
  return `'${value.replaceAll("'", "''")}'`;
}

function execSql(sql: string): string {
  return execFileSync(
    "psql",
    [DATABASE_URL, "-q", "-v", "ON_ERROR_STOP=1", "-At", "-c", sql],
    { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
  ).trim();
}

function execOwnerSql(sql: string): string {
  return execFileSync(
    "psql",
    [OWNER_DATABASE_URL, "-q", "-v", "ON_ERROR_STOP=1", "-At", "-c", sql],
    { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
  ).trim();
}

async function loginAs(
  page: Page,
  roleLabel: string,
  orgId = ORG_ID,
): Promise<void> {
  await page.goto("/login");
  // A role switch must replace—not accidentally reuse—the prior persona. This
  // is the same browser-cookie logout path a developer uses, never a forged
  // token or storage mutation.
  await page.evaluate(async () => {
    await fetch("/api/v1/auth/logout", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Auth-Transport": "cookie",
      },
      credentials: "include",
      body: "{}",
    });
  });
  await page.goto("/login");
  const switcher = page.getByRole("region", { name: "로컬 역할 전환" });
  await switcher
    .getByRole("combobox", { name: "역할" })
    .selectOption({ label: roleLabel });
  if (orgId !== ORG_ID) {
    await switcher.getByRole("button", { name: "고급 설정" }).click();
    await switcher.getByLabel("조직 ID").fill(orgId);
    await switcher.getByLabel("지점 ID (쉼표로 구분)").fill("");
  }
  await switcher
    .getByRole("button", { name: new RegExp(`${roleLabel} 로그인$`) })
    .click();
  await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 });
}

function provisionPrerequisites(scenario: Scenario): DevAuthActors {
  assertDevOnlyEnvironment();
  const adminPhone = `dev-auth:${ORG_ID}:ADMIN`;
  const executivePhone = `dev-auth:${ORG_ID}:EXECUTIVE`;
  const users = execSql(`
    BEGIN;
    SET LOCAL app.current_org = ${sqlLiteral(ORG_ID)};
    SELECT phone || '|' || id
    FROM users
    WHERE org_id = ${sqlLiteral(ORG_ID)}::uuid
      AND phone IN (${sqlLiteral(adminPhone)}, ${sqlLiteral(executivePhone)})
    ORDER BY phone;
    COMMIT;
  `).split("\n");
  const ids = new Map(
    users.filter(Boolean).map((row) => row.split("|") as [string, string]),
  );
  const adminUserId = ids.get(adminPhone) ?? "";
  const executiveUserId = ids.get(executivePhone) ?? "";
  if (!adminUserId || !executiveUserId) {
    throw new Error(
      "dev-auth personas were not provisioned through the login UI",
    );
  }

  const originalEmployeeId = execSql(`
    BEGIN;
    SET LOCAL app.current_org = ${sqlLiteral(ORG_ID)};
    SELECT employee_id::text
    FROM users
    WHERE id = ${sqlLiteral(executiveUserId)}::uuid;
    COMMIT;
  `);
  executiveOriginalEmployeeId = originalEmployeeId || null;
  executiveUserIdForRestore = executiveUserId;

  // These are immutable, run-scoped business prerequisites. The actual cycle,
  // enrollment, goals, reviews, calibration, finalization, and readback below
  // are all persisted via browser-driven product requests.
  execSql(`
    BEGIN;
    SET LOCAL app.current_org = ${sqlLiteral(ORG_ID)};
    INSERT INTO employees (
      id, org_id, company, name, source_filename, source_sheet, source_row,
      source_key, hire_date, employment_status, org_unit, identity_review_required
    ) VALUES (
      ${sqlLiteral(scenario.subjectEmployeeId)}::uuid, ${sqlLiteral(ORG_ID)}::uuid,
      'E2E', ${sqlLiteral(scenario.subjectEmployeeName)}, 'evaluation-live-e2e.xlsx',
      'evaluation', 1, ${sqlLiteral(`evaluation-live-e2e-${scenario.runId}`)},
      '2024-01-02', 'ACTIVE', 'E2E 평가', false
    );
    UPDATE users
    SET employee_id = ${sqlLiteral(scenario.subjectEmployeeId)}::uuid
    WHERE id = ${sqlLiteral(executiveUserId)}::uuid;
    COMMIT;
  `);
  return { adminUserId, executiveUserId };
}

function restoreSharedPersonaLink(): void {
  if (!executiveUserIdForRestore || executiveOriginalEmployeeId === undefined)
    return;
  const employeeId = executiveOriginalEmployeeId
    ? `${sqlLiteral(executiveOriginalEmployeeId)}::uuid`
    : "NULL";
  execSql(`
    BEGIN;
    SET LOCAL app.current_org = ${sqlLiteral(ORG_ID)};
    UPDATE users
    SET employee_id = ${employeeId}
    WHERE id = ${sqlLiteral(executiveUserIdForRestore)}::uuid;
    COMMIT;
  `);
  executiveOriginalEmployeeId = undefined;
  executiveUserIdForRestore = undefined;
}

function provisionIsolatedTenant(scenario: Scenario): void {
  execOwnerSql(`
    INSERT INTO organizations (id, slug, name, status)
    VALUES (
      ${sqlLiteral(scenario.isolatedOrgId)}::uuid,
      ${sqlLiteral(scenario.isolatedOrgSlug)},
      ${sqlLiteral(`E2E 평가 격리 ${scenario.runId.slice(0, 8)}`)},
      'ACTIVE'
    );
  `);
}

async function openEvaluation(page: Page): Promise<void> {
  await page.goto("/console/evaluation");
  await expect(
    page.getByRole("heading", { name: "평가", level: 1 }),
  ).toBeVisible({
    timeout: 15_000,
  });
}

async function createCycleAndSubject(
  page: Page,
  scenario: Scenario,
  adminUserId: string,
  refreshPreflight = false,
): Promise<void> {
  await openEvaluation(page);
  await page.getByLabel("사이클 이름").fill(scenario.cycleName);
  await page.getByLabel("기간").fill("2026-E2E");
  await page.getByLabel("마감일").fill("2026-12-31");
  await page.getByRole("button", { name: "생성", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: scenario.cycleName, level: 2 }),
  ).toBeVisible();

  await page.getByLabel("직원 검색").fill(scenario.subjectEmployeeName);
  const result = page.getByRole("list", { name: "직원 검색 결과" });
  await expect(
    result.getByRole("button", {
      name: new RegExp(scenario.subjectEmployeeName),
    }),
  ).toBeVisible({
    timeout: 15_000,
  });
  await result
    .getByRole("button", { name: new RegExp(scenario.subjectEmployeeName) })
    .click();
  await page.getByLabel("평가자").selectOption(adminUserId);
  await page.getByRole("button", { name: "추가", exact: true }).click();
  const subjectRow = page
    .getByRole("list", { name: "대상자" })
    .getByRole("button", { name: new RegExp(scenario.subjectEmployeeName) });
  await expect(subjectRow).toBeVisible();

  await subjectRow.click();
  await page.getByRole("button", { name: "목표 추가" }).click();
  await page.getByLabel("목표명").fill("현장 품질");
  await page.getByLabel("목표치").fill("불량 0건");
  await page.getByLabel("가중치").fill("100");
  const goalsSaved = page.waitForResponse(
    (response) =>
      response.request().method() === "PUT" &&
      /\/api\/v1\/evaluation\/subjects\/[^/]+\/goals$/.test(
        new URL(response.url()).pathname,
      ) &&
      response.status() === 200,
  );
  await page.getByRole("button", { name: "목표 저장" }).click();
  await goalsSaved;
  await page.getByRole("button", { name: "뒤로" }).click();

  // The positive story deliberately leaves this false: a save must refresh the
  // already visible transition gate. This refresh is setup-only for the
  // independent SELF-identity RED below, so that test reaches its own seam.
  if (refreshPreflight) {
    await openEvaluation(page);
    await page
      .getByRole("button", { name: new RegExp(scenario.cycleName) })
      .click();
  }

  const openCycle = page.getByRole("button", { name: "개시", exact: true });
  await expect(openCycle).toBeEnabled();
  await openCycle.click();
  await expect(page.getByText("진행", { exact: true })).toBeVisible();
}

async function bootstrapOpenCycle(
  page: Page,
  scenario: Scenario,
): Promise<void> {
  await loginAs(page, "관리자");
  await loginAs(page, "임원");
  await loginAs(page, "최고 관리자");
  const { adminUserId } = provisionPrerequisites(scenario);
  await createCycleAndSubject(page, scenario, adminUserId, true);
}

async function submitReview(
  page: Page,
  scenario: Scenario,
  kind: "자기평가" | "관리자 평가",
): Promise<void> {
  const task = page
    .getByRole("list", { name: "내 평가 할 일" })
    .getByRole("listitem")
    .filter({ hasText: kind });
  await expect(task).toBeVisible({ timeout: 15_000 });
  await task.getByRole("button", { name: "작성" }).click();
  const dialog = page.getByRole("dialog", { name: "평가 스코어카드" });
  await expect(dialog).toBeVisible();
  if (kind === "관리자 평가") {
    await dialog
      .getByLabel("개체 코드")
      .fill(`E2E-EVIDENCE-${scenario.runId.slice(0, 8)}`);
    await dialog.getByLabel("설명").fill("현장 검증 기록");
    await dialog.getByRole("button", { name: "증빙 추가" }).click();
  }
  await dialog
    .getByRole("group", { name: "등급" })
    .getByRole("button", { name: "A" })
    .click();
  await dialog
    .getByLabel("평가 의견 (선택)")
    .fill(`${kind} ${scenario.runId.slice(0, 8)}`);
  await dialog.getByRole("button", { name: "제출", exact: true }).click();
  await expect(dialog).toHaveCount(0);
}

test.describe("EVALUATION-32 real dev-auth story", () => {
  test.beforeEach(() => {
    assertDevOnlyEnvironment();
  });

  test.afterEach(() => {
    restoreSharedPersonaLink();
  });

  test("administrator UI persists an enrolled cycle through manager review, calibration, finalization, and ledger readback", async ({
    page,
  }) => {
    const scenario = newScenario();
    // Provision all actor personas through the same local login surface first.
    await loginAs(page, "관리자");
    await loginAs(page, "임원");
    await loginAs(page, "최고 관리자");
    const { adminUserId } = provisionPrerequisites(scenario);

    await createCycleAndSubject(page, scenario, adminUserId);

    // Current REST contract assigns both review kinds to the designated manager.
    // The next test locks the missing employee-owned SELF behavior as a RED
    // product requirement rather than pretending this delegation is parity.
    await loginAs(page, "관리자");
    await openEvaluation(page);
    await submitReview(page, scenario, "자기평가");
    await submitReview(page, scenario, "관리자 평가");

    await loginAs(page, "최고 관리자");
    await openEvaluation(page);
    await page
      .getByRole("button", { name: new RegExp(scenario.cycleName) })
      .click();
    await page.getByRole("button", { name: "조정 시작", exact: true }).click();
    await page
      .getByRole("button", { name: new RegExp(scenario.subjectEmployeeName) })
      .click();
    await page
      .getByRole("group", { name: "등급" })
      .getByRole("button", { name: "A" })
      .click();
    await page.getByLabel("조정 사유").fill("실제 현장 성과 검토");
    await page.getByRole("button", { name: "조정 확정" }).click();
    await expect(page.getByText("실제 현장 성과 검토")).toBeVisible();
    await page.getByRole("button", { name: "뒤로" }).click();
    await page.getByRole("button", { name: "확정", exact: true }).click();
    await expect(page.getByText("확정", { exact: true })).toBeVisible();

    await page
      .getByRole("button", { name: scenario.subjectEmployeeName, exact: true })
      .last()
      .click();
    await expect(
      page.getByRole("heading", {
        name: scenario.subjectEmployeeName,
        level: 2,
      }),
    ).toBeVisible();
    await expect(
      page
        .getByRole("list", { name: "평가 이력" })
        .getByText(scenario.cycleName),
    ).toBeVisible({
      timeout: 15_000,
    });
  });

  test("RED: linked employee sees and submits their own SELF review before manager review", async ({
    page,
  }) => {
    // This is the intended employee workflow. It currently fails because
    // `my_tasks` joins only `manager_user_id`, and require_review_access
    // authorizes only the manager or an EvaluationManage principal; employee_id
    // is neither consulted nor exposed by the UI as a SELF action.
    const scenario = newScenario();
    await bootstrapOpenCycle(page, scenario);
    await loginAs(page, "임원");
    await openEvaluation(page);
    await expect(
      page
        .getByRole("list", { name: "내 평가 할 일" })
        .getByRole("listitem")
        .filter({ hasText: "자기평가" }),
    ).toBeVisible({ timeout: 15_000 });
  });

  test("member role is denied and an isolated tenant cannot read the source tenant cycle", async ({
    page,
  }) => {
    const scenario = newScenario();
    // A real source-tenant cycle makes the later empty isolated-tenant list an
    // actual RLS/API isolation proof rather than merely a blank-database check.
    await loginAs(page, "최고 관리자");
    await openEvaluation(page);
    await page.getByLabel("사이클 이름").fill(scenario.cycleName);
    await page.getByLabel("기간").fill("2026-E2E");
    await page.getByLabel("마감일").fill("2026-12-31");
    await page.getByRole("button", { name: "생성", exact: true }).click();
    await expect(
      page.getByRole("heading", { name: scenario.cycleName, level: 2 }),
    ).toBeVisible();
    provisionIsolatedTenant(scenario);

    await loginAs(page, "일반 멤버");
    // The console shell denies this role by omission before the dark module
    // mounts. This is a stronger wrong-role proof than a client-side error
    // message: direct navigation is returned to the authorized overview.
    await page.goto("/console/evaluation");
    await expect(page).toHaveURL(/\/overview$/, { timeout: 15_000 });

    await loginAs(page, "최고 관리자", scenario.isolatedOrgId);
    await openEvaluation(page);
    await expect(
      page.getByText(scenario.cycleName, { exact: true }),
    ).toHaveCount(0);
    await expect(
      page.getByText("평가 사이클이 없습니다. 새 사이클을 생성하세요."),
    ).toBeVisible();
  });
});
