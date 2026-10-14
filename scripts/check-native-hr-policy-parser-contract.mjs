import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawn } from 'node:child_process';
import { ROOT, hash, loadBinding, regular } from './hr-policy-parser/binding.mjs';
import { classify } from './hr-policy-parser/observations.mjs';
import { candidate, command, inspect, logs } from './hr-policy-parser/native-run.mjs';

export async function childProcess(binding, run, nonce, isolation) {
  return await new Promise(resolve => {
    const child = spawn(binding.node.path, ['--test', 'scripts/check-native-hr-policy-parser-contract.test.mjs'], {
      cwd: ROOT, detached: true, stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...process.env, NODE_OPTIONS: '--test-reporter=tap', NODE_PATH: '', CONSOLE_HR_POLICY_RUN: run,
        CONSOLE_HR_POLICY_NONCE: nonce, CONSOLE_HR_POLICY_ISOLATION: isolation },
    });
    let stdout = ''; let stderr = ''; let abnormal = false;
    const stop = () => { abnormal = true; try { process.kill(-child.pid, 'SIGKILL'); } catch {} };
    const timer = setTimeout(stop, binding.timeout_ms);
    const interrupt = () => stop();
    process.on('SIGINT', interrupt); process.on('SIGTERM', interrupt);
    child.stdout.on('data', bytes => { stdout += bytes; if (stdout.length > 32 * 1024 * 1024) stop(); });
    child.stderr.on('data', bytes => { stderr += bytes; if (stderr.length > 32 * 1024 * 1024) stop(); });
    child.on('error', () => { abnormal = true; });
    child.on('close', (status, signal) => {
      clearTimeout(timer); process.removeListener('SIGINT', interrupt); process.removeListener('SIGTERM', interrupt);
      resolve({ status, signal, stdout, stderr, abnormal, pid: child.pid });
    });
  });
}
export async function main() {
  let run; let binding; let isolation; let child; let observation; let cleanup = false; let status = 126;
  try {
    binding = loadBinding(); binding.binding_sha256 = process.env.CONSOLE_HR_POLICY_BINDING_SHA256; candidate(binding);
    run = fs.mkdtempSync(path.join(os.tmpdir(), 'console-hr-policy-parser-'));
    const nonce = crypto.randomBytes(32).toString('hex'); isolation = `hr-policy-${crypto.randomBytes(16).toString('hex')}`;
    if (logs(isolation).length !== 0) throw new Error('isolation already occupied');
    const started = Date.now();
    child = await childProcess(binding, run, nonce, isolation);
    const completed = Date.now();
    if (child.abnormal) throw new Error('child interrupted or incomplete');
    observation = JSON.parse(regular(path.join(run, 'observation.json')));
    // Independently inspect real current-run logs/files; do not trust self-reported actions.
    const actualLogs = new Set(logs(isolation));
    if (!Number.isSafeInteger(observation.build_start_ms) || observation.build_start_ms < started
        || observation.build_start_ms > completed || !Array.isArray(observation.build_logs)
        || observation.build_logs.length === 0 || observation.build_logs.some(file => !actualLogs.has(file))) throw new Error('unowned or stale event log');
    const evidence = inspect(binding, isolation, observation.build_logs, observation.build_start_ms);
    if (JSON.stringify(evidence.actions) !== JSON.stringify(observation.compiler_actions)
        || JSON.stringify(evidence.ids) !== JSON.stringify(observation.fresh_build_ids)
        || JSON.stringify(evidence.logs) !== JSON.stringify(observation.event_logs)
        || hash(regular(path.join(run, 'build-report.json'))) !== observation.build_report_sha256
        || regular(path.join(run, 'build-report.json')).toString() !== observation.build_report) throw new Error('changed compiler observation');
    candidate(binding);
    status = classify(observation, child, binding, nonce, true);
  } catch (error) {
    process.stderr.write(`HR parser prerequisite: ${error.message}\n`); status = 126;
  } finally {
    try {
      if (child?.pid) {
        try { process.kill(-child.pid, 0); process.kill(-child.pid, 'SIGKILL'); status = 126; }
        catch (e) { if (e.code !== 'ESRCH') throw e; }
      }
      if (binding && isolation) {
        const kill = command(binding, isolation, ['kill'], 30_000);
        if (kill.status !== 0 || kill.signal !== null || kill.error) throw new Error('owned Buck daemon cleanup unverified');
      }
      if (run) { fs.rmSync(run, { recursive: true, force: false }); if (fs.existsSync(run)) throw new Error('run cleanup failed'); }
      cleanup = true;
    } catch (e) { process.stderr.write(`HR parser cleanup: ${e.message}\n`); status = 126; }
  }
  if (!cleanup || child?.abnormal) status = 126;
  if (observation) process.stdout.write(JSON.stringify({ status, cleanup, host: child && { status: child.status, signal: child.signal, stdout: child.stdout, stderr: child.stderr }, observation }) + '\n');
  return status;
}
if (process.argv[1] === new URL(import.meta.url).pathname) process.exitCode = await main();
