'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

// Only the reviewed histories may publish a fresh public-evidence root.
const histories = Object.freeze({
  people: ['CONSOLE_INHERITED_PEOPLE_OUTPUT', 'requested'],
  policy: ['CONSOLE_INHERITED_POLICY_OUTPUT', 'requested'],
  'policy-startup': ['CONSOLE_INHERITED_POLICY_STARTUP_OUTPUT', 'requested'],
  'account-controller': ['CONSOLE_INHERITED_ACCOUNT_CONTROLLER_OUTPUT', 'account-controller'],
  'account-handoff': ['CONSOLE_INHERITED_ACCOUNT_HANDOFF_OUTPUT', 'account-company-handoff'],
  'company-manager': ['CONSOLE_INHERITED_COMPANY_MANAGER_OUTPUT', 'company-information-manager-current'],
  'group-process': ['CONSOLE_INHERITED_GROUP_PROCESS_OUTPUT', 'requested'],
  'group-correction': ['CONSOLE_INHERITED_GROUP_CORRECTION_OUTPUT', 'requested'],
  'group-held': ['CONSOLE_INHERITED_GROUP_HELD_OUTPUT', 'requested'],
});
function canonical(value, directory) {
  assert.equal(typeof value, 'string');
  assert.ok(value && !/[\r\n\0]/.test(value) && path.isAbsolute(value));
  assert.notEqual(value, path.parse(value).root);
  assert.equal(path.resolve(value), value, 'canonical path required');
  let current = value;
  for (;;) {
    assert.ok(!fs.lstatSync(current).isSymbolicLink(), 'symlink in path');
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  assert.equal(fs.realpathSync(value), value);
  const stat = fs.lstatSync(value);
  assert.ok(directory ? stat.isDirectory() : stat.isFile());
  return stat;
}
function absent(value) {
  try { fs.lstatSync(value); } catch (error) {
    if (error.code === 'ENOENT') return;
    throw error;
  }
  throw Error('output must not already exist');
}
assert.equal(process.argv.length, 6, 'tag, stage, fresh parent and GITHUB_ENV required');
const [tag, stage, parent, envFile] = process.argv.slice(2);
assert.ok(Object.hasOwn(histories, tag), 'unknown history');
const [variable, basename] = histories[tag];
const stageStat = canonical(stage, true);
assert.match(path.basename(stage), /^console-native-browser\.[A-Za-z0-9]+$/);
assert.equal(stageStat.mode & 0o077, 0, 'private installer stage required');
canonical(path.join(stage, 'outputs'), true);
const parentStat = canonical(parent, true);
assert.equal(path.dirname(parent), path.join(stage, 'outputs'));
assert.match(path.basename(parent), new RegExp('^inherited-' + tag + '\\.[A-Za-z0-9]{6,}$'));
assert.equal(parentStat.mode & 0o077, 0, 'private output allocation required');
if (process.getuid) {
  assert.equal(stageStat.uid, process.getuid());
  assert.equal(parentStat.uid, process.getuid());
}
const actual = path.join(parent, basename);
absent(path.join(parent, 'requested'));
absent(actual);
const envStat = canonical(envFile, false);
const previous = fs.readFileSync(envFile, 'utf8');
assert.ok(!previous || previous.endsWith('\n'), 'complete environment lines required');
for (const line of previous.split('\n')) {
  assert.ok(!line.startsWith(variable + '<<'), 'history setting already published');
  const separator = line.indexOf('=');
  const key = line.slice(0, separator), value = line.slice(separator + 1);
  assert.notEqual(key, variable, 'history setting already published');
  if (Object.values(histories).some(([name]) => name === key)) {
    assert.notEqual(path.dirname(value), parent, 'history allocation already published');
  }
}
const fd = fs.openSync(envFile, fs.constants.O_WRONLY | fs.constants.O_APPEND | fs.constants.O_NOFOLLOW);
try {
  const opened = fs.fstatSync(fd);
  assert.ok(opened.isFile() && opened.dev === envStat.dev && opened.ino === envStat.ino);
  fs.writeFileSync(fd, variable + '=' + actual + '\n');
} finally { fs.closeSync(fd); }
