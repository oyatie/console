#!/usr/bin/env node
/**
 * Babysit probe for PR #751 — avoids hung gh/git CLIs.
 * Uses GITHUB_TOKEN / GH_TOKEN from env. Never prints the token.
 */
import { writeFileSync } from 'node:fs'

const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN
if (!token) {
  console.error('missing GITHUB_TOKEN/GH_TOKEN')
  process.exit(2)
}

const repo = 'jason931225/console'
const pr = 751
const headers = {
  Authorization: `Bearer ${token}`,
  Accept: 'application/vnd.github+json',
  'X-GitHub-Api-Version': '2022-11-28',
  'User-Agent': 'console-m68-babysit',
}

async function get(path) {
  const ctrl = new AbortController()
  const t = setTimeout(() => ctrl.abort(), 25000)
  try {
    const res = await fetch(`https://api.github.com${path}`, { headers, signal: ctrl.signal })
    const text = await res.text()
    let body
    try {
      body = JSON.parse(text)
    } catch {
      body = { raw: text.slice(0, 500) }
    }
    return { status: res.status, body }
  } finally {
    clearTimeout(t)
  }
}

const prRes = await get(`/repos/${repo}/pulls/${pr}`)
writeFileSync('.cursor/pr751-view.json', JSON.stringify(prRes.body, null, 2))
if (prRes.status !== 200) {
  console.error('pr_http', prRes.status, prRes.body?.message || prRes.body)
  process.exit(1)
}

const head = prRes.body.head?.sha
const summary = {
  number: prRes.body.number,
  state: prRes.body.state,
  merged: prRes.body.merged,
  mergeable: prRes.body.mergeable,
  mergeable_state: prRes.body.mergeable_state,
  head,
  title: prRes.body.title,
  html_url: prRes.body.html_url,
  merge_commit_sha: prRes.body.merge_commit_sha,
}
console.log('PR', JSON.stringify(summary))

const checks = await get(`/repos/${repo}/commits/${head}/check-runs?per_page=100`)
writeFileSync('.cursor/pr751-checks.json', JSON.stringify(checks.body, null, 2))
if (checks.status !== 200) {
  console.error('checks_http', checks.status, checks.body?.message || checks.body)
  process.exit(1)
}

const runs = checks.body.check_runs || []
for (const c of runs) {
  console.log(`CHECK\t${c.name}\t${c.status}\t${c.conclusion || '-'}`)
}

const required = runs.filter((c) => /Required\s*\//i.test(c.name) || /authenticate-console-authority/i.test(c.name))
console.log('REQUIRED_SLICE', required.length)
for (const c of required) {
  console.log(`REQ\t${c.name}\t${c.status}\t${c.conclusion || '-'}`)
}

const rate = await get('/rate_limit')
if (rate.status === 200) {
  console.log('RATE_REMAINING', rate.body.resources?.core?.remaining)
}
