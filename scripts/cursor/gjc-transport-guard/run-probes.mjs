#!/usr/bin/env node
/**
 * Acceptance probe runner: 8 probes (6 hostile DENY + 2 safe ALLOW).
 * Prints examined=N and exits 1 if examined==0 or any probe mismatches.
 */
import { fileURLToPath } from 'node:url'
import { runAcceptanceProbes, resetExamined, assertExaminedNonZero } from './policy.mjs'

function help() {
  console.log(`usage: run-probes.mjs [--examined-zero]
  default          run 8 acceptance probes; require examined=8
  --examined-zero  invoke self-report with zero examinations (must exit 1)`)
}

export function main(argv = process.argv.slice(2)) {
  if (argv.includes('--help') || argv.includes('-h')) {
    help()
    return 0
  }
  if (argv.includes('--examined-zero')) {
    resetExamined()
    console.log('examined=0')
    try {
      assertExaminedNonZero('run-probes')
    } catch (error) {
      console.error(error.message)
      return 1
    }
    return 1
  }
  const report = runAcceptanceProbes()
  for (const r of report.results) {
    const mark = r.ok ? 'OK' : 'FAIL'
    console.log(`${r.id}: expect=${r.expect} got=${r.got} class=${r.classId} ${mark}`)
  }
  console.log(`examined=${report.examined}`)
  if (report.examined === 0) return 1
  if (report.examined !== 8) {
    console.error(`run-probes: expected examined=8, got examined=${report.examined}`)
    return 1
  }
  return report.exitCode
}

const invoked = process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]
if (invoked) process.exit(main())

