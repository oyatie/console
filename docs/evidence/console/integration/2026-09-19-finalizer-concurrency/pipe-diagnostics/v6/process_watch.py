"""Passive own-process stdout/stderr metadata; never read payload, argv or env."""
import datetime
import json
import subprocess
import threading
import re
import time


class ProcessWatch:
    def __init__(self, path, root_pid):
        self.path = path
        self.root_pid = root_pid
        self.known = {root_pid}
        self.tests = {}
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
                            match = re.search(r' ([a-z][a-z0-9_]+) --exact(?: |$)', args.stdout.strip())
                            self.tests[pid] = match[1] if match else 'unresolved'
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
