// Authorized presentation only. Rust owners recheck every identity and action.
export type Terms = {kind: string; title: string; content_url: string; content: string};
export type Context = {kind: "empty" | "unavailable"} | {kind: "companies"; companies: [string, string][]};
export type Page =
  | {kind: "public" | "sign_in" | "company_uncertain" | "refused" | "unavailable"}
  | {kind: "register"; version: string; items: Terms[]}
  | {kind: "account"; account_id: string; context: Context; can_logout: boolean; company_setup: "eligible" | "ineligible" | "unavailable"}
  | {kind: "company_setup"; account_id: string; command_id: string | null}
  | {kind: "company_created"; command_id: string; receipt_id: string; administrative_account_id: string; company: [string, string] | null}
  | {kind: "company_pending"; command_id: string; name: string; slug: string; account_id: string; group_id: string | null}
  | {kind: "company_terminal"; command_id: string; expired: boolean};
export type Projection = {version: 1; page: Page; groups: {group: string; label: string}[]};
type RecordValue = {[key: string]: unknown};
function reject(): never { throw new Error("INVALID_ACCOUNT_PROJECTION"); }
function object(value: unknown, keys: readonly string[]): RecordValue {
  if (value === null || typeof value !== "object" || Array.isArray(value)) reject();
  const v = value as RecordValue;
  if (Object.keys(v).length !== keys.length || !keys.every(key => Object.hasOwn(v, key))) reject();
  return v;
}
function text(value: unknown, maximum: number, minimum = 0): string {
  if (typeof value !== "string" || value.length < minimum || value.length > maximum) reject();
  for (const character of value) {
    const point = character.codePointAt(0)!;
    if (point >= 0xd800 && point <= 0xdfff) reject();
  }
  return value;
}
function id(value: unknown): string {
  const result = text(value, 36, 36);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(result) || result === "00000000-0000-0000-0000-000000000000") reject();
  return result;
}
function optionalId(value: unknown): string | null { return value === null ? null : id(value); }
function bool(value: unknown): boolean { if (typeof value !== "boolean") reject(); return value; }
function company(value: unknown): [string, string] {
  if (!Array.isArray(value) || value.length !== 2) reject();
  return [id(value[0]), text(value[1], 4096, 1)];
}
function context(value: unknown): Context {
  if (value === null || typeof value !== "object" || Array.isArray(value)) reject();
  const kind = (value as RecordValue).kind;
  if (kind === "empty" || kind === "unavailable") { object(value, ["kind"]); return {kind}; }
  if (kind !== "companies") reject();
  const v = object(value, ["kind", "companies"]);
  if (!Array.isArray(v.companies) || !v.companies.length || v.companies.length > 4096) reject();
  const companies = v.companies.map(company);
  if (new Set(companies.map(([identifier]) => identifier)).size !== companies.length) reject();
  return {kind, companies};
}
function terms(value: unknown): Terms {
  const v = object(value, ["kind", "title", "content_url", "content"]);
  const url = text(v.content_url, 2048, 1);
  if (/[\u0000-\u0020\u007f\\]/.test(url)) reject();
  if (!url.startsWith("/") || url.startsWith("//")) {
    let parsed: URL;
    try { parsed = new URL(url); } catch { return reject(); }
    if (parsed.protocol !== "https:" || parsed.username || parsed.password) reject();
  }
  return {kind: text(v.kind, 128, 1), title: text(v.title, 4096, 1), content_url: url, content: text(v.content, 262144, 1)};
}
function page(value: unknown): Page {
  if (value === null || typeof value !== "object" || Array.isArray(value)) reject();
  const kind = (value as RecordValue).kind;
  switch (kind) {
    case "public": case "sign_in": case "company_uncertain": case "refused": case "unavailable": object(value, ["kind"]); return {kind};
    case "register": {
      const v = object(value, ["kind", "version", "items"]);
      if (!Array.isArray(v.items) || !v.items.length || v.items.length > 32) reject();
      const items = v.items.map(terms);
      if (new Set(items.map(item => item.kind)).size !== items.length) reject();
      return {kind, version: text(v.version, 4096, 1), items};
    }
    case "account": {
      const v = object(value, ["kind", "account_id", "context", "can_logout", "company_setup"]);
      if (v.company_setup !== "eligible" && v.company_setup !== "ineligible" && v.company_setup !== "unavailable") reject();
      return {kind, account_id: id(v.account_id), context: context(v.context), can_logout: bool(v.can_logout), company_setup: v.company_setup};
    }
    case "company_setup": {
      const v = object(value, ["kind", "account_id", "command_id"]);
      return {kind, account_id: id(v.account_id), command_id: optionalId(v.command_id)};
    }
    case "company_created": {
      const v = object(value, ["kind", "command_id", "receipt_id", "administrative_account_id", "company"]);
      return {kind, command_id: id(v.command_id), receipt_id: id(v.receipt_id), administrative_account_id: id(v.administrative_account_id), company: v.company === null ? null : company(v.company)};
    }
    case "company_pending": {
      const v = object(value, ["kind", "command_id", "name", "slug", "account_id", "group_id"]);
      const name = text(v.name, 256, 1), slug = text(v.slug, 63, 1);
      if (new TextEncoder().encode(name).length > 256 || /^\p{White_Space}*$/u.test(name) || /[\u0000-\u001f\u007f-\u009f]/.test(name) || !/^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/.test(slug)) reject();
      return {kind, command_id: id(v.command_id), name, slug, account_id: id(v.account_id), group_id: optionalId(v.group_id)};
    }
    case "company_terminal": {
      const v = object(value, ["kind", "command_id", "expired"]);
      return {kind, command_id: id(v.command_id), expired: bool(v.expired)};
    }
    default: return reject();
  }
}
export function decodeProjection(value: unknown): Projection {
  const v = object(value, ["version", "page", "groups"]);
  if (v.version !== 1 || !Array.isArray(v.groups) || v.groups.length > 4096) reject();
  const decoded = page(v.page), groups = v.groups.map(value => {
    const group = object(value, ["group", "label"]);
    return {group: id(group.group), label: text(group.label, 4096, 1)};
  });
  if ((decoded.kind !== "account" && groups.length) || new Set(groups.map(group => group.group)).size !== groups.length) reject();
  return {version: 1, page: decoded, groups};
}
