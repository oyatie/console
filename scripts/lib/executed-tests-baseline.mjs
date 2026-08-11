// The ratchet's comparison, separated from the resolver so it can be tested without a
// backend tree — and so a test never has to mutate the committed baseline to exercise it.
//
// The baseline is a SET of named binaries, not a count. A count is blind to substitution:
// with `10` as the bar, wiring one dark test while a different one goes dark keeps the
// gate green while the repository silently swaps which test cannot fail. Naming them also
// lets independent lanes append to their own bucket instead of contending on one integer.

/**
 * @param {string[]} dark      test binaries that reach no workflow step, as measured now
 * @param {object}   doc       parsed docs/program/executed-tests-baseline.json
 * @param {string}   label     how to refer to the baseline file in messages
 * @returns {{fatal: string|null, advisory: string|null}}
 */
export function evaluateBaseline(dark, doc, label) {
  // Only deferred_* keys are accepted-dark buckets. The baseline also carries
  // defined_feature_variants, another string array whose members are deliberately executing;
  // recognizing buckets by shape would let that metadata silently widen the accepted set.
  const buckets = Object.entries(doc).filter(([key]) => key.startsWith("deferred_"));
  if (buckets.length === 0) {
    return {
      fatal: `${label} names no accepted-dark files. A bare count cannot detect substitution; restore the named buckets.`,
      advisory: null,
    };
  }
  const malformed = buckets.find(
    ([, value]) => !Array.isArray(value) || !value.every((entry) => typeof entry === "string"),
  );
  if (malformed) {
    return {
      fatal: `${label} has malformed accepted-dark bucket ${malformed[0]}; every deferred_* value must be an array of binary names.`,
      advisory: null,
    };
  }
  const accepted = new Set(buckets.flatMap(([, v]) => v));

  // The count is derived from the set, so a hand-edit that updates one and not the other
  // is caught here rather than silently widening the bar.
  //
  // It is mandatory, not optional: the number is derived documentation and a cheap guard
  // against an incomplete hand-edit to one of the named buckets.
  if (!Number.isInteger(doc.dark_baseline) || doc.dark_baseline < 0) {
    return {
      fatal: `${label} has no non-negative integer dark_baseline. The count is the cross-check that keeps the named buckets honest.`,
      advisory: null,
    };
  }
  if (doc.dark_baseline !== accepted.size) {
    return {
      fatal: `dark_baseline is ${doc.dark_baseline} but ${accepted.size} files are named. The count is derived from the set — make them agree.`,
      advisory: null,
    };
  }

  const added = dark.filter((f) => !accepted.has(f)).sort();
  if (added.length > 0) {
    return {
      fatal: [
        `${added.length} test binary(ies) execute nowhere and are not named in ${label}:`,
        ...added.map((f) => `  ${f}`),
        `Wire each into a workflow step, or the repository has ${added.length} more test(s) that cannot fail.`,
      ].join("\n"),
      advisory: null,
    };
  }

  const darkSet = new Set(dark);
  const cleared = [...accepted].filter((f) => !darkSet.has(f)).sort();
  if (cleared.length > 0) {
    return {
      // Exact-set behavior from the Cargo-aware train: a stale cleared member leaves a free
      // slot for a later substitution, so it must be maintained in the same change.
      fatal: [
        `${cleared.length} previously-dark binary(ies) now execute or are gone. Remove them from ${label} to lock the gain in:`,
        ...cleared.map((f) => `  ${f}`),
      ].join("\n"),
      advisory: null,
    };
  }

  return { fatal: null, advisory: null };
}

// Lexical static evidence only: counts declared test attributes without evaluating
// cfg expressions or `#[ignore]`, so callers must never describe the result as executed
// runtime cases. Comments and string/char/raw-string literals are stripped first so a
// doc-comment or prose paste of `#[test]` (process.doc-comment-test-attr-false-count)
// cannot inflate the ratchet — same class as process.doc-comment-cfg-test-false-dark.
const TEST_ATTRIBUTE = /^[ \t]*#\[(?:tokio::|sqlx::)?test(?:\([^\n)]*\))?\]/gm;

/**
 * Strip line/block comments and string/char/raw-string literals, replacing each
 * with a space. Doc comments (`///`, `//!`, `/**`) are ordinary comments here.
 *
 * @param {string} source
 * @returns {string}
 */
export function stripRustCommentsAndStringLiterals(source) {
  let output = "";
  let index = 0;
  while (index < source.length) {
    const char = source[index];
    const next = source[index + 1];

    if (char === "/" && next === "/") {
      while (index < source.length && source[index] !== "\n") index += 1;
      output += " ";
      continue;
    }
    if (char === "/" && next === "*") {
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source[index] === "/" && source[index + 1] === "*") {
          depth += 1;
          index += 2;
        } else if (source[index] === "*" && source[index + 1] === "/") {
          depth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      output += " ";
      continue;
    }
    if (char === "r" && (next === '"' || next === "#")) {
      const raw = source.slice(index).match(/^r(#*)"/);
      if (raw) {
        const terminator = `"${raw[1]}`;
        const end = source.indexOf(terminator, index + raw[0].length);
        index = end === -1 ? source.length : end + terminator.length;
        output += " ";
        continue;
      }
    }
    if (char === "'") {
      const literal = source.slice(index).match(/^'(?:\\.|[^\\'])'/);
      if (literal) {
        index += literal[0].length;
        output += " ";
        continue;
      }
    }
    if (char === '"') {
      index += 1;
      while (index < source.length) {
        const inner = source[index];
        index += 1;
        if (inner === "\\") {
          if (index < source.length) index += 1;
          continue;
        }
        if (inner === '"') break;
      }
      output += " ";
      continue;
    }

    output += char;
    index += 1;
  }
  return output;
}

export function countDeclaredTestAttributes(source) {
  return (stripRustCommentsAndStringLiterals(source).match(TEST_ATTRIBUTE) ?? []).length;
}

export function evaluateTestAttributeBaseline(current, baseline, label) {
  if (!baseline
    || typeof baseline !== "object"
    || Array.isArray(baseline)
    || Object.values(baseline).some((count) => !Number.isInteger(count) || count < 0)) {
    return {
      fatal: `${label} must contain a test_attribute_baseline object of non-negative integer static attribute counts.`,
    };
  }

  const lost = [];
  for (const [source, was] of Object.entries(baseline)) {
    if (!(source in current)) {
      lost.push(`${source}: ${was} -> gone (source deleted, or no binary for it reaches a CI step)`);
    } else if (current[source] < was) {
      lost.push(`${source}: ${was} -> ${current[source]} (-${was - current[source]})`);
    }
  }
  if (lost.length > 0) {
    return {
      fatal: [
        `${lost.length} reachable test source(s) lost declared test attributes:`,
        ...lost.map((loss) => `  ${loss}`),
        "",
        "If each removal is intentional because its subject is gone too, say so in the commit message and run 'node scripts/check-executed-tests.mjs --update'.",
      ].join("\n"),
    };
  }

  const gained = Object.entries(current).filter(
    ([source, count]) => count > (baseline[source] ?? 0),
  );
  if (gained.length > 0) {
    return {
      fatal: `${gained.length} reachable test source(s) gained declared test attributes. Run --update to lock the gain in before merge.`,
    };
  }
  return { fatal: null };
}
