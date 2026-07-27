import {
  getInstance,
  getInstanceHistory,
  type ObjectTypeDetailWire,
  traverseInstance,
} from "../../api/ontology";
import type { ConsoleApiClient } from "../../api/client";
import { objectCardDescriptorFrom } from "../ontology";
import type { ObjectCardDescriptor } from "../objectcard/types";
import type { EntityRef } from "./entityRef";

export interface ObjectRuntimePort {
  readonly authority: EntityRef["authority"];
  resolve(
    ref: EntityRef,
    options: { signal: AbortSignal },
  ): Promise<ObjectCardDescriptor | undefined>;
}

/**
 * Ontology adapter: all three detail reads are scoped by the authenticated API
 * client. A reference from another authority is omitted before any read, so it
 * cannot reveal existence/counts across a tenant or session transition.
 */
export function createOntologyObjectRuntime(input: {
  api: ConsoleApiClient;
  authorityKey: string;
  tenantScopeKey: string;
  detailForObjectType: (objectTypeId: string) => ObjectTypeDetailWire | undefined;
  linkTitleById: ReadonlyMap<string, string>;
}): ObjectRuntimePort {
  const authorityKey = input.authorityKey;
  const tenantScopeKey = input.tenantScopeKey;
  return {
    authority: "ontology",
    async resolve(ref, { signal }) {
      if (
        ref.authorityKey !== authorityKey ||
        ref.tenantScopeKey !== tenantScopeKey ||
        signal.aborted
      ) {
        return undefined;
      }
      const detail = input.detailForObjectType(ref.objectTypeId);
      if (detail === undefined) return undefined;

      const [state, history, neighbors] = await Promise.all([
        getInstance(input.api, ref.id, { signal }),
        getInstanceHistory(input.api, ref.id, { signal }),
        traverseInstance(input.api, ref.id, { depth: 1, signal }),
      ]);
      if (state.instance.object_type_id !== ref.objectTypeId) {
        return undefined;
      }
      return objectCardDescriptorFrom({
        state,
        history,
        neighbors,
        detail,
        linkTitleById: input.linkTitleById,
      });
    },
  };
}
