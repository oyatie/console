import { useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router";

import { useAuth } from "../../context/auth";
import { ApprovalCompose } from "../appr/ApprovalCompose";
import { MESSENGER_ACTIONS, MessengerConsoleScreen } from "../messenger";
import { loadObjectTypeRegistryForAuthority } from "../ontology/typeRegistrySource";
import { ontologyWorkspaceAuthorityKey } from "../ontology/useOntologyRevisionCommitQueue";
import { PolicyGateProvider } from "../policy";
import { GenericModuleScreen } from "./GenericModuleScreen";
import { getModuleScreen, MOD_SCREENS } from "./moduleScreens";

const MODULE_READ_ROLES = new Set([
  "SUPER_ADMIN",
  "ADMIN",
  "EXECUTIVE",
  "MECHANIC",
  "RECEPTIONIST",
]);

const MESSENGER_ROLES = new Set([
  "SUPER_ADMIN",
  "ADMIN",
  "EXECUTIVE",
  "MECHANIC",
  "RECEPTIONIST",
  "MEMBER",
]);

const MESSENGER_ACTION_SET = new Set<string>(Object.values(MESSENGER_ACTIONS));
const APPR_ROLES = new Set(["SUPER_ADMIN", "ADMIN", "EXECUTIVE", "MECHANIC", "RECEPTIONIST", "MEMBER"]);

function sessionCanReadModule(roles: readonly string[] | undefined): boolean {
  return roles?.some((role) => MODULE_READ_ROLES.has(role)) ?? false;
}

function RoutedModuleScreen({
  api,
  config,
  authorityKey,
  featureGrants,
  roles,
}: {
  api: ReturnType<typeof useAuth>["api"];
  config: ReturnType<typeof getModuleScreen>;
  authorityKey: string | undefined;
  featureGrants: readonly string[];
  roles: readonly string[] | undefined;
}) {
  const gate = useMemo(
    () => {
      return {
        can: (action: string) => {
          if (featureGrants.includes(action)) return true;
          if (action === config.policy.read) return sessionCanReadModule(roles);
          return false;
        },
      };
    },
    [config.policy.read, featureGrants, roles],
  );

  return (
    <PolicyGateProvider gate={gate}>
      <GenericModuleScreen api={api} config={config} authorityKey={authorityKey} />
    </PolicyGateProvider>
  );
}

function DynamicModuleScreen({
  api,
  screen,
  authorityKey,
  featureGrants,
  roles,
}: {
  api: ReturnType<typeof useAuth>["api"];
  screen: string;
  authorityKey: string | undefined;
  featureGrants: readonly string[];
  roles: readonly string[] | undefined;
}) {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    if (!authorityKey) return;
    const controller = new AbortController();
    void loadObjectTypeRegistryForAuthority(api, authorityKey, controller.signal)
      .then(() => {
        if (!controller.signal.aborted) setReady(true);
      })
      .catch(() => {
        // Discovery failure is intentionally represented by the fail-closed
        // empty route below rather than an alternate module surface.
      });
    return () => {
      controller.abort();
    };
  }, [api, authorityKey]);

  // Until the exact authority's registry proves that this screen is a dynamic
  // module, render nothing. In particular, never substitute finance for an
  // unavailable or stale discovery response.
  if (!ready) return null;
  const config = getModuleScreen(screen);
  if (config.id !== screen) return null;
  return (
    <RoutedModuleScreen
      api={api}
      config={config}
      authorityKey={authorityKey}
      featureGrants={featureGrants}
      roles={roles}
    />
  );
}

export function ConsoleModuleRoute() {
  const { api, session, viewAs } = useAuth();
  const [searchParams] = useSearchParams();
  const screen = searchParams.get("screen") ?? "finance";
  const featureGrants = useMemo(() => session?.feature_grants ?? [], [session?.feature_grants]);
  const roles = session?.roles;
  const authorityKey = ontologyWorkspaceAuthorityKey(session, viewAs);

  const messengerGate = useMemo(
    () => {
      return {
        can: (action: string) => {
          if (featureGrants.includes(action)) return true;
          if (MESSENGER_ACTION_SET.has(action)) {
            return roles?.some((role) => MESSENGER_ROLES.has(role)) ?? false;
          }
          return false;
        },
      };
    },
    [featureGrants, roles],
  );

  const apprGate = useMemo(
    () => {
      return {
        can: (action: string) => {
          if (featureGrants.includes(action)) return true;
          if (action.startsWith("appr.")) {
            return roles?.some((role) => APPR_ROLES.has(role)) ?? false;
          }
          return false;
        },
      };
    },
    [featureGrants, roles],
  );

  if (screen === "msgr") {
    return (
      <PolicyGateProvider gate={messengerGate}>
        <MessengerConsoleScreen
          accessToken={session?.access_token}
          branchId={session?.branches?.[0]}
          currentUserId={session?.user_id}
        />
      </PolicyGateProvider>
    );
  }

  if (screen === "appr") {
    return (
      <PolicyGateProvider gate={apprGate}>
        <ApprovalCompose bearerToken={session?.access_token} currentUserId={session?.user_id} />
      </PolicyGateProvider>
    );
  }

  if (Object.prototype.hasOwnProperty.call(MOD_SCREENS, screen)) {
    const config = getModuleScreen(screen);
    return (
      <RoutedModuleScreen
        api={api}
        config={config}
        authorityKey={authorityKey}
        featureGrants={featureGrants}
        roles={roles}
      />
    );
  }

  return (
    <DynamicModuleScreen
      key={`${screen}:${authorityKey ?? "untrusted"}`}
      api={api}
      screen={screen}
      authorityKey={authorityKey}
      featureGrants={featureGrants}
      roles={roles}
    />
  );
}
