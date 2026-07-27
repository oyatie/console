import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import type { AuthSession } from "../../context/auth";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { resetObjectTypeRegistry } from "../ontology/typeRegistrySource";
import { ontologyWorkspaceAuthorityKey } from "../ontology/useOntologyRevisionCommitQueue";
import { getObjectType } from "./typeRegistry";
import { ConsoleModuleRoute } from "./ConsoleModuleRoute";

const typeVersionId = "00000000-0000-4000-8000-000000000701";

function session(): AuthSession {
  return {
    access_token: "route-integration-token",
    org_id: "org-a",
    user_id: "user-a",
    roles: ["ADMIN"],
    feature_grants: ["object.view"],
    branches: ["branch-a"],
    client_session_incarnation: "session-a",
  };
}

function canonicalApi(): { api: ConsoleApiClient; requestedPaths: string[] } {
  const requestedPaths: string[] = [];
  const GET = vi.fn((path: string) => {
    requestedPaths.push(path);
    if (path === "/api/v1/object-types") {
      return Promise.resolve({
        data: [{ kind: "widget", code_prefix: "WD-", description: "dynamic widget", status: "active", active_count: 1 }],
        response: new Response(),
      });
    }
    if (path === "/api/v1/ontology/object-types/{key}") {
      return Promise.resolve({
        data: {
          object_type: {
            id: typeVersionId,
            stable_key: "widget",
            title: "Real Widget",
            backing_kind: "instance",
            schema_version: 7,
            lifecycle_state: "published",
            key_write_revision: 7,
            key_write_etag: '"widget-r7"',
          },
          title_property_key: "label",
          backing_table: null,
          primary_key_property: null,
          properties: [{
            id: "p1",
            key: "label",
            title: "Governed label",
            field_type: "text",
            config: {},
            backing_column: null,
            required: true,
            in_property_policy: false,
          }],
          links: [],
          actions: [],
          analytics: [],
        },
        response: new Response(),
      });
    }
    if (path === "/api/v1/ontology/instances") {
      return Promise.resolve({
        data: [{
          instance: {
            id: "00000000-0000-4000-8000-000000000703",
            object_type_id: typeVersionId,
            title: "Real widget instance",
            current_revision_id: "r1",
            lifecycle_state: "active",
          },
          revision: {
            id: "r1",
            instance_id: "00000000-0000-4000-8000-000000000703",
            version: 1,
            attributes: { label: "Real governed value" },
            valid_from: "2026-07-25T00:00:00Z",
            valid_to: null,
            action_type_id: null,
            actor: null,
            reason: null,
            prev_hash: "0".repeat(64),
            row_hash: "a".repeat(64),
          },
        }],
        response: new Response(),
      });
    }
    throw new Error(`unexpected GET path: ${path}`);
  });
  return { api: { GET } as unknown as ConsoleApiClient, requestedPaths };
}

afterEach(() => {
  resetObjectTypeRegistry();
});

describe("ConsoleModuleRoute canonical dynamic integration", () => {
  it("renders a fresh backend-registered kind with its canonical schema, version, and values", async () => {
    const { api, requestedPaths } = canonicalApi();
    const currentSession = session();

    render(
      <MemoryRouter initialEntries={["/modules?screen=widget"]}>
        <AuthTestProvider session={currentSession} overrides={{ api }}>
          <ConsoleModuleRoute />
        </AuthTestProvider>
      </MemoryRouter>,
    );

    expect(await screen.findByRole("columnheader", { name: "Governed label" })).toBeVisible();
    expect(screen.getAllByText("Real governed value").length).toBeGreaterThan(0);
    expect(getObjectType("widget", ontologyWorkspaceAuthorityKey(currentSession, undefined))).toMatchObject({
      canonical: { id: typeVersionId, schemaVersion: 7, schemaLifecycle: "published" },
    });
    expect(screen.queryByText("OT-WIDGET")).not.toBeInTheDocument();
    expect(requestedPaths).toEqual([
      "/api/v1/object-types",
      "/api/v1/ontology/object-types/{key}",
      "/api/v1/ontology/instances",
    ]);
  });
});
