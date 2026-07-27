import { readFileSync } from 'node:fs';
import path from 'node:path';

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
  return { facts, mounted, exposed, nav, bodies };
}
export function extractConsoleRouteFacts(repoRoot) {
  return extractConsoleRouteFactsFromTexts(readFileSync(path.join(repoRoot, 'web/src/console/shell/nav.ts'), 'utf8'), readFileSync(path.join(repoRoot, 'web/src/console/screens/registry.ts'), 'utf8'));
}
