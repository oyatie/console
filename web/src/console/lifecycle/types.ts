// Lifecycle payload types — sourced from the ONE generated OpenAPI client
// (`@console/api-client-ts`, regenerated from backend/openapi/openapi.yaml)
// so the card can never drift from the real REST contract (#211 BE-LC).

import type { components } from "@console/api-client-ts";

export type Lifecycle = components["schemas"]["ObjectLifecycle"];
export type LifecycleTransition = components["schemas"]["ObjectLifecycleTransition"];
