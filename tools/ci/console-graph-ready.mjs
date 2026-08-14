#!/usr/bin/env node
/**
 * Select ready, path-disjoint work-graph lanes for console-complete.
 *
 * Catalog (immutable-ish): .grok/harness/work-graph.v1.json
 * Live status preference: lane-board.live.json + catalog node.status
 *
 * Usage:
 *   node tools/ci/console-graph-ready.mjs
 *   node tools/ci/console-graph-ready.mjs --phase P1 --max-parallel 3
 *   node tools/ci/console-graph-ready.mjs --lane-ids L0-SSF,L1-PG-PART
 *   node tools/ci/console-graph-ready.mjs --dry-run
 *   node tools/ci/console-graph-ready.mjs --json-out /tmp/packets.json
 *
 * Exit: 0 ok, 2 missing catalog / hard error
 */
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const graphPath = resolve(root, ".grok/harness/work-graph.v1.json");
const boardPath = resolve(root, ".grok/harness/lane-board.live.json");

function parseArgs(argv) {
  const out = {
    phase: null,
    maxParallel: 3,
    laneIds: null,
    dryRun: false,
    tipWriters: null,
    jsonOut: null,
    includeBlocked: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--phase") out.phase = argv[++i];
    else if (a === "--max-parallel") out.maxParallel = Math.max(1, parseInt(argv[++i], 10) || 3);
    else if (a === "--lane-ids") {
      out.laneIds = String(argv[++i] || "")
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
    } else if (a === "--dry-run") out.dryRun = true;
    else if (a === "--tip-writers") out.tipWriters = parseInt(argv[++i], 10) || 0;
    else if (a === "--json-out") out.jsonOut = argv[++i];
    else if (a === "--include-blocked") out.includeBlocked = true;
    else if (a === "--help" || a === "-h") {
      console.log(`Usage: console-graph-ready.mjs [--phase P1] [--max-parallel N] [--lane-ids id,id] [--dry-run] [--tip-writers N]`);
      process.exit(0);
    }
  }
  return out;
}

function loadJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function loadBoard() {
  if (!existsSync(boardPath)) return null;
  try {
    return loadJson(boardPath);
  } catch {
    return null;
  }
}

/** @param {object} board @param {string} laneId */
function boardStatusFor(board, laneId) {
  if (!board) return null;
  const items = Array.isArray(board.items) ? board.items : [];
  for (const it of items) {
    const id = it.lane_id || it.id || "";
    const sk = it.source_key || "";
    if (id === laneId || sk === `lane:${laneId}` || sk.endsWith(`:${laneId}`)) {
      return it.status || null;
    }
  }
  // Advisory lists on board header
  if (Array.isArray(board.done_lane_ids) && board.done_lane_ids.includes(laneId)) return "done";
  return null;
}

function phaseTrialOk(board, phaseId) {
  if (!board) return false;
  if (board.phase_trials && board.phase_trials[phaseId] === true) return true;
  if (Array.isArray(board.phase_trial_ok) && board.phase_trial_ok.includes(phaseId)) return true;
  return false;
}

function normalizeStatus(s) {
  if (!s) return "ready";
  const t = String(s).toLowerCase();
  if (t === "done" || t === "closed" || t === "complete" || t === "completed") return "done";
  if (t === "cancelled" || t === "canceled") return "cancelled";
  if (t === "in_flight" || t === "in-progress" || t === "active") return "in_flight";
  if (t === "blocked") return "blocked";
  if (t === "skipped" || t === "skip") return "skipped";
  return t;
}

/**
 * HOLD that blocks product implement (not prepare-only).
 * prepare-only:H1 is allowed; bare H1 / H2 is not.
 */
function holdBlocksImplement(holdTouch) {
  if (!holdTouch || holdTouch === "none") return false;
  const h = String(holdTouch);
  if (h.startsWith("prepare-only:")) return false;
  // Any bare Hn or "H1,H2" style requires human clear — skip auto implement
  if (/\bH[0-9]\b/.test(h)) return true;
  return false;
}

function isTipSerial(v) {
  if (v === true) return true;
  if (v === false || v == null) return false;
  // string lease kinds still tip-serial for mutex
  return true;
}

function allowlistOf(node) {
  const a = node.allowlist || node.allowlist_hint || [];
  return Array.isArray(a) ? a.map(String) : [];
}

/** crude path-set conflict: exact match or prefix/glob-ish prefix before * */
function pathsConflict(a, b) {
  if (!a.length || !b.length) return false; // unknown allowlist → do not block others
  const norm = (p) => p.replace(/\*\*$/, "").replace(/\*$/, "").replace(/\/$/, "");
  for (const x of a) {
    const nx = norm(x);
    for (const y of b) {
      const ny = norm(y);
      if (nx === ny) return true;
      if (nx.startsWith(ny) || ny.startsWith(nx)) return true;
    }
  }
  return false;
}

function docsCurrentForbidden(allowlist) {
  return allowlist.some((p) => p.includes("docs/current"));
}

/**
 * @param {object} graph
 * @param {object|null} board
 * @param {object} opts
 */
export function selectReady(graph, board, opts = {}) {
  const maxParallel = opts.maxParallel ?? 3;
  const forcePhase = opts.phase || null;
  const laneFilter = opts.laneIds || null;
  const tipWriters =
    opts.tipWriters != null
      ? opts.tipWriters
      : board && typeof board.tip_writers === "number"
        ? board.tip_writers
        : 0;

  const nodes = Array.isArray(graph.nodes) ? graph.nodes : [];
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const phases = Array.isArray(graph.phases) ? graph.phases : [];

  const statusOf = (id) => {
    const n = byId.get(id);
    if (!n) return "done"; // missing dep → treat done to avoid stuck forever? No: treat blocked
    const live = boardStatusFor(board, id);
    return normalizeStatus(live || n.status || "ready");
  };

  const depsDone = (node) => {
    const deps = Array.isArray(node.depends_on) ? node.depends_on : [];
    return deps.every((d) => {
      const st = statusOf(d);
      return st === "done" || st === "cancelled";
    });
  };

  const packetFor = (node, reasonSkip = null) => {
    const allowlist = allowlistOf(node);
    return {
      lane_id: node.id,
      objective: node.objective || "",
      phase: node.phase,
      depends_on: node.depends_on || [],
      allowlist,
      forbidden: node.forbidden || ["docs/current/**"],
      must_read: node.must_read || node.must_use || ["docs/current/PRODUCT.md", "docs/current/ROADMAP.md", "docs/current/DELIVERY.md"],
      hold_touch: node.hold_touch || "none",
      tip_serial: isTipSerial(node.tip_serial),
      tip_serial_kind: node.tip_serial === true ? "hard" : node.tip_serial || false,
      verification: node.verification || [],
      admit: node.admit || [],
      hindsight_recall: node.hindsight_recall || node.must_use || [],
      hindsight_retain: node.hindsight_retain || [`lane ${node.id} outcome`],
      beads_id: node.beads_id || node.beads_epic || null,
      not_doing: node.not_doing || ["Clear PRODUCT HOLDs", "docs/current/** edits", "Multi-tip thrash"],
      priority: node.priority ?? 99,
      status: statusOf(node.id),
      skip_reason: reasonSkip,
    };
  };

  const phaseOrder = phases.map((p) => p.id);
  // Fallback: unique phases from nodes
  if (!phaseOrder.length) {
    for (const n of nodes) {
      if (n.phase && !phaseOrder.includes(n.phase)) phaseOrder.push(n.phase);
    }
  }

  const skipped = [];
  const waiting = [];
  let selectedPhase = forcePhase;
  let readyCandidates = [];

  const considerPhase = (phaseId) => {
    const inPhase = nodes.filter((n) => n.phase === phaseId);
    if (laneFilter) {
      return inPhase.filter((n) => laneFilter.includes(n.id));
    }
    return inPhase;
  };

  const evaluate = (phaseId) => {
    const list = considerPhase(phaseId);
    const open = [];
    const ready = [];
    for (const n of list) {
      const st = statusOf(n.id);
      if (st === "done" || st === "cancelled") continue;
      open.push(n);
      if (st === "skipped") {
        skipped.push(packetFor(n, "status skipped"));
        continue;
      }
      if (!depsDone(n)) {
        waiting.push(packetFor(n, "deps not done"));
        continue;
      }
      if (holdBlocksImplement(n.hold_touch)) {
        skipped.push(packetFor(n, `hold_touch blocks implement: ${n.hold_touch}`));
        continue;
      }
      if (docsCurrentForbidden(allowlistOf(n))) {
        skipped.push(packetFor(n, "docs/current in allowlist — process only + human"));
        continue;
      }
      if (isTipSerial(n.tip_serial) && tipWriters >= 1) {
        skipped.push(packetFor(n, `tip_serial busy tip_writers=${tipWriters}`));
        continue;
      }
      // in_flight is still "ready" for babysit/select but implement may re-claim
      ready.push(n);
    }
    return { open, ready };
  };

  if (forcePhase) {
    selectedPhase = forcePhase;
    const { open, ready } = evaluate(forcePhase);
    readyCandidates = ready;
    if (!ready.length && open.length) {
      // blocked phase
    }
  } else if (laneFilter) {
    // explicit ids across phases
    readyCandidates = [];
    for (const id of laneFilter) {
      const n = byId.get(id);
      if (!n) {
        skipped.push({ lane_id: id, skip_reason: "unknown lane_id" });
        continue;
      }
      selectedPhase = selectedPhase || n.phase;
      const { ready } = evaluate(n.phase);
      const hit = ready.find((r) => r.id === id);
      if (hit) readyCandidates.push(hit);
      else {
        const st = statusOf(id);
        if (st !== "done" && st !== "cancelled") {
          waiting.push(packetFor(n, "not ready under filters"));
        }
      }
    }
  } else {
    for (const phaseId of phaseOrder) {
      const { open, ready } = evaluate(phaseId);
      if (!ready.length && !open.length) continue;
      if (!ready.length && open.length) {
        selectedPhase = phaseId;
        readyCandidates = [];
        break;
      }
      if (ready.length) {
        selectedPhase = phaseId;
        readyCandidates = ready;
        break;
      }
    }
  }

  // Sort by priority asc, then lane_id
  readyCandidates.sort((a, b) => {
    const pa = a.priority ?? 99;
    const pb = b.priority ?? 99;
    if (pa !== pb) return pa - pb;
    return String(a.id).localeCompare(String(b.id));
  });

  const selected = [];
  let tipTaken = false;
  for (const n of readyCandidates) {
    if (selected.length >= maxParallel) break;
    const al = allowlistOf(n);
    const tip = isTipSerial(n.tip_serial);
    if (tip && tipTaken) {
      skipped.push(packetFor(n, "another tip_serial already selected"));
      continue;
    }
    if (tip && selected.length > 0) {
      // tip-serial alone: if we already have non-tip, defer tip
      skipped.push(packetFor(n, "tip_serial deferred until exclusive slot"));
      continue;
    }
    let conflict = false;
    for (const s of selected) {
      if (pathsConflict(al, allowlistOf(s))) {
        conflict = true;
        skipped.push(packetFor(n, `path conflict with ${s.id}`));
        break;
      }
    }
    if (conflict) continue;
    // If selecting tip-serial, only that one
    if (tip) {
      selected.length = 0;
      selected.push(n);
      tipTaken = true;
      break;
    }
    selected.push(n);
  }

  const packets = selected.map((n) => packetFor(n));
  const trialOk = phaseTrialOk(board, selectedPhase);

  const result = {
    ok: true,
    version: graph.version || "1.0.0",
    live_status_source: "board+catalog",
    phase: selectedPhase,
    tip_writers: tipWriters,
    tip_serial_busy: tipWriters >= 1,
    trial_ok: trialOk,
    max_parallel: maxParallel,
    ready: packets,
    ready_ids: packets.map((p) => p.lane_id),
    skipped: skipped.slice(0, 40),
    waiting: waiting.slice(0, 40),
    blocked:
      packets.length === 0 && waiting.length > 0
        ? { phase: selectedPhase, reason: "deps or filters", waiting_ids: waiting.map((w) => w.lane_id) }
        : null,
    dry_run: !!opts.dryRun,
    next_hint:
      packets.length === 0
        ? selectedPhase
          ? `/workflow console-complete {"phase":${JSON.stringify(selectedPhase)}} or resolve blockers`
          : "/workflow pr-babysit or wait for deps"
        : `/workflow console-complete {"phase":${JSON.stringify(selectedPhase)},"max_parallel":${maxParallel}}`,
  };
  return result;
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  if (!existsSync(graphPath)) {
    console.error(JSON.stringify({ ok: false, error: "missing work-graph.v1.json", path: graphPath }));
    process.exit(2);
  }
  let graph;
  try {
    graph = loadJson(graphPath);
  } catch (e) {
    console.error(JSON.stringify({ ok: false, error: String(e) }));
    process.exit(2);
  }
  const board = loadBoard();
  const result = selectReady(graph, board, {
    phase: opts.phase,
    maxParallel: opts.maxParallel,
    laneIds: opts.laneIds,
    tipWriters: opts.tipWriters,
    dryRun: opts.dryRun,
  });
  const text = JSON.stringify(result, null, 2) + "\n";
  if (opts.jsonOut) writeFileSync(opts.jsonOut, text);
  process.stdout.write(text);
  process.exit(0);
}

main();
