#!/usr/bin/env node
import { writeFileSync } from 'node:fs'

const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN
const headers = {
  Authorization: `Bearer ${token}`,
  Accept: 'application/vnd.github+json',
  'X-GitHub-Api-Version': '2022-11-28',
  'User-Agent': 'console-m68-babysit',
}
const want = 'fc1355b6f925b43ea9603406246da92f4f3a00df'
const maxRounds = Number(process.env.POLL_ROUNDS || 60)
const sleepMs = Number(process.env.POLL_SLEEP_MS || 30000)

async function get(path) {
  const ctrl = new AbortController()
  const t = setTimeout(() => ctrl.abort(), 25000)
  try {
    const res = await fetch(`https://api.github.com${path}`, { headers, signal: ctrl.signal })
    return { status: res.status, body: await res.json() }
  } finally {
    clearTimeout(t)
  }
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms))

for (let i = 1; i <= maxRounds; i++) {
  const prRes = await get('/repos/jason931225/console/pulls/751')
  const p = prRes.body
  const head = p.head?.sha
  writeFileSync('.cursor/pr751-view.json', JSON.stringify(p, null, 2))

  const checks = await get(`/repos/jason931225/console/commits/${head}/check-runs?per_page=100`)
  writeFileSync('.cursor/pr751-checks.json', JSON.stringify(checks.body, null, 2))
  const runs = checks.body.check_runs || []
  const reqCi = runs.find((c) => c.name === 'Required / CI')
  const reqSec = runs.find((c) => c.name === 'Required / Security')
  const auth = runs.find((c) => c.name === 'authenticate-console-authority')

  const ciRun = await get('/repos/jason931225/console/actions/runs/31508069619')
  writeFileSync('.cursor/pr751-run.json', JSON.stringify(ciRun.body, null, 2))

  console.log(
    `=== ${new Date().toISOString()} r=${i} tipOk=${head === want} mergeable=${p.mergeable}/${p.mergeable_state} merged=${p.merged}`,
  )
  console.log(
    `CI_RUN attempt=${ciRun.body.run_attempt} status=${ciRun.body.status} conclusion=${ciRun.body.conclusion || '-'}`,
  )
  console.log(
    `REQCI ${reqCi?.status}/${reqCi?.conclusion || '-'} REQSEC ${reqSec?.status}/${reqSec?.conclusion || '-'} AUTH ${auth?.status}/${auth?.conclusion || '-'}`,
  )

  if (head !== want) {
    console.log('TIP_DRIFT', head)
    process.exit(3)
  }
  if (p.merged) {
    console.log('MERGED', p.merge_commit_sha)
    process.exit(0)
  }

  const open = runs.filter((c) => c.status !== 'completed')
  if (open.length) {
    console.log('OPEN', open.map((c) => `${c.name}:${c.status}`).slice(0, 15).join('|'))
  }
  const fails = runs.filter((c) => ['failure', 'timed_out', 'action_required'].includes(c.conclusion))
  if (fails.length) {
    console.log('FAILS', fails.map((c) => c.name).join('|'))
  }

  const ciDone = ciRun.body.status === 'completed'
  const reqDone = reqCi?.status === 'completed' && reqSec?.status === 'completed' && auth?.status === 'completed'

  if (ciDone && reqDone) {
    const green =
      reqCi.conclusion === 'success' &&
      reqSec.conclusion === 'success' &&
      auth.conclusion === 'success' &&
      ciRun.body.conclusion === 'success'
    if (green) {
      console.log('ALL_REQUIRED_GREEN')
      process.exit(0)
    }
    console.log('SETTLED_RED', {
      reqCi: reqCi.conclusion,
      reqSec: reqSec.conclusion,
      auth: auth.conclusion,
      ciRun: ciRun.body.conclusion,
    })
    process.exit(4)
  }

  await sleep(sleepMs)
}
console.log('POLL_TIMEOUT')
process.exit(5)
