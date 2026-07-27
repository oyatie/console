#!/usr/bin/env node
import { existsSync, lstatSync, readFileSync, realpathSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function isSafeWorkspacePath(path) {
  return (
    typeof path === "string" &&
    path.length > 0 &&
    !path.startsWith("/") &&
    !path.split(/[\\/]+/).includes("..") &&
    !/[*!?{}[\]]/.test(path)
  );
}

function readJson(root, path, failures) {
  try {
    return JSON.parse(readFileSync(resolve(root, path), "utf8"));
  } catch (error) {
    failures.push(`${path} must be valid JSON: ${error.message}`);
    return null;
  }
}

function inspectLocalDirectory(root, path) {
  const result = {
    exists: false,
    hasSymbolicLink: false,
    isDirectory: false,
    isWithinRoot: false,
  };
  let current = resolve(root);

  try {
    for (const component of path.split(/[\\/]+/)) {
      current = resolve(current, component);
      const metadata = lstatSync(current, { throwIfNoEntry: false });
      if (!metadata) return result;
      if (metadata.isSymbolicLink()) {
        result.exists = true;
        result.hasSymbolicLink = true;
        return result;
      }
      if (!metadata.isDirectory()) {
        result.exists = true;
        return result;
      }
    }

    result.exists = true;
    result.isDirectory = true;
    const resolvedFromRoot = relative(realpathSync(root), realpathSync(current));
    result.isWithinRoot =
      resolvedFromRoot === "" ||
      (
        resolvedFromRoot !== ".." &&
        !resolvedFromRoot.startsWith(`..${sep}`) &&
        !isAbsolute(resolvedFromRoot)
      );
    return result;
  } catch {
    return result;
  }
}

function collectFileOverrideSpecs(value, specs = []) {
  if (typeof value === "string") {
    if (value.startsWith("file:")) {
      specs.push(value.slice("file:".length).replaceAll("\\", "/"));
    }
    return specs;
  }
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return specs;
  }
  for (const nested of Object.values(value)) {
    collectFileOverrideSpecs(nested, specs);
  }
  return specs;
}

function findLocalOverrideTargets(packageJson, packages, declared) {
  const specs = collectFileOverrideSpecs(packageJson.overrides);
  return new Set(
    Object.keys(packages).filter((path) => {
      if (path === "" || path.includes("node_modules/") || declared.has(path)) {
        return false;
      }
      return (
        isSafeWorkspacePath(path) &&
        specs.some((spec) => spec === path || spec.endsWith(`/${path}`))
      );
    }),
  );
}

function checkLockfileWorkspaceTopology(packageJson, packageLock, failures) {
  const declaredWorkspaces = packageJson.workspaces;
  const packages = packageLock?.packages;
  if (!packages || typeof packages !== "object" || Array.isArray(packages)) {
    failures.push("package-lock.json must contain a packages object");
    return;
  }

  const lockRootWorkspaces = packages[""]?.workspaces;
  if (!Array.isArray(lockRootWorkspaces)) {
    failures.push("package-lock.json root package must declare a workspaces array");
  } else if (JSON.stringify(lockRootWorkspaces) !== JSON.stringify(declaredWorkspaces)) {
    failures.push("package-lock.json root workspaces must exactly match package.json workspaces");
  }

  const declared = new Set(declaredWorkspaces);
  const localOverrideTargets = findLocalOverrideTargets(
    packageJson,
    packages,
    declared,
  );
  const allowedLocalPackages = new Set([...declared, ...localOverrideTargets]);
  for (const [path, metadata] of Object.entries(packages)) {
    if (path === "") continue;

    if (!path.includes("node_modules/")) {
      if (!allowedLocalPackages.has(path)) {
        failures.push(`package-lock.json contains stale workspace package entry ${JSON.stringify(path)}`);
      }
      continue;
    }

    if (
      metadata?.link === true &&
      typeof metadata.resolved === "string" &&
      !allowedLocalPackages.has(metadata.resolved)
    ) {
      failures.push(`package-lock.json contains stale workspace link ${JSON.stringify(path)} -> ${JSON.stringify(metadata.resolved)}`);
    }
  }
}

function checkLocalOverridePackages(root, packageJson, packageLock, failures) {
  const packages = packageLock?.packages;
  if (!packages || typeof packages !== "object" || Array.isArray(packages)) {
    return;
  }
  const declared = new Set(packageJson.workspaces);
  for (const path of findLocalOverrideTargets(packageJson, packages, declared)) {
    const directory = inspectLocalDirectory(root, path);
    if (
      !directory.exists ||
      directory.hasSymbolicLink ||
      !directory.isDirectory ||
      !directory.isWithinRoot
    ) {
      failures.push(
        `package.json local override target ${JSON.stringify(path)} must resolve to an existing non-symlink directory`,
      );
      continue;
    }
    const manifest = readJson(root, `${path}/package.json`, failures);
    if (manifest && manifest.private !== true) {
      failures.push(
        `package.json local override target ${JSON.stringify(path)} must be private`,
      );
    }
  }
}

export function evaluateRootWorkspaces(root) {
  const failures = [];
  const packageJson = readJson(root, "package.json", failures);
  const packageLock = readJson(root, "package-lock.json", failures);
  if (!packageJson) return { failures };

  if (!Array.isArray(packageJson.workspaces) || packageJson.workspaces.length === 0) {
    failures.push("package.json must declare a non-empty workspaces array");
    return { failures };
  }

  for (const workspace of packageJson.workspaces) {
    if (!isSafeWorkspacePath(workspace)) {
      failures.push(`package.json workspace ${JSON.stringify(workspace)} must be a relative literal path without traversal or globs`);
      continue;
    }

    const workspaceDirectory = inspectLocalDirectory(root, workspace);
    if (!workspaceDirectory.exists) {
      failures.push(`package.json workspace ${JSON.stringify(workspace)} must resolve to an existing directory`);
      continue;
    }
    if (workspaceDirectory.hasSymbolicLink) {
      failures.push(`package.json workspace ${JSON.stringify(workspace)} must not be a symbolic link`);
      continue;
    }
    if (!workspaceDirectory.isDirectory || !workspaceDirectory.isWithinRoot) {
      failures.push(`package.json workspace ${JSON.stringify(workspace)} must resolve to an existing directory`);
      continue;
    }
    if (!existsSync(resolve(root, workspace, "package.json"))) {
      failures.push(`package.json workspace ${JSON.stringify(workspace)} must contain package.json`);
    }
  }

  if (packageLock) {
    checkLockfileWorkspaceTopology(packageJson, packageLock, failures);
    checkLocalOverridePackages(root, packageJson, packageLock, failures);
  }

  return { failures };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const { failures } = evaluateRootWorkspaces(root);

  if (failures.length > 0) {
    console.error("Root npm workspace integrity check failed:");
    for (const failure of failures) console.error(`- ${failure}`);
    process.exit(1);
  }

  console.log("Root npm workspace integrity check passed.");
}
