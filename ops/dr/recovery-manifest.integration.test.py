#!/usr/bin/env python3
"""Real PostgreSQL comparator controls, not CNPG failover/PITR certification."""
import importlib.util
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent
IMAGE = 'postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280'


def run(*argv, data=None, check=True):
    return subprocess.run(argv, input=data, text=True, capture_output=True, check=check, timeout=30)


def main():
    if not os.environ.get('DOCKER_CONTEXT'):
        raise SystemExit('select an explicit disposable DOCKER_CONTEXT')
    run('docker', 'image', 'inspect', IMAGE)  # Never pull or contact a provider.
    spec = importlib.util.spec_from_file_location('manifest', ROOT / 'recovery-manifest.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    with tempfile.TemporaryDirectory(prefix='console-recovery-oracle-') as tmp:
        env = Path(tmp) / 'container.env'
        env.write_text('POSTGRES_USER=oracle\nPOSTGRES_DB=console\nPOSTGRES_PASSWORD=' + secrets.token_hex(32) + '\n')
        env.chmod(0o600)
        # Register ownership before create: a lost create reply must not make
        # cleanup depend on the ID that the client never received.
        cid = 'console-recovery-oracle-' + secrets.token_hex(12)
        try:
            run('docker', 'create', '--name', cid, '--network', 'none', '--tmpfs', '/var/lib/postgresql:rw,size=256m', '--env-file', str(env), '--label', 'console.recovery.run=' + cid, IMAGE)
            run('docker', 'start', cid)
            for _ in range(60):
                if run('docker', 'exec', cid, 'pg_isready', '-U', 'oracle', '-d', 'console', check=False).returncode == 0:
                    # Require the final postgres PID1, not initdb's temporary server.
                    if run('docker', 'exec', cid, 'cat', '/proc/1/comm').stdout.strip() == 'postgres':
                        break
                time.sleep(1)
            else:
                raise AssertionError('synthetic PostgreSQL readiness timeout')

            def sql(text, check=True):
                return run('docker', 'exec', '-i', cid, 'psql', '-XqAt', '-U', 'oracle', '-d', 'console', '-v', 'ON_ERROR_STOP=1', data=text, check=check)

            def snapshot():
                return module.collect(iter(sql(module.SQL).stdout.splitlines(True)))

            setup = '''
CREATE TABLE payroll (id int, amount bigint, note text);
INSERT INTO payroll VALUES (1,5000,'before'),(2,20000,'before');
CREATE TABLE empty_inputs (value text);
CREATE TABLE events (id int, note text) PARTITION BY RANGE(id);
CREATE TABLE events_a PARTITION OF events FOR VALUES FROM (0) TO (100);
INSERT INTO events VALUES (1,'preserved');
CREATE SCHEMA "odd'schema";
CREATE TABLE "odd'schema"."quoted\""table" (value text);
INSERT INTO "odd'schema"."quoted\""table" VALUES ('private synthetic value');
'''
            sql(setup)
            baseline = snapshot()
            assert len(baseline['tables']) == 4
            assert sum(t['rows'] for t in baseline['tables']) == 4
            passed = 1
            print('PASS real snapshot including empty, quoted and partitioned tables')
            cases = [
                ('missing row', 'DELETE FROM payroll WHERE id=1;', "INSERT INTO payroll VALUES(1,5000,'before');"),
                ('same-count corruption', 'UPDATE payroll SET amount=1 WHERE id=1;', 'UPDATE payroll SET amount=5000 WHERE id=1;'),
                ('duplicate replaces distinct row', 'UPDATE payroll SET id=1,amount=5000 WHERE id=2;', "DELETE FROM payroll; INSERT INTO payroll VALUES(1,5000,'before'),(2,20000,'before');"),
                ('PITR overshoot', "INSERT INTO payroll VALUES(3,1,'after');", 'DELETE FROM payroll WHERE id=3;'),
                ('missing empty table', 'DROP TABLE empty_inputs;', 'CREATE TABLE empty_inputs(value text);'),
                ('column type drift', 'ALTER TABLE empty_inputs ALTER COLUMN value TYPE varchar;', 'ALTER TABLE empty_inputs ALTER COLUMN value TYPE text;'),
                ('missing partition row', 'DELETE FROM events;', "INSERT INTO events VALUES(1,'preserved');"),
            ]
            for name, fault, repair in cases:
                sql(fault)
                assert snapshot() != baseline, name
                sql(repair)
                assert snapshot() == baseline, name + ': repaired independent positive control'
                print('PASS ' + name + ' rejected, repaired state matches')
                passed += 1
            sql("CREATE FOREIGN DATA WRAPPER synthetic; CREATE SERVER synthetic FOREIGN DATA WRAPPER synthetic; CREATE FOREIGN TABLE remote_events PARTITION OF events FOR VALUES FROM(100) TO(200) SERVER synthetic;")
            foreign = sql(module.SQL, check=False)
            assert foreign.returncode != 0
            assert 'recovery_verification.unsupported_foreign_table' in foreign.stderr
            assert '"complete"' not in foreign.stdout
            print('PASS foreign partition refused before traversal')
            passed += 1
            print(json.dumps({'tests': passed, 'passed': passed, 'failed': 0, 'scope': 'real PG18.6 snapshot comparator; not actual backup restore/PITR'}))
        finally:
            # Inspect only ownership labels, never credential-bearing Config.Env.
            owner = run('docker', 'inspect', '--type', 'container', '--format', '{{json .Config.Labels}}', cid, check=False)
            if owner.returncode == 0:
                assert json.loads(owner.stdout).get('console.recovery.run') == cid, 'refuse unrelated container cleanup'
                try:
                    removal = run('docker', 'rm', '-fv', cid, check=False)
                finally:
                    absent = run('docker', 'inspect', '--type', 'container', cid, check=False)
                    assert absent.returncode != 0 and ('No such container: ' + cid) in absent.stderr, 'owned resource cleanup unconfirmed'
                assert removal.returncode == 0, 'owned resource removal failed'
            else:
                assert ('No such container: ' + cid) in owner.stderr, 'owned resource state unknown'
            print('owned_container_cleanup=confirmed')


if __name__ == '__main__':
    main()
