import test from 'node:test';
import { HOST } from './hr-policy-parser/binding.mjs';
import { runNative } from './hr-policy-parser/native-run.mjs';

test(HOST, () => runNative());
