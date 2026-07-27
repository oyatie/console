import { describe, expect, it, vi } from "vitest";

import type { EntityRef } from "./entityRef";
import { ontologyEntityRef } from "./entityRef";
import { createOntologyObjectRuntime } from "./objectRuntime";

const objectTypeDetail = {
  object_type: {
    id: "type-a",
    stable_key: "work_order",
    title: "작업지시",
    backing_kind: "instance",
    schema_version: 1,
    lifecycle_state: "published",
    key_write_revision: 1,
    key_write_etag: "\"type-a:1\"",
  },
  title_property_key: null,
  backing_table: null,
  primary_key_property: null,
  properties: [],
  links: [],
  actions: [],
  analytics: [],
} as const;

const instanceState = {
  instance: {
    id: "object-a",
    object_type_id: "type-a",
    title: "WO-1",
    current_revision_id: "revision-a",
    lifecycle_state: "active",
  },
  revision: {
    id: "revision-a",
    instance_id: "object-a",
    version: 1,
    attributes: {},
    valid_from: "2026-07-25T00:00:00Z",
    valid_to: null,
    action_type_id: null,
    actor: null,
    reason: null,
    prev_hash: "0".repeat(64),
    row_hash: "1".repeat(64),
  },
};

function apiResult(data: unknown) {
  return { data, error: undefined, response: { status: 200 } };
}

function requiredRef(
  input: Parameters<typeof ontologyEntityRef>[0],
): EntityRef {
  const reference = ontologyEntityRef(input);
  if (!reference) throw new Error("expected a valid ontology entity reference");
  return reference;
}

function matchingRef(): EntityRef {
  return requiredRef({
    tenantScopeKey: "tenant-a",
    authorityKey: "authority-a",
    objectTypeId: "type-a",
    id: "object-a",
  });
}

describe("createOntologyObjectRuntime", () => {
  it("omits cross-authority references before issuing any backend read", async () => {
    const get = vi.fn();
    const runtime = createOntologyObjectRuntime({
      api: { GET: get } as never,
      authorityKey: "authority-a",
      tenantScopeKey: "tenant-a",
      detailForObjectType: () => undefined,
      linkTitleById: new Map(),
    });
    const ref = requiredRef({ tenantScopeKey: "tenant-b", authorityKey: "authority-b", objectTypeId: "type-a", id: "object-a" });
    await expect(runtime.resolve(ref, { signal: new AbortController().signal })).resolves.toBeUndefined();
    expect(get).not.toHaveBeenCalled();
  });

  it("rejects an already-aborted resolution without exposing a descriptor", async () => {
    const runtime = createOntologyObjectRuntime({
      api: {} as never,
      authorityKey: "authority-a",
      tenantScopeKey: "tenant-a",
      detailForObjectType: () => undefined,
      linkTitleById: new Map(),
    });
    const controller = new AbortController();
    controller.abort();
    const ref = matchingRef();
    await expect(runtime.resolve(ref, { signal: controller.signal })).resolves.toBeUndefined();
  });

  it("fails closed before backend reads when authoritative type detail is absent", async () => {
    const get = vi.fn((path: string) => {
      if (path.endsWith("/history")) return Promise.resolve(apiResult([]));
      if (path.endsWith("/traverse")) {
        return Promise.resolve(apiResult({ nodes: [], edges: [] }));
      }
      return Promise.resolve(apiResult(instanceState));
    });
    const runtime = createOntologyObjectRuntime({
      api: { GET: get } as never,
      authorityKey: "authority-a",
      tenantScopeKey: "tenant-a",
      detailForObjectType: () => undefined,
      linkTitleById: new Map(),
    });

    await expect(
      runtime.resolve(matchingRef(), { signal: new AbortController().signal }),
    ).resolves.toBeUndefined();
    expect(get).not.toHaveBeenCalled();
  });

  it("passes one AbortSignal to all three authenticated reads and aborts them", async () => {
    const observedSignals: AbortSignal[] = [];
    const get = vi.fn((_path: string, options: { signal?: AbortSignal }) => {
      const signal = options.signal;
      if (signal === undefined) {
        return Promise.reject(new Error("missing AbortSignal"));
      }
      observedSignals.push(signal);
      return new Promise<never>((_resolve, reject) => {
        signal.addEventListener(
          "abort",
          () => {
            reject(new DOMException("aborted", "AbortError"));
          },
          { once: true },
        );
      });
    });
    const runtime = createOntologyObjectRuntime({
      api: { GET: get } as never,
      authorityKey: "authority-a",
      tenantScopeKey: "tenant-a",
      detailForObjectType: () => objectTypeDetail,
      linkTitleById: new Map(),
    });
    const controller = new AbortController();

    const resolution = runtime.resolve(matchingRef(), {
      signal: controller.signal,
    });
    await vi.waitFor(() => {
      expect(get).toHaveBeenCalledTimes(3);
    });
    controller.abort();

    await expect(resolution).rejects.toMatchObject({ name: "AbortError" });
    expect(observedSignals).toHaveLength(3);
    expect(observedSignals.every((signal) => signal === controller.signal)).toBe(true);
  });
});
