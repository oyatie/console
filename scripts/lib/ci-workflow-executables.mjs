// Minimal GitHub Actions + shell command extraction for reachability gates.
//
// This intentionally does not pretend YAML text is executable merely because it
// contains a command-shaped substring. Only `run:` bodies from jobs/steps that can
// gate CI are admitted, shell comments and continuations are resolved, and each
// returned entry is one actual command segment. A line such as
// `echo cargo test ...`, or a step guarded by literal `if: false`, therefore cannot
// manufacture test reachability.

function jobBlocks(workflow) {
  const jobsStart = workflow.indexOf("jobs:\n");
  if (jobsStart < 0) return [];
  const jobs = workflow.slice(jobsStart + "jobs:\n".length);
  return [...jobs.matchAll(/^  ([A-Za-z0-9_-]+):\n([\s\S]*?)(?=^  [A-Za-z0-9_-]+:|(?![\s\S]))/gm)]
    .map(([, name, block]) => ({ name, block }));
}

function stepBlocks(block) {
  const steps = block.match(/^    steps:\n([\s\S]*)$/m)?.[1] ?? "";
  return steps.split(/^      - /m).slice(1);
}

function runScript(step) {
  const scalar = step.match(/^        run: ([^\n]+)$/m)?.[1]?.trim();
  if (scalar && scalar !== "|") return scalar;
  const lines = step.split(/\r?\n/);
  const start = lines.findIndex((line) => line === "        run: |");
  if (start < 0) return "";
  const body = [];
  for (const line of lines.slice(start + 1)) {
    // YAML permits physically empty lines inside a block scalar. Do not let one truncate
    // the executable surface: the command after a visual paragraph break still runs.
    if (line === "") {
      body.push("");
    } else if (line.startsWith("          ")) {
      body.push(line.slice(10));
    } else {
      break;
    }
  }
  return body.join("\n");
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

export function shellCommandTokens(script) {
  const commands = [];
  let command = "";
  for (const physicalLine of script.split(/\r?\n/)) {
    const line = stripShellComment(physicalLine.trim());
    if (!line) continue;
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
    return { source, tokens, malformed };
  });
}

function commandSegments(surface) {
  const segments = [];
  let segment = [];
  for (const token of [...surface.tokens, ";"]) {
    if ([";", "|", "&", "||", "&&"].includes(token)) {
      if (segment.length > 0) segments.push(segment);
      segment = [];
    } else {
      segment.push(token);
    }
  }
  return segments;
}

/** Commands from workflow steps whose result can make the workflow fail. */
export function executableWorkflowCommands(workflow) {
  const commands = [];
  for (const job of jobBlocks(workflow)) {
    if (!canGate(job.block, 4)) continue;
    for (const step of stepBlocks(job.block)) {
      if (!canGate(step, 8)) continue;
      const script = runScript(step);
      if (!script) continue;
      let terminated = false;
      let errexitDisabled = false;
      for (const surface of shellCommandTokens(script)) {
        if (terminated) break;
        if (surface.malformed) {
          commands.push({ job: job.name, step, tokens: surface.tokens, malformed: true });
          continue;
        }
        const controlFlow = surface.tokens.some((token) => [";", "|", "&", "||", "&&"].includes(token));
        for (const tokens of commandSegments(surface)) {
          const executable = directExecutable(tokens);
          if (executable.tokens[0] === "set") {
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
            controlFlow: controlFlow || errexitDisabled,
          });
          if (executable.tokens[0] === "exit" || executable.tokens[0] === "return") {
            terminated = true;
            break;
          }
        }
      }
    }
  }
  return commands;
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
      if (option === "-p") { index += 2; continue; }
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
