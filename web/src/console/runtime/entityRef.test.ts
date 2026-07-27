import { describe, expect, it } from "vitest";

import { entityRefKey, ontologyEntityRef } from "./entityRef";

describe("ontologyEntityRef", () => {
  it("requires explicit tenant and authority partitions and excludes display hints from identity", () => {
    const ref = ontologyEntityRef({
      tenantScopeKey: "tenant-a",
      authorityKey: "authority-a",
      objectTypeId: "type-a",
      id: "object-a",
      codeHint: "WO-1",
      titleHint: "Work order",
    });
    expect(ref).toMatchObject({ authority: "ontology", tenantScopeKey: "tenant-a", authorityKey: "authority-a" });
    if (!ref) throw new Error("expected a valid ontology entity reference");
    expect(entityRefKey(ref)).not.toContain("WO-1");
    expect(entityRefKey(ref)).not.toContain("Work%20order");
    expect(ontologyEntityRef({ tenantScopeKey: "", authorityKey: "a", objectTypeId: "t", id: "o" })).toBeUndefined();
  });
});
