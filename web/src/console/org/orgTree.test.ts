import { describe, expect, it } from "vitest";

import { orgStrings as text } from "../../i18n/org";
import type { BranchSummary, HrOrgChartResponse, OrgEntitySummary, RegionSummary } from "./orgApi";
import { applyPendingOps, buildOrgColumns, deriveUnitHead, totalActive } from "./orgTree";

const chart: HrOrgChartResponse = {
  companies: [
    {
      company: "코스",
      total: 12,
      active: 10,
      units: [
        {
          name: "운영팀",
          total: 6,
          positions: [
            { title: "팀장", total: 1, employees: [{ id: "e1", name: "김하나", status: "재직" }] },
            { title: "사원", total: 5, employees: [{ id: "e2", name: "이두리", status: "재직" }] },
          ],
        },
        { name: "지원팀", total: 4, positions: [{ title: "사원", total: 4, employees: [] }] },
      ],
    },
    { company: "무소속상사", total: 3, active: 3, units: [] },
  ],
};

const entities: OrgEntitySummary[] = [
  { orgId: "org-1", slug: "coss", name: "코스", status: "ACTIVE" },
  { orgId: "org-2", slug: "knl", name: "케이앤엘", status: "ACTIVE" },
];

const regions: RegionSummary[] = [
  { id: "r1", name: "경남", deactivated_at: null, created_at: "2026-01-01T00:00:00Z" },
];

const branches: BranchSummary[] = [
  { id: "b1", region_id: "r1", name: "창원지점", deactivated_at: null, created_at: "2026-01-01T00:00:00Z" },
];

describe("buildOrgColumns", () => {
  it("maps entities to chart companies by name, keeps unmapped columns, and attaches sites to the viewer company", () => {
    const columns = buildOrgColumns({ chart, entities, regions, branches, viewerCompany: "코스" });
    expect(columns.map((column) => column.company)).toEqual(["코스", "케이앤엘", "무소속상사"]);
    expect(columns[0].entity?.slug).toBe("coss");
    expect(columns[0].sites).toHaveLength(1);
    expect(columns[0].sites[0].regionName).toBe("경남");
    expect(columns[1].units).toHaveLength(0);
    expect(columns[2].entity).toBeUndefined();
    expect(totalActive(columns)).toBe(13);
  });

  it("keeps identity sites in an explicit unmapped column when the viewer company matches nothing", () => {
    const columns = buildOrgColumns({ chart, entities, regions, branches, viewerCompany: null });
    const unmapped = columns.find((column) => column.company === text.unmappedColumn);
    expect(unmapped?.sites).toHaveLength(1);
  });
});

describe("deriveUnitHead", () => {
  it("derives the head from a leader-titled position without hardcoded names", () => {
    const head = deriveUnitHead(chart.companies[0].units[0]);
    expect(head).toEqual({ id: "e1", name: "김하나", title: "팀장" });
  });

  it("returns undefined when no leader-titled position has an employee", () => {
    expect(deriveUnitHead(chart.companies[0].units[1])).toBeUndefined();
  });
});

describe("applyPendingOps", () => {
  const base = buildOrgColumns({ chart, entities, regions, branches, viewerCompany: "코스" });

  it("projects branch rename, deactivate, and create onto the site owner column", () => {
    const next = applyPendingOps(base, [
      { op: "RENAME_BRANCH", branchId: "b1", name: "창원제1지점" },
      { op: "DEACTIVATE_BRANCH", branchId: "b1" },
      { op: "CREATE_BRANCH", regionId: "r1", name: "부산지점" },
    ], "코스");
    const sites = next[0].sites;
    expect(sites[0].branch.name).toBe("창원제1지점");
    expect(sites[0].pendingOff).toBe(true);
    expect(sites[1].branch.name).toBe("부산지점");
    expect(sites[1].pendingNew).toBe(true);
  });

  it("projects an org-unit reassign as rename, merging into an existing target unit", () => {
    const renamed = applyPendingOps(base, [
      { op: "REASSIGN_ORG_UNIT", fromOrgUnit: "지원팀", toOrgUnit: "경영지원팀", scope: { company: "코스" } },
    ], "코스");
    expect(renamed[0].units.map((unit) => unit.name)).toEqual(["운영팀", "경영지원팀"]);

    const merged = applyPendingOps(base, [
      { op: "REASSIGN_ORG_UNIT", fromOrgUnit: "지원팀", toOrgUnit: "운영팀", scope: { company: "코스" } },
    ], "코스");
    expect(merged[0].units).toHaveLength(1);
    expect(merged[0].units[0].total).toBe(10);
    expect(merged[0].units[0].positions).toEqual([
      { title: "팀장", total: 1, employees: [{ id: "e1", name: "김하나", status: "재직" }] },
      { title: "사원", total: 9, employees: [{ id: "e2", name: "이두리", status: "재직" }] },
    ]);
  });

  it("does not touch other companies and returns the same reference for empty ops", () => {
    expect(applyPendingOps(base, [], "코스")).toBe(base);
    const next = applyPendingOps(base, [
      { op: "REASSIGN_ORG_UNIT", fromOrgUnit: "운영팀", toOrgUnit: "다른팀", scope: { company: "케이앤엘" } },
    ], "코스");
    expect(next[0].units.map((unit) => unit.name)).toEqual(["운영팀", "지원팀"]);
  });
});
