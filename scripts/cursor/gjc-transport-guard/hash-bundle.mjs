#!/usr/bin/env node
/**
 * Write the deterministic packed-source sha256 to BUNDLE-HASH.
 * Re-run after any packed source change so the registry claim stays reproducible.
 */
import { writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { hashBundleFiles } from './policy.mjs'

const root = dirname(fileURLToPath(import.meta.url))
const hashed = hashBundleFiles(root)
writeFileSync(join(root, 'BUNDLE-HASH'), `${hashed.sha256}\n`, 'utf8')
process.stdout.write(`${hashed.sha256}\n`)
