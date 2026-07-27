import { afterEach, describe, expect, it } from "vitest";

import {
  FALLBACK_CODE_PREFIXES,
  objectCodePrefixes,
  resetCodePrefixes,
} from "../ontology/codeGrammar";
import {
  primeObjectTypeRegistry,
  resetObjectTypeRegistry,
  type RegistryObjectType,
} from "../ontology/typeRegistrySource";
import { linkTargetFromCode, slugLabel, slugTone } from "../objectcard/kinds";

import { parseTokenGrammar, serializeTokenSpans } from "./grammar";
import { KIND_META, TONE, kindFromCode, type ObjectKind } from "./objectKinds";

/**
 * L-F3 equivalence bar, authored BEFORE the convergence refactor and green
 * against the pre-refactor code. Three code-grammar sources (composer
 * `BARE_CODE_RE`, composer `KIND_META`, objectcard `SLUG_META` +
 * `COMPOSER_KIND_TO_SLUG`) each hold their own prefix knowledge today. They are
 * about to derive it from the single dynamic source (`ontology/codeGrammar` and
 * the object-type registry that primes it) — every assertion below pins the
 * behaviour that must survive that, prefix by prefix.
 *
 * The later describes are the deliberate, named deltas: "fixed" (codes the old
 * digit-only/uppercase-only regex could never match), "narrowed" (an
 * unregistered prefix is now inert at PARSE, where before it parsed and was
 * gated inert only at render) and "one edit" (the zero-frontend-edit proof).
 */

const codeOf = (prefix: string) => `${prefix}-1234`;
const spans = (text: string) =>
  parseTokenGrammar(text)
    .filter((s) => s.kind !== "text")
    .map((s) => ({ kind: s.kind, raw: (s as { raw: string }).raw }));

/** Prefixes the OLD `[A-Z]{1,8}` regex could match: uppercase-only. */
const UPPERCASE_FALLBACK = FALLBACK_CODE_PREFIXES.filter((p) => p === p.toUpperCase());

/** Every prefix KIND_META claims, bare (no trailing dash). */
const KIND_META_PREFIXES = Object.values(KIND_META)
  .map((meta) => meta.codePrefix)
  .filter((prefix): prefix is string => Boolean(prefix))
  .map((prefix) => prefix.replace(/-+$/, ""));

/**
 * Frozen pre-refactor truth table: `kindFromCode` / `linkTargetFromCode` for a
 * canonical code under EVERY fallback prefix, with the registry unprimed (the
 * offline floor). Written by reading the pre-refactor maps; any divergence here
 * is a semantics regression, not a refactor.
 */
const OFFLINE_TRUTH: Record<string, { kind: ObjectKind | undefined; slug: string | undefined }> = {
  AP: { kind: "approval", slug: "approval_run" },
  WO: { kind: "workOrder", slug: "work_order" },
  AT: { kind: "attendance", slug: undefined },
  CS: { kind: "support", slug: "support_ticket" },
  JL: { kind: "journal", slug: undefined },
  PS: { kind: "payroll", slug: undefined },
  IN: { kind: "intake", slug: undefined },
  DX: { kind: undefined, slug: undefined },
  Bid: { kind: undefined, slug: undefined },
  MT: { kind: undefined, slug: undefined },
  EV: { kind: undefined, slug: undefined },
  OT: { kind: undefined, slug: undefined },
  SR: { kind: undefined, slug: undefined },
  PAY: { kind: undefined, slug: undefined },
  EQ: { kind: undefined, slug: undefined },
  VC: { kind: undefined, slug: undefined },
  FL: { kind: undefined, slug: undefined },
  HR: { kind: undefined, slug: undefined },
  TK: { kind: undefined, slug: undefined },
  C: { kind: "contract", slug: undefined },
  R: { kind: undefined, slug: undefined },
};

/** The nine backend slugs the object card labels, with their frozen ko label + tone. */
const SLUG_TRUTH: Record<string, { label: string; tone: Parameters<typeof TONE>[0] }> = {
  work_order: { label: "정비/배차", tone: "info" },
  equipment: { label: "장비", tone: "neutral" },
  account: { label: "계정", tone: "purple" },
  support_ticket: { label: "고객회신", tone: "warn" },
  org_unit: { label: "조직", tone: "neutral" },
  person: { label: "사람", tone: "purple" },
  approval_run: { label: "결재", tone: "accent" },
  passkey: { label: "패스키", tone: "ok" },
  consent: { label: "동의", tone: "ok" },
};

afterEach(() => {
  resetObjectTypeRegistry();
  resetCodePrefixes();
});

describe("preserved — every prefix the old grammar recognized still parses identically", () => {
  it.each(UPPERCASE_FALLBACK)("%s-1234 parses as one codeLink after a boundary", (prefix) => {
    expect(spans(`참조 ${codeOf(prefix)} 확인`)).toEqual([
      { kind: "codeLink", raw: codeOf(prefix) },
    ]);
  });

  it.each(UPPERCASE_FALLBACK)("%s-20260612-001 (two-segment) parses whole", (prefix) => {
    expect(spans(`정비 ${prefix}-20260612-001 참고`)).toEqual([
      { kind: "codeLink", raw: `${prefix}-20260612-001` },
    ]);
  });

  it.each(UPPERCASE_FALLBACK)("%s-1234 parses at line start and inside brackets", (prefix) => {
    expect(spans(codeOf(prefix))).toEqual([{ kind: "codeLink", raw: codeOf(prefix) }]);
    expect(spans(`(${codeOf(prefix)})`)).toEqual([{ kind: "codeLink", raw: codeOf(prefix) }]);
  });

  it.each([...FALLBACK_CODE_PREFIXES])("%s-1234 resolves the same kind + link slug offline", (prefix) => {
    const code = codeOf(prefix);
    const expected = OFFLINE_TRUTH[prefix];
    expect(expected, `no frozen truth row for ${prefix}`).toBeDefined();
    expect(kindFromCode(code)).toBe(expected.kind);
    expect(linkTargetFromCode(code)).toEqual(
      expected.slug ? { kind: expected.slug, id: code } : undefined,
    );
  });

  it.each(KIND_META_PREFIXES)("%s (a KIND_META prefix) is known to the grammar", (prefix) => {
    expect(objectCodePrefixes()).toContain(prefix);
  });

  it.each(Object.entries(SLUG_TRUTH))("slugLabel/slugTone are unchanged for %s", (slug, truth) => {
    expect(slugLabel(slug)).toBe(truth.label);
    expect(slugTone(slug)).toEqual(TONE(truth.tone));
  });

  it("falls back to the raw slug for an unregistered, unprimed kind", () => {
    expect(slugLabel("no_such_kind")).toBe("no_such_kind");
    expect(slugTone("no_such_kind")).toEqual(TONE("neutral"));
  });

  it("keeps plain text inert — phone numbers, lowercase codes, emails, punctuation", () => {
    expect(spans("연락 010-1234 로")).toEqual([]);
    expect(spans("ap-3121 아님")).toEqual([]);
    expect(spans("user@example.com")).toEqual([]);
    expect(spans("주의!! 확인")).toEqual([]);
  });

  it("round-trips mixed token text verbatim", () => {
    const text = "example@x.com #23 주의!! 그리고 @kim WO-2643";
    expect(serializeTokenSpans(parseTokenGrammar(text))).toBe(text);
  });

  it("keeps mention/channel/code disjoint on one line", () => {
    expect(spans("@kim #정비팀 WO-2643")).toEqual([
      { kind: "mention", raw: "@kim" },
      { kind: "channel", raw: "#정비팀" },
      { kind: "codeLink", raw: "WO-2643" },
    ]);
  });

  it("never treats a #-prefixed code as an object code (channel wins, as before)", () => {
    expect(spans("#WO-2643")).toEqual([{ kind: "channel", raw: "#WO-2643" }]);
  });
});


describe("fixed — codes the old digit-only/uppercase-only regex could never match", () => {
  it("linkifies a mixed-case fallback prefix (Bid-)", () => {
    expect(spans("입찰 Bid-1204 검토")).toEqual([{ kind: "codeLink", raw: "Bid-1204" }]);
  });

  it("linkifies an alphanumeric code body (OT-FINANCE, PAY-CHO)", () => {
    expect(spans("계정 OT-FINANCE 확인")).toEqual([{ kind: "codeLink", raw: "OT-FINANCE" }]);
    expect(spans("급여 PAY-CHO 확인")).toEqual([{ kind: "codeLink", raw: "PAY-CHO" }]);
  });

  it("linkifies a three-segment code whole instead of truncating it", () => {
    expect(spans("계획 WO-2026-Q1-07 참고")).toEqual([{ kind: "codeLink", raw: "WO-2026-Q1-07" }]);
  });
});

describe("narrowed — an unregistered prefix is now inert at parse, not just at render", () => {
  it("does not parse COVID-19 as a code (no registered COVID prefix)", () => {
    expect(spans("COVID-19 대응")).toEqual([]);
  });

  it("still round-trips the text verbatim", () => {
    expect(serializeTokenSpans(parseTokenGrammar("COVID-19 대응"))).toBe("COVID-19 대응");
  });
});

describe("one edit — a newly registered type costs the frontend zero map edits", () => {
  const DEAL: RegistryObjectType = {
    kind: "deal",
    codePrefix: "DL-",
    description: "영업 기회",
    status: "active",
    activeCount: 4,
  };

  it("denies the prefix before the registry is primed", () => {
    expect(spans("신규 DL-1042 검토")).toEqual([]);
    expect(linkTargetFromCode("DL-1042")).toBeUndefined();
    expect(slugLabel("deal")).toBe("deal");
  });

  it("linkifies, resolves a card slug and labels it once primed — no map edited", () => {
    primeObjectTypeRegistry([DEAL]);

    expect(spans("신규 DL-1042 검토")).toEqual([{ kind: "codeLink", raw: "DL-1042" }]);
    expect(linkTargetFromCode("DL-1042")).toEqual({ kind: "deal", id: "DL-1042" });
    expect(slugLabel("deal")).toBe("영업 기회");
    expect(slugTone("deal")).toEqual(TONE("neutral"));
  });

  it("keeps the seeded floor intact after priming (union, never replace)", () => {
    primeObjectTypeRegistry([DEAL]);
    expect(spans("결재 AP-3122 확인")).toEqual([{ kind: "codeLink", raw: "AP-3122" }]);
    expect(linkTargetFromCode("WO-2643")).toEqual({ kind: "work_order", id: "WO-2643" });
  });

  it("does not resolve a slug for a non-active registered type (deny by default)", () => {
    primeObjectTypeRegistry([{ ...DEAL, status: "draft" }]);
    expect(linkTargetFromCode("DL-1042")).toBeUndefined();
  });

  /**
   * The zero-edit claim's exact upper bound, pinned so it cannot rot into an
   * overstatement. Parse and card-slug resolution ARE dynamic; the composer
   * CHIP is not. `kindFromCode` returns the closed `ObjectKind` union, so a
   * runtime-registered prefix carries no tone and no Korean kind label and
   * `TokenText.tsx:73-84` renders it as inert raw text. Closing that needs a
   * dynamic kind in TokenText.tsx, ModuleScreen.tsx and TokenComposer.tsx —
   * none of them L-F3 roots. L-X10 consequence: a `DL-` code links and
   * resolves a card, but draws no chip until that lands.
   */
  it("parses and resolves but draws no composer chip — the named gap, executable", () => {
    primeObjectTypeRegistry([DEAL]);
    expect(spans("신규 DL-1042 검토")).toEqual([{ kind: "codeLink", raw: "DL-1042" }]);
    expect(linkTargetFromCode("DL-1042")).toEqual({ kind: "deal", id: "DL-1042" });
    expect(kindFromCode("DL-1042")).toBeUndefined();
  });

  /**
   * The label half of the zero-edit story costs something, and the cost is a
   * backend obligation, not a frontend one. `object_types.description` is
   * English prose in EVERY row this repo seeds (0102/0115/0131/0188), so
   * `slugLabel` returns prose verbatim unless the registering lane authors a
   * Korean display name. `series` is the live case: it is in
   * `RESOLVABLE_KIND_AUTH` and has an `SR-` prefix, so it reaches this path
   * today. The Korean-fixture test above proves the WIRING; this proves the
   * CONTRACT it depends on, so neither is mistaken for the other.
   */
  it("returns the registry description verbatim — prose in, prose out", () => {
    primeObjectTypeRegistry([
      {
        kind: "series",
        codePrefix: "SR-",
        description: "A user-authored series grouping recurring instances",
        status: "active",
        activeCount: 2,
      },
    ]);
    expect(slugLabel("series")).toBe("A user-authored series grouping recurring instances");
    expect(linkTargetFromCode("SR-77")).toEqual({ kind: "series", id: "SR-77" });
  });
});
