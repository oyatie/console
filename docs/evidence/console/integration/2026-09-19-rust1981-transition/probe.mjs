import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';

const root = process.cwd();
const {parsePin} = await import(pathToFileURL(resolve(root, 'scripts/lib/rust-pin.mjs')));
const pin = parsePin(readFileSync(resolve(root, 'rust-toolchain.toml'), 'utf8'));
assert.equal(pin.channel, '1.98.1', 'RELEASE_RUST_PIN: the explicit release contract requires Rust 1.98.1');
assert.deepEqual(pin.components, ['rustfmt', 'clippy']);
assert.deepEqual(pin.targets, ['wasm32-unknown-unknown']);
const lock = readFileSync(resolve(root, 'toolchains/rust/lock.bzl'), 'utf8');
assert.match(lock, /^RUST_CHANNEL = "1\.98\.1"$/m, 'RELEASE_RUST_LOCK: generated compiler lock must follow the canonical pin');
console.log('release Rust pin: 1 executable configuration check passed');
