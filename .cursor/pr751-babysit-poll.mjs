#!/usr/bin/env node
/**
 * Poll Required checks for PR #751 until Required/CI completes or max rounds.
 */
import { writeFileSync } from 'node:fs'

const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN
if (!token) {
  console.error('missing token')
  process.exit(2)
}
const headers = {
  Authorization: `Bearer ${token}`,
  Accept: 'application/vnd.github+json',
  'X-GitHub-Api-Version': '2022-11-28',
  'User-Agent': 'console-m68-babysit',
}
const repo = 'jason931225/console'
const pr = 751
const want = 'fc1355b6f925b43ea9603406246da92f4f3a00df'
const maxRounds = Number(process.env.POLL_ROUNDS || 40)
const sleepMs = Number(process.env.POLL_SLEEP_MS || 45000)

async function get(path) {
  const ctrl = new AbortController()
  const t = setTimeout(() => ctrl.abort(), 25000)
  try {
    const res = await fetch(`https://api.github.com${path}`, { headers, signal: ctrl.signal })
    const body = await res.json()
    return { status: res.status, body }
  } finally {
    clearTimeout(t)
  }
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms))
}

for (let i = 1; i <= maxRounds; i++) {
  const prRes = await get(`/repos/${repo}/pulls/${pr}`)
  if (prRes.status !== 200) {
    console.error('pr_http', prRes.status, prRes.body?.message)
    process.exit(1)
  }
  const p = prRes.body
  const head = p.head?.sha
  writeFileSync('.cursor/pr751-view.json', JSON.stringify(p, null, 2))

  const checks = await get(`/repos/${repo}/commits/${head}/check-runs?per_page=100`)
  writeFileSync('.cursor/pr751-checks.json', JSON.stringify(checks.body, null, 2))
  const runs = checks.body.check_runs || []
  const reqCi = runs.find((c) => c.name === 'Required / CI')
  const reqSec = runs.find((c) => c.name === 'Required / Security')
  const auth = runs.find((c) => c.name === 'authenticate-console-authority')
  const stamp = new Date().toISOString()

  console.log(
    `=== ${stamp} round=${i} head=${head?.slice(0, 9)} tipMatch=${head === want} mergeable=${p.mergeable} state=${p.mergeable_state} merged=${p.merged}`,
  )
  console.log(
    `REQCI ${reqCi?.status}/${reqCi?.conclusion || '-'} | REQSEC ${reqSec?.status}/${reqSec?.conclusion || '-'} | AUTH ${auth?.status}/${auth?.conclusion || '-'}`,
  )

  if (head !== want) {
    console.log('TIP_DRIFT')
    process.exit(3)
  }
  if (p.merged) {
    console.log('ALREADY_MERGED', p.merge_commit_sha)
    process.exit(0)
  }

  const open = runs.filter((c) => c.status !== 'completed')
  if (open.length) {
    console.log(
      'OPEN',
      open
        .map((c) => `${c.name}:${c.status}`)
        .slice(0, 12)
        .join('|'),
    )
  }

  const fails = runs.filter((c) => c.conclusion === 'failure' || c.conclusion === 'timed_out')
  if (fails.length) {
    console.log(
      'FAILS',
      fails.map((c) => c.name).join('|'),
    )
  }

  if (reqCi?.status === 'completed') {
    if (reqCi.conclusion === 'success' && reqSec?.conclusion === 'success' && auth?.conclusion === 'success') {
      if (p.mergeable === true && (p.mergeable_state === 'clean' || p.mergeable_state === 'unstable')) {
        console.log('READY_TO_MERGE')
        process.exit(0)
      }
      console.log('CHECKS_GREEN_BUT_STATE', p.mergeable_state)
      // still may be blocked by branch protection review — report and exit for merge attempt
      if (reqCi.conclusion === 'success') {
        console.log('ATTEMPT_MERGE_CANDIDATE')
        process.exit(0)
      }
    }
    console.log('REQUIRED_CI_DONE', reqCi.conclusion)
    process.exit(reqCi.conclusion === 'success' ? 0 : 4)
  }

  await sleep(sleepMs)
}

console.log('POLL_TIMEOUT')
process.exit(5)
