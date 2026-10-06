// This document projection selects presentation only. The existing owners
// recheck every form proof, policy decision, input and expected revision.
export type Scope = {
  company: string; company_name: string | null; company_link: boolean;
  directory_link: boolean; can_create: boolean; payroll_link: boolean; policy_link: boolean;
};
export type PersonRecord = {
  employee_id: string; person_id: string; legal_name: string | null;
  employee_number: string | null; person_version: string; registered_at: string;
};
export type Expectations = {
  company_epoch: string; object_type_id: string; action_type_id: string;
  action_revision: string; schema_revision: string;
  legal_name_property_id: string; employee_number_property_id: string;
};
export type Registration = {
  command: string; proof: string; expected: Expectations; legal_name: string;
  employee_number: string; name_error: string | null; number_error: string | null;
  form_error: string | null;
};
export type Outcome =
  | {kind: "pending"; proof: string}
  | {kind: "committed"; employee_id: string; person_id: string; receipt: string; registered_at: string}
  | {kind: "rejected" | "conflicting"; reason: string}
  | {kind: "cancelled" | "expired"};
export type PeopleRequest = {
  command: string; legal_name: string; employee_number: string; accepted_at: string;
  deadline: string; intake_receipt: string; expected_company_epoch: string; outcome: Outcome;
};
export type Page =
  | {kind: "directory"; scope: Scope; records: PersonRecord[]; search_number: string | null; next_href: string | null; after_cursor: boolean}
  | {kind: "registration"; scope: Scope; form: Registration}
  | {kind: "registration_conflict"; scope: Scope; command: string; legal_name: string; employee_number: string}
  | {kind: "request"; scope: Scope; request: PeopleRequest}
  | {kind: "request_not_visible"; scope: Scope; command: string}
  | {kind: "detail"; scope: Scope; record: PersonRecord}
  | {kind: "uncertain"; company: string; command: string};

type ObjectValue = {[key: string]: unknown};
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const NIL_UUID = "00000000-0000-0000-0000-000000000000";
function reject(): never { throw new Error("INVALID_PEOPLE_PROJECTION"); }
function object(value: unknown, keys: readonly string[]): ObjectValue {
  if (value === null || typeof value !== "object" || Array.isArray(value)) reject();
  const candidate = value as ObjectValue;
  const actual = Object.keys(candidate);
  if (actual.length !== keys.length || !keys.every(key => Object.hasOwn(candidate, key))) reject();
  return candidate;
}
function text(value: unknown, maximum: number, minimum = 0): string {
  if (typeof value !== "string") reject();
  // Rust owner bounds count Unicode scalars, not UTF-16 code units. Reject
  // lone surrogates and never truncate a retained draft to fit the projection.
  let count = 0;
  for (const character of value) {
    const point = character.codePointAt(0)!;
    if ((point >= 0xd800 && point <= 0xdfff) || ++count > maximum) reject();
  }
  if (count < minimum) reject();
  return value;
}
function nullable(value: unknown, maximum: number): string | null {
  return value === null ? null : text(value, maximum);
}
function id(value: unknown): string {
  const result = text(value, 36, 36);
  if (!UUID.test(result) || result === NIL_UUID) reject();
  return result;
}
function revision(value: unknown): string {
  const result = text(value, 20, 1);
  if (!/^(0|[1-9][0-9]*)$/.test(result)) reject();
  return result;
}
function bool(value: unknown): boolean {
  if (typeof value !== "boolean") reject();
  return value;
}
function scope(value: unknown): Scope {
  const v = object(value, ["company", "company_name", "company_link", "directory_link", "can_create", "payroll_link", "policy_link"]);
  return {
    company: id(v.company), company_name: nullable(v.company_name, 500),
    company_link: bool(v.company_link), directory_link: bool(v.directory_link),
    can_create: bool(v.can_create), payroll_link: bool(v.payroll_link), policy_link: bool(v.policy_link),
  };
}
function record(value: unknown): PersonRecord {
  const v = object(value, ["employee_id", "person_id", "legal_name", "employee_number", "person_version", "registered_at"]);
  return {
    employee_id: id(v.employee_id), person_id: id(v.person_id), legal_name: nullable(v.legal_name, 200),
    employee_number: nullable(v.employee_number, 64), person_version: revision(v.person_version), registered_at: text(v.registered_at, 128, 1),
  };
}
function expectations(value: unknown): Expectations {
  const v = object(value, ["company_epoch", "object_type_id", "action_type_id", "action_revision", "schema_revision", "legal_name_property_id", "employee_number_property_id"]);
  return {
    company_epoch: revision(v.company_epoch), object_type_id: id(v.object_type_id), action_type_id: id(v.action_type_id),
    action_revision: revision(v.action_revision), schema_revision: revision(v.schema_revision),
    legal_name_property_id: id(v.legal_name_property_id), employee_number_property_id: id(v.employee_number_property_id),
  };
}
function registration(value: unknown): Registration {
  const v = object(value, ["command", "proof", "expected", "legal_name", "employee_number", "name_error", "number_error", "form_error"]);
  return {
    command: id(v.command), proof: text(v.proof, 4096, 1), expected: expectations(v.expected),
    legal_name: text(v.legal_name, 200), employee_number: text(v.employee_number, 64),
    name_error: nullable(v.name_error, 1024), number_error: nullable(v.number_error, 1024), form_error: nullable(v.form_error, 1024),
  };
}
function outcome(value: unknown): Outcome {
  if (value === null || typeof value !== "object" || Array.isArray(value)) reject();
  const kind = (value as ObjectValue).kind;
  switch (kind) {
    case "pending": {
      const v = object(value, ["kind", "proof"]);
      return {kind, proof: text(v.proof, 4096, 1)};
    }
    case "committed": {
      const v = object(value, ["kind", "employee_id", "person_id", "receipt", "registered_at"]);
      return {kind, employee_id: id(v.employee_id), person_id: id(v.person_id), receipt: id(v.receipt), registered_at: text(v.registered_at, 128, 1)};
    }
    case "rejected": case "conflicting": {
      const v = object(value, ["kind", "reason"]);
      return {kind, reason: text(v.reason, 1024, 1)};
    }
    case "cancelled": case "expired": object(value, ["kind"]); return {kind};
    default: return reject();
  }
}
function request(value: unknown): PeopleRequest {
  const v = object(value, ["command", "legal_name", "employee_number", "accepted_at", "deadline", "intake_receipt", "expected_company_epoch", "outcome"]);
  return {
    command: id(v.command), legal_name: text(v.legal_name, 200), employee_number: text(v.employee_number, 64),
    accepted_at: text(v.accepted_at, 128, 1), deadline: text(v.deadline, 128, 1), intake_receipt: id(v.intake_receipt),
    expected_company_epoch: revision(v.expected_company_epoch), outcome: outcome(v.outcome),
  };
}
function nextHref(value: unknown, company: string): string | null {
  if (value === null) return null;
  const href = text(value, 4096, 1), path = `/companies/${company}/people`;
  if (!href.startsWith(path + "?") || /[\u0000-\u0020\u007f#\\]/.test(href)) reject();
  const query = new URLSearchParams(href.slice(path.length + 1));
  const keys = [...query.keys()];
  if (keys.length > 2 || new Set(keys).size !== keys.length || !keys.every(key => key === "employee_number" || key === "after_employee_id")) reject();
  const after = query.get("after_employee_id");
  if (after === null) reject();
  id(after);
  const number = query.get("employee_number");
  if (number !== null) text(number, 64);
  return href;
}
export function decodeProjection(value: unknown): Page {
  const envelope = object(value, ["version", "page"]);
  if (envelope.version !== 1) reject();
  const page = envelope.page;
  if (page === null || typeof page !== "object" || Array.isArray(page)) reject();
  const kind = (page as ObjectValue).kind;
  switch (kind) {
    case "directory": {
      const v = object(page, ["kind", "scope", "records", "search_number", "next_href", "after_cursor"]), s = scope(v.scope);
      if (!Array.isArray(v.records) || v.records.length > 100) reject();
      return {kind, scope: s, records: v.records.map(record), search_number: nullable(v.search_number, 64), next_href: nextHref(v.next_href, s.company), after_cursor: bool(v.after_cursor)};
    }
    case "registration": {
      const v = object(page, ["kind", "scope", "form"]);
      return {kind, scope: scope(v.scope), form: registration(v.form)};
    }
    case "registration_conflict": {
      const v = object(page, ["kind", "scope", "command", "legal_name", "employee_number"]);
      return {kind, scope: scope(v.scope), command: id(v.command), legal_name: text(v.legal_name, 200), employee_number: text(v.employee_number, 64)};
    }
    case "request": {
      const v = object(page, ["kind", "scope", "request"]);
      return {kind, scope: scope(v.scope), request: request(v.request)};
    }
    case "request_not_visible": {
      const v = object(page, ["kind", "scope", "command"]);
      return {kind, scope: scope(v.scope), command: id(v.command)};
    }
    case "detail": {
      const v = object(page, ["kind", "scope", "record"]);
      return {kind, scope: scope(v.scope), record: record(v.record)};
    }
    case "uncertain": {
      const v = object(page, ["kind", "company", "command"]);
      return {kind, company: id(v.company), command: id(v.command)};
    }
    default: return reject();
  }
}
