"""Passive own-process stdout/stderr metadata; never read payload, argv or env."""
import datetime
import json
import subprocess
import threading
import time


class ProcessWatch:
    def __init__(self, path, root_pid):
        self.path = path
        self.root_pid = root_pid
        self.known = {root_pid}
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
                    record['processes'] = selected
                    if selected:
                        result = subprocess.run(['/usr/sbin/lsof', '-nP', '-a', '-p',
                            ','.join(str(r['pid']) for r in selected), '-d', '1,2', '-Fpcfatn'],
                            capture_output=True, text=True, timeout=3)
                        record['lsof_status'] = result.returncode
                        record['stdio_metadata'] = result.stdout
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
