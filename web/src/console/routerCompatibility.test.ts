import { readdirSync, readFileSync } from "node:fs";
import { extname, join, resolve } from "node:path";

import { describe, expect, it } from "vitest";

const routerPackage = ["react", "router"].join("-");
const retiredRouterPackage = `${routerPackage}-dom`;
const supportedSourceExtensions = new Set([".ts", ".tsx", ".js", ".jsx"]);
const webSourceRoot = resolve(process.cwd(), "src");

function sourceFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return sourceFiles(path);
    return supportedSourceExtensions.has(extname(entry.name)) ? [path] : [];
  });
}

describe("PR490 router compatibility", () => {
  it("keeps every web source and test surface on the React Router v8 package", () => {
    const legacyImports = sourceFiles(webSourceRoot).filter((path) =>
      readFileSync(path, "utf8").includes(retiredRouterPackage),
    );
    expect(legacyImports).toEqual([]);
  });
});
