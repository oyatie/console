#!/usr/bin/env node
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, resolve, relative, isAbsolute } from "node:path";

const root = resolve(process.argv[2] ?? process.cwd());
const markdown = [];
function walk(dir) {
  for (const name of readdirSync(dir)) {
    if (
      name === ".git"
      || name === "buck-out"
      || name === "node_modules"
      || name === "target"
      || name === "third-party"
    ) continue;
    const path = join(dir, name);
    const st = statSync(path);
    if (st.isDirectory()) walk(path);
    else if (/\.md$/i.test(name)) markdown.push(path);
  }
}

function localTarget(raw) {
  const value = raw.trim().replace(/^<|>$/g, "");
  if (!value || value.startsWith("#") || /^[a-z][a-z0-9+.-]*:/i.test(value) || value.startsWith("//")) return null;
  const pathPart = value.split("#", 1)[0].split("?", 1)[0];
  if (!pathPart) return null;
  return decodeURIComponent(pathPart);
}

function withoutInlineCode(line) {
  let visible = "";
  let cursor = 0;
  while (cursor < line.length) {
    if (line[cursor] !== "`") {
      visible += line[cursor];
      cursor += 1;
      continue;
    }

    let endOfOpeningRun = cursor + 1;
    while (line[endOfOpeningRun] === "`") endOfOpeningRun += 1;
    const delimiter = line.slice(cursor, endOfOpeningRun);
    const closingRun = line.indexOf(delimiter, endOfOpeningRun);
    if (closingRun === -1) {
      visible += delimiter;
      cursor = endOfOpeningRun;
      continue;
    }
    cursor = closingRun + delimiter.length;
  }
  return visible;
}

walk(root);
const failures = [];
for (const file of markdown) {
  const lines = readFileSync(file, "utf8").split(/\r?\n/);
  let fenced = false;
  for (let lineNo = 0; lineNo < lines.length; lineNo++) {
    const line = lines[lineNo];
    if (/^\s*(```|~~~)/.test(line)) { fenced = !fenced; continue; }
    if (fenced) continue;
    const visibleLine = withoutInlineCode(line);
    // Markdown inline links/images; reference-style definitions are handled too.
    const links = [...visibleLine.matchAll(/!?\[[^\]]*\]\(\s*([^\s)]+|<[^>]+>)(?:\s+[^)]*)?\s*\)/g)];
    const reference = visibleLine.match(/^\s*\[[^\]]+\]:\s*(\S+)/);
    if (reference) links.push({ 1: reference[1] });
    for (const match of links) {
      const target = localTarget(match[1]);
      if (!target) continue;
      const candidate = resolve(file, "..", target);
      if (isAbsolute(candidate) && !candidate.startsWith(root + "/") && candidate !== root) {
        failures.push(`${relative(root, file)}:${lineNo + 1}: link escapes repository: ${target}`);
      } else if (!existsSync(candidate)) {
        failures.push(`${relative(root, file)}:${lineNo + 1}: missing target: ${target}`);
      }
    }
  }
}
if (failures.length) {
  console.error(failures.join("\n"));
  process.exitCode = 1;
} else {
  console.log(`doc links OK (${markdown.length} markdown files)`);
}
