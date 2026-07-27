import { afterEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import {
  canOpenTypeCard,
  getObjectType,
  rowCardDescriptor,
  typeCardDescriptor,
} from "../modules/typeRegistry";
import type { ModuleRow } from "../modules/types";
import {
  canonicalObjectType,
  loadCanonicalObjectType,
  resetObjectTypeRegistry,
} from "./typeRegistrySource";

const DETAIL = {
  object_type: {
    id: "00000000-0000-4000-8000-000000000701",
    stable_key: "widget",
    title: "Widget",
    backing_kind: "instance",
    schema_version: 7,
    lifecycle_state: "published",
    key_write_revision: 7,
    key_write_etag: '"widget-r7"',
  },
  title_property_key: "label",
  backing_table: null,
  primary_key_property: null,
  properties: [
    {
      id: "00000000-0000-4000-8000-000000000702",
      key: "label",
      title: "Label",
      field_type: "text",
      config: {},
      backing_column: null,
      required: true,
      in_property_policy: false,
    },
  ],
  links: [],
  actions: [],
  analytics: [],
};

const INSTANCE = {
  instance: {
    id: "00000000-0000-4000-8000-000000000703",
    object_type_id: DETAIL.object_type.id,
    title: "Real widget",
    current_revision_id: "00000000-0000-4000-8000-000000000704",
    lifecycle_state: "active",
  },
  revision: {
    id: "00000000-0000-4000-8000-000000000704",
    instance_id: "00000000-0000-4000-8000-000000000703",
    version: 2,
    attributes: { label: "Real label" },
    valid_from: "2026-07-25T00:00:00Z",
    valid_to: null,
    action_type_id: null,
    actor: null,
    reason: null,
    prev_hash: "0".repeat(64),
    row_hash: "a".repeat(64),
  },
};

function deferred<T>() {
  let resolve: (value: T) => void = () => {
    throw new Error("deferred promise was not initialized");
  };
  let reject: (reason?: unknown) => void = () => {
    throw new Error("deferred promise was not initialized");
  };
  const promise = new Promise<T>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });
  return { promise, resolve, reject };
}

const NEWER_DETAIL = {
  ...DETAIL,
  object_type: { ...DETAIL.object_type, schema_version: 8, key_write_revision: 8 },
};

afterEach(() => {
  resetObjectTypeRegistry();
  vi.restoreAllMocks();
});

describe("canonical dynamic object-type source", () => {
  it("uses the canonical detail's real version UUID to load instances and caches the governed schema only in its authority scope", async () => {
    const GET = vi.fn()
      .mockResolvedValueOnce({ data: DETAIL, response: new Response() })
      .mockResolvedValueOnce({ data: [INSTANCE], response: new Response() });
    const api = { GET } as unknown as ConsoleApiClient;

    await expect(loadCanonicalObjectType(api, "widget", "tenant-a:session-1")).resolves.toMatchObject({
      definition: {
        id: DETAIL.object_type.id,
        schemaVersion: 7,
        properties: [{ key: "label", title: "Label" }],
      },
      instances: [INSTANCE],
    });
    expect(GET.mock.calls[1]?.[1]).toMatchObject({
      params: { query: { type: DETAIL.object_type.id } },
    });
    expect(canonicalObjectType("widget", "tenant-a:session-1")?.definition.schemaVersion).toBe(7);
    expect(canonicalObjectType("widget", "tenant-b:session-1")).toBeUndefined();
  });

  it.each([403, 404])("does not retain a canonical result after a %i response", async (status) => {
    const api = {
      GET: vi.fn().mockResolvedValue({
        data: undefined,
        error: { error: { code: "DENIED", message: "denied" } },
        response: new Response(null, { status }),
      }),
    } as unknown as ConsoleApiClient;

    await expect(loadCanonicalObjectType(api, "widget", "tenant-a:session-1")).rejects.toMatchObject({ status });
    expect(canonicalObjectType("widget", "tenant-a:session-1")).toBeUndefined();
  });

  it("fails closed when the instance API returns a different type-version UUID", async () => {
    const GET = vi.fn()
      .mockResolvedValueOnce({ data: DETAIL, response: new Response() })
      .mockResolvedValueOnce({
        data: [{ ...INSTANCE, instance: { ...INSTANCE.instance, object_type_id: "wrong-version" } }],
        response: new Response(),
      });
    const api = { GET } as unknown as ConsoleApiClient;

    await expect(loadCanonicalObjectType(api, "widget", "tenant-a:session-1")).rejects.toThrow(
      "type version mismatch",
    );
    expect(canonicalObjectType("widget", "tenant-a:session-1")).toBeUndefined();
  });

  it("does not let an older same-authority resolve replace a newer cache entry", async () => {
    const oldDetail = deferred<{ data: typeof DETAIL; response: Response }>();
    const oldInstances = deferred<{ data: typeof INSTANCE[]; response: Response }>();
    const GET = vi.fn()
      .mockImplementationOnce(() => oldDetail.promise)
      .mockResolvedValueOnce({ data: NEWER_DETAIL, response: new Response() })
      .mockResolvedValueOnce({ data: [INSTANCE], response: new Response() })
      .mockImplementationOnce(() => oldInstances.promise);
    const api = { GET } as unknown as ConsoleApiClient;

    const older = loadCanonicalObjectType(api, "widget", "tenant-a:session-1");
    const newer = loadCanonicalObjectType(api, "widget", "tenant-a:session-1");
    await newer;
    oldDetail.resolve({ data: DETAIL, response: new Response() });
    await vi.waitFor(() => {
      expect(GET).toHaveBeenCalledTimes(4);
    });
    oldInstances.resolve({ data: [INSTANCE], response: new Response() });
    await older;

    expect(canonicalObjectType("widget", "tenant-a:session-1")?.definition.schemaVersion).toBe(8);
  });

  it("does not let an older same-authority rejection delete a newer cache entry", async () => {
    const oldDetail = deferred<{ data: typeof DETAIL; response: Response }>();
    const GET = vi.fn()
      .mockImplementationOnce(() => oldDetail.promise)
      .mockResolvedValueOnce({ data: NEWER_DETAIL, response: new Response() })
      .mockResolvedValueOnce({ data: [INSTANCE], response: new Response() });
    const api = { GET } as unknown as ConsoleApiClient;

    const older = loadCanonicalObjectType(api, "widget", "tenant-a:session-1");
    await loadCanonicalObjectType(api, "widget", "tenant-a:session-1");
    oldDetail.reject(new Error("older request failed"));
    await expect(older).rejects.toThrow("older request failed");

    expect(canonicalObjectType("widget", "tenant-a:session-1")?.definition.schemaVersion).toBe(8);
  });

  it("threads canonical schema version and draft/archived lifecycle values into type and instance cards", async () => {
    const detail = {
      ...DETAIL,
      object_type: { ...DETAIL.object_type, lifecycle_state: "draft" as const, schema_version: 9 },
    };
    const instance = {
      ...INSTANCE,
      instance: { ...INSTANCE.instance, lifecycle_state: "archived" as const },
    };
    const api = {
      GET: vi.fn()
        .mockResolvedValueOnce({ data: detail, response: new Response() })
        .mockResolvedValueOnce({ data: [instance], response: new Response() }),
    } as unknown as ConsoleApiClient;

    await loadCanonicalObjectType(api, "widget", "tenant-a:session-1");
    const type = getObjectType("widget", "tenant-a:session-1");
    expect(type).toBeDefined();
    if (!type) throw new Error("canonical type must be registered");
    const row: ModuleRow = {
      id: instance.instance.id,
      code: "W-703",
      title: instance.instance.title,
      cells: { label: "Real label" },
      sourceRecord: instance,
    };

    expect(typeCardDescriptor(type)).toMatchObject({
      objectType: { id: detail.object_type.id },
      lifecycleState: "draft",
      schemaVersion: 9,
    });
    expect(rowCardDescriptor(type, row)).toMatchObject({
      lifecycleState: "archived",
      schemaVersion: 9,
    });
  });

  it("omits a type-card representation when canonical schema lifecycle is not an instance lifecycle", async () => {
    const api = {
      GET: vi.fn()
        .mockResolvedValueOnce({ data: DETAIL, response: new Response() })
        .mockResolvedValueOnce({ data: [INSTANCE], response: new Response() }),
    } as unknown as ConsoleApiClient;

    await loadCanonicalObjectType(api, "widget", "tenant-a:session-1");
    const type = getObjectType("widget", "tenant-a:session-1");
    expect(type).toBeDefined();
    if (!type) throw new Error("canonical type must be registered");
    expect(canOpenTypeCard(type)).toBe(false);
    expect(() => typeCardDescriptor(type)).toThrow("no instance-card representation");
  });
});
