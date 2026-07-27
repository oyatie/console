// The dynamic console has two intentionally separate sources. Bootstrap
// GET /api/v1/object-types supplies only kind/prefix/count metadata for code
// grammar and module discovery. A registered unknown kind gets its render
// schema and instance rows only from canonical GET /ontology/object-types/{key}
// plus GET /ontology/instances?type=<returned-version-uuid> below.
//
// Fail-closed: on any network/parse error the cache and the static-fallback
// grammar are left intact — never emptied — and the fetch resolves to the last
// good cache (or []). Bootstrap metadata is never treated as field/schema
// authority and can never synthesize a generic code/title surface.
//
// The fetch is co-located here (not api/ontology.ts) to keep this lane's files
// self-contained; api/ontology.ts is under concurrent edit by the serial-wire
// lane. It still goes through the generated typed client, so the path/response
// are compile-checked.
import {
  getObjectType,
  listInstances,
  type InstanceStateWire,
  type ObjectTypeDetailWire,
} from "../../api/ontology";
import type { ConsoleApiClient } from "../../api/client";

import { primeCodePrefixes } from "./codeGrammar";
import { objectTypeDefFromDetail } from "./wire";
import type { OntObjectTypeDef } from "./types";

/** ObjectTypeResponse — one seeded object-type registry head. */
export interface RegistryObjectType {
  kind: string;
  /** Canonical per-kind code prefix (e.g. "AP-", "CS-"); null for id/name-referenced kinds. */
  codePrefix: string | null;
  description: string;
  status: "draft" | "active" | "archived";
  /** Instances visible to the caller (same per-kind visibility as resolveObject). */
  activeCount: number;
}

let cachedTypes: readonly RegistryObjectType[] | null = null;
let objectTypeRegistryGeneration = 0;
const canonicalTypesByAuthority = new Map<string, Map<string, CanonicalObjectType>>();
const canonicalGenerationsByAuthority = new Map<string, Map<string, number>>();

/** A detail/schema snapshot that was read under one effective authority only. */
export interface CanonicalObjectType {
  detail: ObjectTypeDetailWire;
  definition: OntObjectTypeDef;
  instances: readonly InstanceStateWire[];
}

function authorityCache(authorityKey: string): Map<string, CanonicalObjectType> {
  let cache = canonicalTypesByAuthority.get(authorityKey);
  if (!cache) {
    cache = new Map();
    canonicalTypesByAuthority.set(authorityKey, cache);
  }
  return cache;
}

function nextCanonicalGeneration(authorityKey: string, stableKey: string): number {
  let generations = canonicalGenerationsByAuthority.get(authorityKey);
  if (!generations) {
    generations = new Map();
    canonicalGenerationsByAuthority.set(authorityKey, generations);
  }
  const generation = (generations.get(stableKey) ?? 0) + 1;
  generations.set(stableKey, generation);
  return generation;
}

function isCurrentCanonicalGeneration(authorityKey: string, stableKey: string, generation: number): boolean {
  return canonicalGenerationsByAuthority.get(authorityKey)?.get(stableKey) === generation;
}

/**
 * Returns a prior canonical read only when it belongs to the exact tenant /
 * session authority which loaded it. There is deliberately no global fallback.
 */
export function canonicalObjectType(
  stableKey: string,
  authorityKey: string,
): CanonicalObjectType | undefined {
  return canonicalTypesByAuthority.get(authorityKey)?.get(stableKey);
}

/**
 * Reads the real ontology schema and its instances for a registered no-code
 * kind. The instance endpoint is pinned to the detail response's version UUID;
 * a cross-version response is rejected rather than rendered.
 */
export async function loadCanonicalObjectType(
  api: ConsoleApiClient,
  stableKey: string,
  authorityKey: string,
  signal?: AbortSignal,
): Promise<CanonicalObjectType> {
  if (!authorityKey.trim()) throw new Error("canonical ontology reads require an authority key");
  const cache = authorityCache(authorityKey);
  const generation = nextCanonicalGeneration(authorityKey, stableKey);
  // A failed refresh must never leave previously authorized or stale data
  // eligible for this authority to render.
  cache.delete(stableKey);
  try {
    const detail = await getObjectType(api, stableKey, undefined, { signal, forceRefresh: true });
    if (detail.object_type.stable_key !== stableKey) {
      throw new Error("canonical ontology stable key mismatch");
    }
    const instances = await listInstances(api, detail.object_type.id, { signal, forceRefresh: true });
    if (instances.some((state) => state.instance.object_type_id !== detail.object_type.id)) {
      throw new Error("canonical ontology type version mismatch");
    }
    const definition = objectTypeDefFromDetail(detail, instances, new Map([[detail.object_type.id, stableKey]]));
    const canonical = { detail, definition, instances } satisfies CanonicalObjectType;
    if (isCurrentCanonicalGeneration(authorityKey, stableKey, generation)) {
      cache.set(stableKey, canonical);
    }
    return canonical;
  } catch (error) {
    if (isCurrentCanonicalGeneration(authorityKey, stableKey, generation)) {
      cache.delete(stableKey);
    }
    throw error;
  }
}

/** The last-loaded registry, or null before the bootstrap fetch lands. */
export function registeredObjectTypes(): readonly RegistryObjectType[] | null {
  return cachedTypes;
}

/** One registered type by kind, or undefined (unknown / not yet loaded). */
export function registeredObjectType(kind: string): RegistryObjectType | undefined {
  return cachedTypes?.find((type) => type.kind === kind);
}

function ingest(types: readonly RegistryObjectType[]): void {
  cachedTypes = types;
  primeCodePrefixes(types.map((type) => type.codePrefix));
}

/**
 * Bootstrap fetch — call once at app start (see report: shell wiring seam).
 * Loads the object-type registry, caches it, and primes the code-prefix
 * grammar. Fail-closed: a network/parse error leaves the previous cache and the
 * static-fallback grammar untouched and resolves to the last good cache (or []).
 */
export async function loadObjectTypeRegistry(
  api: ConsoleApiClient,
): Promise<readonly RegistryObjectType[]> {
  try {
    const { data } = await api.GET("/api/v1/object-types");
    if (!data) return cachedTypes ?? [];
    const types: RegistryObjectType[] = data.map((row) => ({
      kind: row.kind,
      codePrefix: row.code_prefix ?? null,
      description: row.description,
      status: row.status,
      activeCount: row.active_count,
    }));
    ingest(types);
    return types;
  } catch {
    return cachedTypes ?? [];
  }
}

/**
 * Loads the discovery registry for one exact effective authority. A newer
 * authority supersedes prior bootstrap requests, so an old response cannot
 * make its discovered kinds available to the current route.
 */
export async function loadObjectTypeRegistryForAuthority(
  api: ConsoleApiClient,
  authorityKey: string,
  signal?: AbortSignal,
): Promise<readonly RegistryObjectType[]> {
  if (!authorityKey.trim()) throw new Error("object-type registry requires an authority key");
  const generation = objectTypeRegistryGeneration + 1;
  objectTypeRegistryGeneration = generation;
  const isCurrent = () =>
    !signal?.aborted && objectTypeRegistryGeneration === generation;

  try {
    const { data } = await api.GET("/api/v1/object-types", { signal });
    if (!isCurrent()) throw new Error("object-type registry invalidated");
    if (!data) throw new Error("object-type registry response missing data");
    const types: RegistryObjectType[] = data.map((row) => ({
      kind: row.kind,
      codePrefix: row.code_prefix ?? null,
      description: row.description,
      status: row.status,
      activeCount: row.active_count,
    }));
    ingest(types);
    return types;
  } catch (error) {
    if (!isCurrent()) {
      throw new Error("object-type registry invalidated", { cause: error });
    }
    throw error;
  }
}

/** Seed the cache + grammar from a payload directly. Bootstrap/test seam. */
export function primeObjectTypeRegistry(types: readonly RegistryObjectType[]): void {
  ingest(types);
}

/** Clear the cache. Test isolation only (pair with codeGrammar.resetCodePrefixes). */
export function resetObjectTypeRegistry(): void {
  cachedTypes = null;
  objectTypeRegistryGeneration = 0;
  canonicalTypesByAuthority.clear();
  canonicalGenerationsByAuthority.clear();
}
