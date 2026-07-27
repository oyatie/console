/**
 * Stable object identity passed between the console's window/surface layers.
 * `tenantScopeKey` and `authorityKey` are opaque client-side partitions only:
 * neither is sent to the backend or treated as authorization evidence.
 */
export interface EntityRef {
  readonly authority: "ontology";
  readonly tenantScopeKey: string;
  readonly authorityKey: string;
  readonly objectTypeId: string;
  readonly id: string;
  readonly codeHint?: string;
  readonly titleHint?: string;
}

export function ontologyEntityRef(input: {
  tenantScopeKey: string;
  authorityKey: string;
  objectTypeId: string;
  id: string;
  codeHint?: string;
  titleHint?: string;
}): EntityRef | undefined {
  const tenantScopeKey = input.tenantScopeKey.trim();
  const authorityKey = input.authorityKey.trim();
  const objectTypeId = input.objectTypeId.trim();
  const id = input.id.trim();
  if (!tenantScopeKey || !authorityKey || !objectTypeId || !id) return undefined;
  return {
    authority: "ontology",
    tenantScopeKey,
    authorityKey,
    objectTypeId,
    id,
    ...(input.codeHint?.trim() ? { codeHint: input.codeHint.trim() } : {}),
    ...(input.titleHint?.trim() ? { titleHint: input.titleHint.trim() } : {}),
  };
}

/** Identity used only for stale-response fencing; it deliberately excludes hints. */
export function entityRefKey(ref: EntityRef): string {
  return [ref.authority, ref.tenantScopeKey, ref.authorityKey, ref.objectTypeId, ref.id]
    .map(encodeURIComponent)
    .join(":");
}
