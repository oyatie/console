// Minimal GitHub Actions + shell command extraction for reachability gates.
//
// This intentionally does not pretend YAML text is executable merely because it
// contains a command-shaped substring. Only `run:` bodies from jobs/steps that can
// gate CI are admitted, shell comments and continuations are resolved, and each
// returned entry is one actual command segment. A line such as
// `echo cargo test ...`, or a step guarded by literal `if: false`, therefore cannot
// manufacture test reachability.

import { accessSync, constants, realpathSync, statSync } from "node:fs";

function jobBlocks(workflow) {
  // Match the top-level `jobs:` key even when it carries a trailing YAML comment
  // (`jobs: # scanner bypass`), which is still a valid key.
  const jobsStart = workflow.search(/^jobs:/m);
  if (jobsStart < 0) return [];
  const jobsLineEnd = workflow.indexOf("\n", jobsStart);
  if (jobsLineEnd < 0) return [];
  const jobs = workflow.slice(jobsLineEnd + 1);
  return [...jobs.matchAll(/^  ([A-Za-z0-9_-]+):\n([\s\S]*?)(?=^  [A-Za-z0-9_-]+:|(?![\s\S]))/gm)]
    .map(([, name, block]) => ({ name, block }));
}

function stepBlocks(block) {
  // `steps:` may carry a trailing YAML comment (`steps: # comment`) and still be valid.
  const steps = block.match(/^    steps:\s*(?:#.*)?\n([\s\S]*)$/m)?.[1] ?? "";
  return steps.split(/^      - /m).slice(1);
}

const BLOCK_SCALAR_INDICATOR = /^[|>](?:[+-][1-9]?|[1-9][+-]?)?$/;

/**
 * YAML folded-scalar semantics: a line break between two non-empty lines folds into a
 * single space, and a break adjacent to an empty line stays a paragraph break. A folded
 * body must be reassembled BEFORE tokenizing, or a wrapper and its target that sit on
 * adjacent lines are misread as two unrelated commands.
 */
function foldBlockScalar(body) {
  const folded = [];
  let current = "";
  for (const line of body) {
    if (line === "") {
      if (current !== "") {
        folded.push(current);
        current = "";
      }
      folded.push("");
    } else {
      current += (current === "" ? "" : " ") + line;
    }
  }
  if (current !== "") folded.push(current);
  return folded.join("\n");
}

/** Strip a trailing ` # ...` YAML comment from a single-line scalar value. */
function stripYamlInlineComment(value) {
  const index = value.search(/\s#/);
  return (index < 0 ? value : value.slice(0, index)).trim();
}

/** Decode the YAML double-quoted scalar escape sequences into their characters. */
function decodeYamlEscapes(value) {
  const singles = { "0": "\0", a: "\x07", b: "\b", e: "\x1b", f: "\f", n: "\n", r: "\r", t: "\t", v: "\v", " ": " ", "\\": "\\", '"': '"', "/": "/" };
  return value
    .replace(/\\U([0-9A-Fa-f]{8})/g, (_, h) => String.fromCodePoint(parseInt(h, 16)))
    .replace(/\\u([0-9A-Fa-f]{4})/g, (_, h) => String.fromCharCode(parseInt(h, 16)))
    .replace(/\\x([0-9A-Fa-f]{2})/g, (_, h) => String.fromCharCode(parseInt(h, 16)))
    .replace(/\\([0abefnrtv \\"\/])/g, (_, c) => (c in singles ? singles[c] : c));
}

/** Strip one pair of matching YAML flow scalar quotes and decode double-quoted escapes. */
function unquoteYamlScalar(value) {
  if (value.length < 2) return value;
  const first = value[0];
  const last = value[value.length - 1];
  if (first === '"' && last === '"') return decodeYamlEscapes(value.slice(1, -1));
  if (first === "'" && last === "'") return value.slice(1, -1).replace(/''/g, "'");
  return value;
}

function runScript(step) {
  const lines = step.split(/\r?\n/);
  const runLineIndex = lines.findIndex((line) => /^        run:/.test(line));
  if (runLineIndex < 0) return "";
  const value = unquoteYamlScalar(stripYamlInlineComment(lines[runLineIndex].slice("        run:".length).trim()));
  if (value === "") return "";
  if (!BLOCK_SCALAR_INDICATOR.test(value)) return value;
  const folded = value.startsWith(">");
  const body = [];
  for (const line of lines.slice(runLineIndex + 1)) {
    // YAML permits physically empty lines inside a block scalar. Do not let one truncate
    // the executable surface: the command after a visual paragraph break still runs.
    // `|`, `|-`, `|+`, `>`, `>-`, and `>+` are all block scalars; chomping only affects
    // trailing newlines, and folding only joins non-empty lines, neither of which hides
    // the argv0 of a wrapper. A trailing `# comment` on the `run:` line is a YAML
    // comment, not part of the indicator.
    if (line === "") {
      body.push("");
    } else if (line.startsWith("          ")) {
      body.push(line.slice(10));
    } else {
      break;
    }
  }
  return folded ? foldBlockScalar(body) : body.join("\n");
}

/**
 * `defaults.run.shell` at the given indentation (`defaults:` at `indent`, `run:` at
 * `indent + 2`, keys at `indent + 4`), or null when the defaults run block names no shell.
 */
function defaultsRunShell(block, indent) {
  const lines = block.split(/\r?\n/);
  for (let i = 0; i < lines.length; i += 1) {
    if (lines[i] !== `${" ".repeat(indent)}defaults:`) continue;
    for (let j = i + 1; j < lines.length; j += 1) {
      if (lines[j] === `${" ".repeat(indent + 2)}run:`) {
        for (let k = j + 1; k < lines.length; k += 1) {
          const key = lines[k];
          if (key.startsWith(`${" ".repeat(indent + 4)}shell:`)) {
            return key.slice(`${" ".repeat(indent + 4)}shell:`.length).trim();
          }
          if (key.trim() === "") continue;
          if (!key.startsWith(`${" ".repeat(indent + 4)}`)) return null;
        }
        return null;
      }
      if (lines[j].trim() === "") continue;
      if (!lines[j].startsWith(`${" ".repeat(indent + 2)}`)) break;
    }
  }
  return null;
}

function yamlValue(block, key, indentation) {
  return block.match(new RegExp(`^${" ".repeat(indentation)}${key}: ([^\\n]+)$`, "m"))?.[1]?.trim();
}

function literalFalse(value) {
  if (!value) return false;
  const normalized = value.replace(/^\$\{\{\s*|\s*\}\}$/g, "").trim().toLowerCase();
  return normalized === "false" || normalized === "0" || normalized === "null";
}

function canGate(block, indentation) {
  if (literalFalse(yamlValue(block, "if", indentation))) return false;
  const continueOnError = yamlValue(block, "continue-on-error", indentation);
  return continueOnError === undefined || literalFalse(continueOnError);
}

function stripShellComment(line) {
  let quote = null;
  let escaped = false;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (escaped) {
      escaped = false;
    } else if (character === "\\") {
      escaped = true;
    } else if (quote) {
      if (character === quote) quote = null;
    } else if (character === "'" || character === '"') {
      quote = character;
    } else if (character === "#" && (index === 0 || /\s/.test(line[index - 1]))) {
      return line.slice(0, index);
    }
  }
  return line;
}

/** Find a shell here-document operator `<<`/`<<-` that sits OUTSIDE quotes, or null. */
function hereDocStart(line) {
  let quote = null;
  let escaped = false;
  for (let index = 0; index < line.length - 1; index += 1) {
    const character = line[index];
    if (escaped) {
      escaped = false;
    } else if (character === "\\") {
      escaped = true;
    } else if (quote) {
      if (character === quote) quote = null;
    } else if (character === "'" || character === '"') {
      quote = character;
    } else if (character === "<" && line[index + 1] === "<" && line[index + 2] !== "<") {
      const rest = line.slice(index + 2);
      const match = /^-?\s*['"]?([A-Za-z_][A-Za-z0-9_]*)['"]?/.exec(rest);
      if (match) return { index, delimiter: match[1], end: index + 2 + match[0].length };
    }
  }
  return null;
}

export function shellCommandTokens(script) {
  const commands = [];
  let command = "";
  let hereDocDelimiter = null;
  for (const physicalLine of script.split(/\r?\n/)) {
    // Skip here-document data: `cmd <<'EOF'` ... `EOF` writes text to a command's stdin;
    // that data is not executable and must not be classified as commands. A `<<` inside
    // quotes (e.g. the GITHUB_ENV `echo "VAR<<EOF"` marker) is data, not a here-doc.
    if (hereDocDelimiter !== null) {
      if (physicalLine.trim() === hereDocDelimiter) hereDocDelimiter = null;
      continue;
    }
    const line = stripShellComment(physicalLine.trim());
    if (!line) continue;
    const hereDoc = hereDocStart(line);
    if (hereDoc) {
      hereDocDelimiter = hereDoc.delimiter;
      // The redirection `<<'EOF'` feeds data to the command on its left, but operators
      // and commands on its right (`cat <<'EOF' && cosign ...`) still execute after the
      // here-document is consumed, so preserve them.
      const remainder = `${line.slice(0, hereDoc.index)} ${line.slice(hereDoc.end)}`.trim();
      if (remainder) {
        commands.push((command ? `${command} ` : "") + remainder);
        command = "";
      }
      continue;
    }
    const continued = /\\\s*$/.test(line);
    command += (command ? " " : "") + (continued ? line.replace(/\\\s*$/, "") : line);
    if (!continued) {
      commands.push(command);
      command = "";
    }
  }
  if (command) commands.push(`${command}\\`);

  return commands.map((source) => {
    const tokens = [];
    const operators = [];
    let token = "";
    let quote = null;
    let escaped = false;
    const flush = () => {
      if (token) tokens.push(token);
      token = "";
    };
    for (let index = 0; index < source.length; index += 1) {
      const character = source[index];
      if (escaped) {
        token += character;
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (quote) {
        if (character === quote) quote = null;
        else token += character;
      } else if (character === "'" || character === '"') {
        quote = character;
      } else if (/\s/.test(character)) {
        flush();
      } else if (";|&".includes(character)) {
        flush();
        operators.push(tokens.length);
        if ((character === "|" || character === "&") && source[index + 1] === character) {
          tokens.push(`${character}${character}`);
          index += 1;
        } else {
          tokens.push(character);
        }
      } else {
        token += character;
      }
    }
    const malformed = quote !== null || escaped;
    if (escaped) token += "\\";
    flush();
    return { source, tokens, malformed, operators };
  });
}

function commandSegments(surface, literalOperatorsOnly = false) {
  const segments = [];
  let segment = [];
  let conditional = false;
  let operator = ";";
  for (const [index, token] of [...surface.tokens, ";"].entries()) {
    const separator = literalOperatorsOnly ? index === surface.tokens.length || surface.operators.includes(index)
      : [";", "|", "&", "||", "&&"].includes(token);
    if (separator) {
      if (segment.length > 0) segments.push({ tokens: segment, conditional, operator });
      segment = [];
      // `;` is a sequence operator (the next segment runs regardless); `&&`, `||`,
      // `|`, and `&` make the next segment conditional on the previous one.
      conditional = token !== ";";
      operator = token;
    } else {
      segment.push(token);
    }
  }
  return segments;
}

/** Bounded shell reachability; audit callers retain the unfiltered token surface. */
export function reachableShellCommands(script) {
  const parts = shellCommandTokens(script).flatMap(surface => {
    // Pipelines/background jobs need a separate execution contract; quoted operators are argv data.
    if (surface.operators.some(index => ["|", "&"].includes(surface.tokens[index]))) return [{
      tokens: surface.tokens, malformed: true, operator: ";", conditional: false, controlFlow: true,
    }];
    return commandSegments(surface, true)
      .map(part => ({ ...part, malformed: surface.malformed,
        controlFlow: surface.operators.length > 0 }));
  });
  const functions = new Map();
  const opener = tokens => tokens[0] === "{" || /^(?:function\b|[A-Za-z_][A-Za-z0-9_]*\(\))/.test(tokens[0] ?? "")
    || /^[A-Za-z_][A-Za-z0-9_]*$/.test(tokens[0] ?? "") && ["()", "("].includes(tokens[1]) && tokens.includes("{");
  function closing(start, end, open, close) {
    let depth = 1;
    let otherwise = -1;
    for (let index = start + 1; index < end; index += 1) {
      const head = parts[index].tokens[0];
      if (open(parts[index].tokens)) depth += 1;
      if (head === close && --depth === 0) return { end: index, otherwise };
      if (depth === 1 && head === "elif") otherwise = -2;
      else if (depth === 1 && head === "else" && otherwise !== -2) otherwise = index;
    }
    return null;
  }
  function project(start, end, stack = []) {
    const commands = [];
    let status = null;
    for (let index = start; index < end; index += 1) {
      const part = parts[index];
      const { tokens, operator } = part;
      const head = tokens[0];
      const skipped = (operator === "&&" && status === false) || (operator === "||" && status === true);
      const certain = !["&&", "||", "|", "&"].includes(operator)
        || (operator === "&&" && status === true) || (operator === "||" && status === false);
      if (part.malformed) return { commands, status: null };
      if (opener(tokens)) {
        const block = closing(index, end, opener, "}");
        if (!block) return { commands, status: null };
        const name = tokens.length === 2 && tokens[1] === "{" && tokens[0].match(/^([A-Za-z_][A-Za-z0-9_]*)\(\)$/)?.[1];
        if (!skipped && name) functions.set(name, { start: index + 1, end: block.end });
        index = block.end;
        if (!skipped) status = true;
        continue;
      }
      if (head === "if") {
        const block = closing(index, end, inner => inner[0] === "if", "fi");
        if (!block) return { commands, status: null };
        const literal = tokens.length === 2 && ["true", "false"].includes(tokens[1]);
        const then = parts[index + 1]?.tokens;
        if (!skipped && literal && then?.length === 1 && then[0] === "then"
            && block.otherwise !== -2) {
          const branchStart = tokens[1] === "true" ? index + 2 : block.otherwise < 0 ? block.end : block.otherwise + 1;
          const branchEnd = tokens[1] === "true" && block.otherwise >= 0 ? block.otherwise : block.end;
          const branch = project(branchStart, branchEnd, stack);
          commands.push(...branch.commands.map(command => ({ ...command, conditional: command.conditional || !certain })));
          status = certain ? branch.status : null;
          if (branch.stop && certain) return { commands, status, stop: branch.stop };
        } else if (!skipped) status = null;
        index = block.end;
        continue;
      }
      if (["case", "for", "while", "until", "select"].includes(head)) {
        const close = head === "case" ? "esac" : "done";
        const block = closing(index, end, inner => close === "esac" ? inner[0] === "case" : ["for", "while", "until", "select"].includes(inner[0]), close);
        if (!block) return { commands, status: null };
        index = block.end;
        if (!skipped) status = null;
        continue;
      }
      if (skipped) continue;
      const direct = directExecutable(tokens);
      if (direct.malformed || ["then", "else", "elif", "fi", "esac", "do", "done", "}", "(", ")", "!", "eval", "exec", "source", "."].includes(direct.tokens[0])) return { commands, status: null };
      const body = functions.get(direct.tokens[0]);
      if (body) {
        if (stack.length >= 12 || stack.includes(direct.tokens[0]) || direct.tokens.length !== 1) return { commands, status: null };
        const called = project(body.start, body.end, [...stack, direct.tokens[0]]);
        commands.push(...called.commands.map(command => ({ ...command, conditional: command.conditional || !certain })));
        status = certain ? called.status : null;
        if (called.stop === "exit" && certain) return { commands, status, stop: "exit" };
      } else {
        commands.push({ ...part, conditional: part.conditional || !certain });
        status = direct.tokens[0] === "true" ? true : direct.tokens[0] === "false" ? false : null;
        const truthPaths = ["/usr/bin/true", "/usr/bin/false", "/bin/true", "/bin/false"];
        if (truthPaths.includes(direct.tokens[0])) {
          if (direct.tokens.length !== 1) return { commands, status: null };
          try {
            const canonical = realpathSync(direct.tokens[0]);
            if (!truthPaths.includes(canonical) || !statSync(canonical).isFile()) return { commands, status: null };
            accessSync(canonical, constants.X_OK);
            status = canonical.endsWith("/true");
          } catch { return { commands, status: null }; }
        }
        if (["exit", "return"].includes(direct.tokens[0]) && certain) {
          const code = direct.tokens[1];
          return { commands, status: code === undefined ? status : /^\d+$/.test(code) ? Number(code) === 0 : null, stop: direct.tokens[0] };
        }
      }
    }
    return { commands, status };
  }
  return project(0, parts.length).commands;
}

function collectWorkflowCommands(workflow, includeNonGating, reachableOnly = false) {
  const commands = [];
  const workflowShell = defaultsRunShell(workflow, 0);
  for (const job of jobBlocks(workflow)) {
    const jobGating = canGate(job.block, 4);
    const jobShell = yamlValue(job.block, "shell", 4) ?? defaultsRunShell(job.block, 4);
    for (const step of stepBlocks(job.block)) {
      const gating = jobGating && canGate(step, 8);
      if (!includeNonGating && !gating) continue;
      const script = runScript(step);
      if (!script) continue;
      // Effective shell: step override, else job default (direct or defaults.run),
      // else workflow defaults.run, else the runner default.
      const shell = yamlValue(step, "shell", 8) ?? jobShell ?? workflowShell ?? null;
      // A fail-slow keep-going block collects per-invocation failures and re-raises
      // them with a summary `exit 1`, so a `set +e` inside it does NOT make the cargo
      // runs non-gating. The `ci-keep-going:` comment is an explicit contract on the
      // run body (see the domain-unit job); without it, `set +e` stays disqualifying.
      const keepGoing = /ci-keep-going:/.test(script);
      let terminated = false;
      let errexitDisabled = false;
      for (const surface of reachableOnly ? reachableShellCommands(script) : shellCommandTokens(script)) {
        if (terminated) break;
        if (surface.malformed) {
          commands.push({ job: job.name, step, tokens: surface.tokens, malformed: true, controlFlow: false, gating, shell });
          continue;
        }
        const controlFlow = surface.controlFlow ?? surface.tokens.some((token) => [";", "|", "&", "||", "&&"].includes(token));
        for (const segment of reachableOnly ? [surface] : commandSegments(surface)) {
          const { tokens } = segment;
          const conditional = segment.conditional || surface.conditional;
          const executable = directExecutable(tokens);
          // `set -e`/`set +e` on a conditional branch (`false && set -e`) never runs, so
          // it must not change the errexit state that governs later commands.
          if (executable.tokens[0] === "set" && !conditional) {
            if (executable.tokens.some((token) => token === "+e" || /^\+[A-Za-z]*e/.test(token))) {
              errexitDisabled = true;
            }
            if (executable.tokens.some((token) => token === "-e" || /^-[A-Za-z]*e/.test(token))) {
              errexitDisabled = false;
            }
          }
          commands.push({
            job: job.name,
            step,
            tokens,
            malformed: false,
            controlFlow: controlFlow || (errexitDisabled && !keepGoing),
            gating,
            shell,
          });
          // Only an unconditional exit/return terminates the script; `false && exit 0`
          // or `cmd || exit 1` does not, so later commands must still be scanned.
          if (!reachableOnly && (executable.tokens[0] === "exit" || executable.tokens[0] === "return") && !conditional) {
            terminated = true;
            break;
          }
        }
      }
    }
  }
  return commands;
}

/** Commands from workflow steps whose result can make the workflow fail. */
export function executableWorkflowCommands(workflow) {
  return collectWorkflowCommands(workflow, false);
}

/** Reachable commands only; allWorkflowCommands remains the full audit projection. */
export function reachableWorkflowCommands(workflow) {
  return collectWorkflowCommands(workflow, false, true);
}

/**
 * Every run-step command, including steps a literal `if: false` or
 * `continue-on-error: true` would drop. Each entry carries `gating` (whether the step
 * can make the workflow fail) and `shell` (the effective shell override, or null for
 * the runner default) so callers can refuse protected executors that would not gate.
 */
export function allWorkflowCommands(workflow) {
  return collectWorkflowCommands(workflow, true);
}

const assignment = /^[A-Za-z_][A-Za-z0-9_]*=/;

/** Strip only transparent `command`/`env` prefixes; never unwrap echo/bash/etc. */
export function directExecutable(tokens, depth = 0) {
  if (depth > 12 || tokens.length > 512) return { tokens: [], malformed: true };
  let index = 0;
  while (assignment.test(tokens[index] ?? "")) index += 1;
  if (index >= tokens.length) return { tokens: [], malformed: false };

  if (tokens[index] === "command") {
    index += 1;
    while (tokens[index]?.startsWith("-")) {
      const option = tokens[index];
      if (option === "--") { index += 1; break; }
      // `-p` only selects a default PATH; it takes no operand, so consume it and keep
      // classifying the command that follows (`command -p timeout ...` still runs timeout).
      if (option === "-p") { index += 1; continue; }
      if (option === "-v" || option === "-V") return { tokens: [], malformed: false };
      return { tokens: [], malformed: true };
    }
    return directExecutable(tokens.slice(index), depth + 1);
  }

  if (tokens[index] === "env") {
    index += 1;
    while (tokens[index]) {
      const option = tokens[index];
      if (option === "--") { index += 1; break; }
      if (["-u", "--unset", "-C", "--chdir"].includes(option)) {
        if (!tokens[index + 1]) return { tokens: [], malformed: true };
        index += 2;
      } else if (["-S", "--split-string"].includes(option)) {
        // `env -S "timeout 30 cosign ..."` splits the operand into the real command;
        // classify that command, not `env`.
        if (!tokens[index + 1]) return { tokens: [], malformed: true };
        const surfaces = shellCommandTokens(tokens[index + 1]);
        if (surfaces.length === 1 && !surfaces[0].malformed && surfaces[0].tokens.length > 0) {
          return directExecutable(surfaces[0].tokens, depth + 1);
        }
        return { tokens: [], malformed: true };
      } else if (["-i", "--ignore-environment", "-v", "--debug"].includes(option)) {
        index += 1;
      } else if (assignment.test(option)) {
        index += 1;
      } else if (option.startsWith("-")) {
        return { tokens: [], malformed: true };
      } else {
        break;
      }
    }
    return directExecutable(tokens.slice(index), depth + 1);
  }

  return { tokens: tokens.slice(index), malformed: false };
}
