"""Passive own-process stdout/stderr metadata; never read payload, argv or env."""
import datetime
import json
import subprocess
import threading
import re
import shlex
import time


class ProcessWatch:
    def __init__(self, path, root_pid):
        self.path = path
        self.root_pid = root_pid
        self.known = {root_pid}
        self.tests = {}
        self.test_names = ['admin_reads_only_branch_scoped_audits_and_read_access_is_audited', 'app_state', 'audit_attestation_preserves_current_grants_after_token_issuance', 'audit_attestation_requires_org_wide_audit_authority', 'audit_attestation_uses_current_role_after_token_issuance', 'audit_current_grant_history', 'audit_log_preserves_current_grants_after_token_issuance', 'audit_log_uses_current_branches_after_token_issuance', 'audit_log_uses_current_role_after_token_issuance', 'audit_role_demotion_history', 'authorization_conceals_and_isolates_without_leakage', 'await_actor_identity_lock_wait', 'evaluation_routes_are_mounted_by_the_authenticated_app_router', 'freshness_audit_get', 'freshness_audit_rows', 'grant_feature', 'identity_relinks_serialize_submit_detail_and_review_authorship', 'insert_audit', 'insert_audit_for_org', 'insert_audit_with_trace', 'link_user_employee', 'mechanic_role_is_denied_audit_read', 'new', 'review_identity_relationships_fail_closed_by_kind', 'router', 'runtime_role_pool', 'seed_branch', 'seed_branch_for_org', 'seed_employee', 'seed_org', 'seed_user', 'seed_user_with_branch', 'send', 'story_evaluation_001_walks_cycle_to_ledger_as_runtime_role', 'super_admin_audit_read_arms_org_and_isolates_cross_org_as_runtime_role', 'target_id_filter_isolates_one_object_and_trace_id_correlates_across_objects']
        self.stop = threading.Event()
        self.thread = threading.Thread(target=self.run)
        self.thread.start()

    def run(self):
        with self.path.open('x') as stream:
            while not self.stop.is_set():
                started = time.monotonic()
                record = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                          'monotonic': started, 'target_interval_seconds': 0.1}
                try:
                    result = subprocess.run(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,comm='],
                                            capture_output=True, text=True, timeout=3)
                    result.check_returncode()
                    rows = []
                    for line in result.stdout.splitlines():
                        pid, parent, group, comm = line.strip().split(None, 3)
                        rows.append({'pid': int(pid), 'parent': int(parent),
                                     'group': int(group), 'comm': comm})
                    while True:
                        children = {r['pid'] for r in rows if r['parent'] in self.known}
                        if children <= self.known:
                            break
                        self.known.update(children)
                    selected = [r for r in rows if r['pid'] in self.known]
                    for row in selected:
                        pid = row['pid']
                        if pid not in self.tests and re.search(r'/(audit_api|evaluation_cycle_api)-[0-9a-f]+$', row['comm']):
                            # Retain only a whitelisted Rust identifier immediately before --exact.
                            args = subprocess.run(['/bin/ps', '-p', str(pid), '-o', 'args='],
                                                  capture_output=True, text=True, timeout=3)
                            tokens = shlex.split(args.stdout)
                            matches = [x for x in tokens if x in self.test_names]
                            self.tests[pid] = matches[0] if len(matches) == 1 and '--exact' in tokens else 'unresolved'
                        if pid in self.tests:
                            row['test_name'] = self.tests[pid]
                    record['processes'] = selected
                    if selected:
                        result = subprocess.run(['/usr/sbin/lsof', '-nP', '-a', '-p',
                            ','.join(str(r['pid']) for r in selected), '-Fpcfatn'],
                            capture_output=True, text=True, timeout=3)
                        record['lsof_status'] = result.returncode
                        # Retain only PIPE records, including inherited descriptors beyond stdio.
                        processes = []
                        current = None
                        descriptor = None
                        for line in result.stdout.splitlines():
                            if line.startswith('p'):
                                current = {'pid': int(line[1:]), 'pipes': []}
                                processes.append(current)
                                descriptor = None
                            elif line.startswith('f') and current is not None:
                                descriptor = {'fd': line[1:]}
                                current['pipes'].append(descriptor)
                            elif descriptor is not None and line[:1] in ('a','t','n'):
                                descriptor[line[0]] = line[1:]
                        for process in processes:
                            process['pipes'] = [fd for fd in process['pipes'] if fd.get('t') == 'PIPE']
                        record['pipe_metadata'] = processes
                except Exception as error:
                    record['error_type'] = type(error).__name__
                elapsed = time.monotonic() - started
                record['elapsed_seconds'] = elapsed
                stream.write(json.dumps(record) + '\n')
                stream.flush()
                self.stop.wait(max(0, 0.1 - elapsed))

    def close(self):
        self.stop.set()
        self.thread.join(timeout=7)
        if self.thread.is_alive():
            raise RuntimeError('passive process watcher did not stop')
