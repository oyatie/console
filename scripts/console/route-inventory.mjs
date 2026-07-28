import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

export const CONSOLE_NAV_SOURCE = 'web/src/console/shell/nav.ts';
export const CONSOLE_REGISTRY_SOURCE = 'web/src/console/screens/registry.ts';

// The single legitimate "no facts" value. `route_source_present: false` is the
// only thing that lets a consumer distinguish "the console has no frontend, so
// it presents no routes" from "extraction produced nothing". Consumers must
// treat a missing/false flag as absent source and refuse to corroborate any
// route claim against it.
export const ABSENT_CONSOLE_ROUTE_FACTS = Object.freeze({ route_source_present: false, facts: Object.freeze({}) });

function literals(text, declaration) {
  const match = text.match(new RegExp(`(?:export\\s+)?const\\s+${declaration}[\\s\\S]*?=\\s*\\[([\\s\\S]*?)\\](?:\\s+as const)?`));
  if (!match) throw new Error(`missing ${declaration}`);
  return [...match[1].matchAll(/"([A-Za-z][A-Za-z0-9]*)"/g)].map((entry) => entry[1]);
}
export function extractConsoleRouteFactsFromTexts(navText, registryText) {
  const mounted = literals(navText, 'MOUNTED_SCREEN_KEYS');
  const exposed = literals(navText, 'EXPOSED_SCREEN_KEYS');
  const nav = [...navText.matchAll(/screen:\s*"([A-Za-z][A-Za-z0-9]*)"/g)].map((entry) => entry[1]);
  const bodyBlock = registryText.match(/SCREEN_REGISTRY[^=]*=\s*\{([\s\S]*?)\n\};/);
  if (!bodyBlock) throw new Error('missing SCREEN_REGISTRY');
  const bodies = [...bodyBlock[1].matchAll(/^\s*([A-Za-z][A-Za-z0-9]*):/gm)].map((entry) => entry[1]);
  const all = new Set([...mounted, ...exposed, ...nav, ...bodies]);
  const facts = Object.fromEntries([...all].sort().map((key) => [key, { source_mounted: mounted.includes(key), production_exposed: exposed.includes(key), registry_body_present: bodies.includes(key), nav_declared: nav.includes(key) }]));
  return { route_source_present: true, facts, mounted, exposed, nav, bodies };
}

/**
 * Absence of BOTH route sources is the only tolerated failure: the 2026-07-28
 * clean-slate pivot deleted the frontend, and a console with no frontend
 * presents no routes. Every other failure — a renamed constant, a malformed
 * SCREEN_REGISTRY, an unreadable file, a half-landed frontend — propagates.
 * Swallowing those would report "no routes" for a console that has them.
 */
export function extractConsoleRouteFacts(repoRoot) {
  const navPath = path.join(repoRoot, CONSOLE_NAV_SOURCE);
  const registryPath = path.join(repoRoot, CONSOLE_REGISTRY_SOURCE);
  const navPresent = existsSync(navPath);
  const registryPresent = existsSync(registryPath);
  if (navPresent !== registryPresent) throw new Error(`console route source is partially present: ${navPresent ? CONSOLE_REGISTRY_SOURCE : CONSOLE_NAV_SOURCE} is missing`);
  if (!navPresent) return ABSENT_CONSOLE_ROUTE_FACTS;
  return extractConsoleRouteFactsFromTexts(readFileSync(navPath, 'utf8'), readFileSync(registryPath, 'utf8'));
}
