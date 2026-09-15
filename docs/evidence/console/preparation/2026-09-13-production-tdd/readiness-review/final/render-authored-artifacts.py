import csv
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent
REPO = '/private/tmp/console-production-tdd-preparation-20260913'
SHA = '0f89f248812cbebd4e9be0997da7ed7812119ab6'
ledger = json.loads((ROOT / 'capability-family-preservation-ledger.json').read_text())
protocol = json.loads((ROOT / 'native-nonpayable-usability-protocol.json').read_text())
ids = [row['id'] for row in ledger['capabilities']]
assert len(ids) == len(set(ids)) == 23
assert len(protocol['tasks']) == protocol['task_count'] == 12
assert len({row['id'] for row in protocol['tasks']}) == 12
assert protocol['actual_task_observations'] == 0
assert all(row['observation_count'] == 0 and row['observations'] == [] for row in protocol['tasks'])
assert all(dep in ids for row in ledger['capabilities'] for dep in row['dependencies'])
required = ['id', 'family', 'phase', 'accountable_owning_layer', 'intended_acceptance_outcome', 'preservation_obligation', 'later_adoption_classification', 'invalidation_trigger']
assert all(row.get(key) for row in ledger['capabilities'] for key in required)

refs = set()
for row in ledger['capabilities']:
    refs.update(row['authority_and_design_refs'])
for row in protocol['tasks']:
    refs.update(row['source_refs'])
refs.update(ref for ref in protocol['source_refs'] if not ref.startswith('https:'))
sources = {}
for ref in sorted(refs):
    match = re.match(r'^(.*?)(?::(\d+))?$', ref)
    path, lineno = match.groups()
    if path not in sources:
        raw = subprocess.check_output(['git', 'show', SHA + ':' + path], cwd=REPO)
        blob = subprocess.check_output(['git', 'rev-parse', SHA + ':' + path], cwd=REPO, text=True).strip()
        sources[path] = {'path': path, 'git_blob': blob, 'sha256': hashlib.sha256(raw).hexdigest(), 'line_count': len(raw.splitlines())}
    if lineno:
        assert int(lineno) <= sources[path]['line_count'], ref
(ROOT / 'authored-source-custody.json').write_text(json.dumps({'base_sha': SHA, 'sources': list(sources.values()), 'note': 'Source existence and line bounds checked; this is not semantic or implementation acceptance.'}, indent=2) + '\n')

def safe(s):
    return str(s).replace('|', '\\|').replace('\n', ' ')

out = ['# Console capability-family preservation ledger', '', 'Authored review evidence at `' + SHA + '`. This is not product authority, implementation admission, full-suite leaf coverage or acceptance evidence.', '', 'The 11 platform families and 12 Console workflow families overlap deliberately. Dependencies name contract relationships; mutually dependent foundation rows are reviewed together, not dispatched as circular implementation lanes.', '', 'Every row requires preservation review before implementation. Detailed later-capability design/tests precede that capability’s implementation; actual acceptance and production exposure remain later evidence gates.', '']
for row in ledger['capabilities']:
    out += ['## ' + row['id'] + ' — ' + row['family'], '', '**Phase:** ' + row['phase'] + '. **Owning responsibility:** ' + row['accountable_owning_layer'] + '.', '', '**Dependencies:** ' + ', '.join(row['dependencies']) + '.', '', '**Intended acceptance:** ' + row['intended_acceptance_outcome'], '', '**Preserve now:** ' + row['preservation_obligation'], '', '**Adoption hypothesis:** ' + row['later_adoption_classification'] + '. **Invalidation trigger:** ' + row['invalidation_trigger'], '', '**Source:** ' + '; '.join('`' + ref + '`' for ref in row['authority_and_design_refs']) + '.', '']
out += ['## Remaining evidence', '', 'The complete versioned Foundry leaf inventory, leaf owner registrations, later detailed specifications/tests and actual acceptance evidence remain open. The JSON companion contains gate timing, source links and explicit status for every row. Owning layers are responsibilities, not invented named employees or instructions to create new crates.', '']
(ROOT / 'capability-family-preservation-ledger.md').write_text('\n'.join(out))
with (ROOT / 'capability-family-preservation-ledger.csv').open('w', newline='') as f:
    writer = csv.DictWriter(f, fieldnames=required + ['dependencies', 'authority_and_design_refs', 'evidence_status'])
    writer.writeheader()
    for row in ledger['capabilities']:
        writer.writerow({k: '; '.join(row[k]) if isinstance(row[k], list) else row[k] for k in writer.fieldnames})

out = ['# Native nonpayable workflow usability protocol', '', 'Authored review protocol at `' + SHA + '`, grounded in approved integrated design SHA `' + protocol['approved_integrated_design_sha'] + '`. Twelve tasks; zero participant sessions, zero observations and zero browser/product executions. No recruitment or external messages are requested.', '', '## Gates and method', '']
for key, value in protocol['gate_timing'].items():
    out += ['**' + key.replace('_', ' ').capitalize() + ':** ' + value, '']
for key, value in protocol['method'].items():
    out += ['**' + key.replace('_', ' ').capitalize() + ':** ' + value, '']
out += ['## Observation rules', '', 'Record exact candidate, configuration, fixtures, participant task context, outcome, errors, time, recovery, assistance and corroborating owner receipts. Role/context labels describe tasks; current policy governs access.', '', 'Outcomes: ' + ', '.join(protocol['outcome_values']) + '.', '', 'Assistance: ' + ', '.join(protocol['assistance_levels']) + '.', '']
for rule in protocol['measurement_rules']:
    out += ['- ' + rule]
out += ['', '## Untested hypotheses', '']
for hypothesis in protocol['proposed_hypotheses']:
    out += ['- **' + hypothesis['id'] + ':** ' + hypothesis['statement'] + ' No numerical target or observation is claimed.']
out += ['']
for row in protocol['tasks']:
    out += ['## ' + row['id'] + ' — ' + row['title'], '', '**Existing surfaces:** ' + ', '.join(row['surfaces']) + '.', '', '**Neutral goal:** ' + row['neutral_participant_goal'], '', '**Prerequisites:**', '']
    out += ['- ' + item for item in row['prerequisites']]
    out += ['', '**Essential information to notice:**', ''] + ['- ' + item for item in row['essential_information_to_notice']]
    out += ['', '**Observable success:**', ''] + ['- ' + item for item in row['observable_success']]
    out += ['', '**Errors to observe:**', ''] + ['- ' + item for item in row['error_observations']]
    out += ['', '**Recovery:** ' + row['recovery_observation'], '', '**Source:** ' + '; '.join('`' + ref + '`' for ref in row['source_refs']) + '.', '', '**Actual observations:** None. Status: AUTHORED_NOT_EXECUTED.', '']
out += ['## Interpretation and stop conditions', '']
for severity, description in protocol['error_severity'].items():
    out += ['**' + severity.capitalize() + ':** ' + description, '']
out += ['An absent prerequisite is reported as blocked rather than treated as evidence that the interaction passed or failed. Stop an affected critical scenario, retain evidence and return semantic changes through the repository’s existing design/test review. Repetition elsewhere cannot erase a consequential failure. This protocol itself clears no acceptance or production gate.', '']
(ROOT / 'native-nonpayable-usability-protocol.md').write_text('\n'.join(out))
template = {'schema_version': 'console-usability-observation-v1', 'status': 'EMPTY_TEMPLATE_NOT_AN_OBSERVATION', 'task_id': None, 'candidate_sha': None, 'design_sha': protocol['approved_integrated_design_sha'], 'fixture_and_oracle_refs': [], 'participant_pseudonym': None, 'task_context_and_current_permissions': None, 'browser_os_viewport_zoom_js_at': None, 'outcome': 'NOT_RUN', 'started_at': None, 'ended_at': None, 'pause_ms': None, 'system_wait_ms': None, 'time_to_first_relevant_action_ms': None, 'essential_information': [], 'errors': [], 'recovery': None, 'assistance_events': [], 'owner_receipt_or_effect_refs': [], 'participant_explanation': None, 'limitations': [], 'reviewer': None}
(ROOT / 'usability-observation-template.json').write_text(json.dumps(template, indent=2) + '\n')
receipt = {'status': 'STRUCTURAL_ARTIFACT_CHECK_ONLY', 'base_sha': SHA, 'command': 'python3 /private/tmp/console-readiness-workflows-research-20260913/render-authored-artifacts.py', 'ledger_rows': 23, 'tasks': 12, 'source_files': len(sources), 'source_refs': len(refs), 'checks': ['JSON parsed', 'unique stable IDs', 'dependency IDs resolve', 'required ledger fields populated', 'task and zero-observation counts agree', 'exact-SHA source blobs exist', 'source line references within bounds'], 'product_tests_discovered': 0, 'product_tests_executed': 0, 'usability_sessions': 0, 'limits': ['Not a semantic independent review', 'Not full leaf coverage', 'Not product or browser acceptance', 'No production or legal authority']}
(ROOT / 'authored-artifact-check.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(receipt))
