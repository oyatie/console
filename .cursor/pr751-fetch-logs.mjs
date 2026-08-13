#!/usr/bin/env node
import { writeFileSync } from 'node:fs'

const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN
const headers = {
  Authorization: `Bearer ${token}`,
  Accept: 'application/vnd.github+json',
  'X-GitHub-Api-Version': '2022-11-28',
  'User-Agent': 'console-m68-babysit',
}
const RUN = 31508069619

const jobsRes = await fetch(
  `https://api.github.com/repos/jason931225/console/actions/runs/${RUN}/jobs?per_page=100`,
  { headers },
)
const jobs = await jobsRes.json()
writeFileSync('.cursor/pr751-jobs.json', JSON.stringify(jobs, null, 2))
for (const job of jobs.jobs || []) {
  if (job.conclusion === 'failure' || /Required \/ CI/.test(job.name)) {
    console.log('JOB', job.id, job.conclusion, job.name)
    for (const s of job.steps || []) {
      if (s.conclusion && s.conclusion !== 'success' && s.conclusion !== 'skipped') {
        console.log('  STEP', s.number, s.conclusion, s.name)
      }
    }
  }
}

for (const jobId of [93854195061, 93847829573]) {
  const res = await fetch(`https://api.github.com/repos/jason931225/console/actions/jobs/${jobId}/logs`, {
    headers: { ...headers, Accept: 'application/vnd.github+json' },
    redirect: 'follow',
  })
  const text = await res.text()
  const path = `.cursor/pr751-job-${jobId}.log`
  writeFileSync(path, text)
  console.log('LOG', jobId, 'http', res.status, 'bytes', text.length)
  const lines = text.split(/\r?\n/)
  const interesting = lines.filter((l) =>
    /error|Error|FAIL|failed|Assertion|exit code|Process completed|##\[error\]/i.test(l),
  )
  console.log('--- interesting', jobId, interesting.length, '---')
  for (const l of interesting.slice(-80)) console.log(l)
}
