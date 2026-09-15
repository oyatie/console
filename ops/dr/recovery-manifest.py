#!/usr/bin/env python3
"""Compare a recovered SQL snapshot with independently retained expected state.

Only row SHA-256 values cross the psql pipe; no row payloads are logged. SQL sorts
hashes (spilling under work_mem); Python uses O(tables + columns) memory. This
checks regular table content/columns, not roles, routines, external stores or HA.
Capture must use an independently qualified snapshot at the intended recovery
point. Capturing the recovered database as its own reference proves nothing.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

SQL = r"""BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;
SET LOCAL search_path=pg_catalog;
SET LOCAL timezone='UTC';
SET LOCAL datestyle='ISO, YMD';
SET LOCAL intervalstyle='postgres';
SET LOCAL extra_float_digits=3;
SET LOCAL bytea_output='hex';
SET LOCAL statement_timeout='120s';
SET LOCAL work_mem='16MB';
SET LOCAL temp_file_limit='1GB';
SET LOCAL transaction_timeout='300s';
DO $$ BEGIN
 IF EXISTS (SELECT 1 FROM pg_class WHERE relkind='f') THEN
  RAISE EXCEPTION 'recovery_verification.unsupported_foreign_table';
 END IF;
END $$;
SELECT json_build_object('database',current_database(),
 'database_oid',(SELECT oid::text FROM pg_database WHERE datname=current_database()),
 'system_identifier',(SELECT system_identifier::text FROM pg_control_system()),
 'in_recovery',pg_is_in_recovery());
SELECT format('SELECT %L::json; SELECT encode(sha256(convert_to(row_to_json(t)::text,''UTF8'')),''hex'') COLLATE "C" FROM %I.%I t ORDER BY 1;',
 json_build_object('table',json_build_array(n.nspname,c.relname),'columns',
   (SELECT COALESCE(json_agg(json_build_array(a.attname,format_type(a.atttypid,a.atttypmod),a.attnotnull) ORDER BY a.attnum),'[]'::json)
    FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped)),
 n.nspname,c.relname)
FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
WHERE c.relkind IN ('r','p') AND NOT c.relispartition
 AND n.nspname NOT LIKE 'pg\_%' AND n.nspname<>'information_schema'
ORDER BY n.nspname COLLATE "C",c.relname COLLATE "C"
\gexec
COMMIT;
SELECT json_build_object('complete',true);
"""


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError('duplicate JSON key')
        value[key] = item
    return value


def decode(data):
    return json.loads(data, object_pairs_hook=unique_object)


def validate_header(header):
    if set(header) != {'database', 'database_oid', 'system_identifier', 'in_recovery'} or header['in_recovery'] is not False or not isinstance(header['database'], str) or not header['database']:
        raise ValueError('invalid recovery database identity or still in recovery')
    for field in ['database_oid', 'system_identifier']:
        if not isinstance(header[field], str) or not re.fullmatch('[1-9][0-9]*', header[field]):
            raise ValueError('invalid physical identity')


def validate_table(entry):
    if not isinstance(entry['table'], list) or len(entry['table']) != 2 or not all(isinstance(x, str) and x for x in entry['table']) or not isinstance(entry['columns'], list):
        raise ValueError('invalid table header')
    for column in entry['columns']:
        if not isinstance(column, list) or len(column) != 3 or not all(isinstance(x, str) and x for x in column[:2]) or type(column[2]) is not bool:
            raise ValueError('invalid column header')


def collect(stream):
    header = decode(next(stream))
    validate_header(header)
    tables = []
    current = None
    previous_hash = ''
    digest = None
    seen = set()
    complete = False
    for raw in stream:
        if complete:
            raise ValueError("output after completion")
        line = raw.rstrip('\n')
        if line.startswith('{'):
            entry = decode(line)
            if set(entry) == {'complete'} and entry['complete'] is True:
                complete = True
                continue
            if set(entry) != {'table', 'columns'}:
                raise ValueError('invalid table header')
            validate_table(entry)
            key = tuple(entry['table'])
            if key in seen:
                raise ValueError('duplicate table')
            seen.add(key)
            if current is not None:
                current['sha256'] = digest.hexdigest()
            current = dict(entry, rows=0)
            tables.append(current)
            digest = hashlib.sha256()
            previous_hash = ''
        else:
            if current is None or not re.fullmatch('[0-9a-f]{64}', line) or line < previous_hash:
                raise ValueError('invalid or unordered row digest')
            digest.update((line + '\n').encode('ascii'))
            current['rows'] += 1
            previous_hash = line
    if current is None or not complete:
        raise ValueError('missing tables or completion')
    current['sha256'] = digest.hexdigest()
    return dict(header, tables=tables)


def load_expected(path, database, target_time):
    with Path(path).open('rb') as source:
        data = source.read(16 * 1024 * 1024 + 1)
    if len(data) > 16 * 1024 * 1024:
        raise ValueError('reference too large')
    expected = decode(data)
    if set(expected) != {'version', 'target_time', 'state'} or type(expected['version']) is not int or expected['version'] != 1:
        raise ValueError('invalid expected manifest')
    state = expected['state']
    if expected['target_time'] != target_time or state['database'] != database:
        raise ValueError('expected recovery target mismatch')
    if set(state) != {'database', 'database_oid', 'system_identifier', 'in_recovery', 'tables'} or state['in_recovery'] is not False or not state['tables']:
        raise ValueError('invalid expected database state')
    validate_header({k: v for k, v in state.items() if k != 'tables'})
    seen = set()
    for t in state['tables']:
        if set(t) != {'table', 'columns', 'rows', 'sha256'} or type(t['rows']) is not int or t['rows'] < 0 or not re.fullmatch('[0-9a-f]{64}', t['sha256']):
            raise ValueError('invalid expected table state')
        validate_table(t)
        key = tuple(t['table'])
        if key in seen:
            raise ValueError('duplicate expected table')
        seen.add(key)
    return expected, hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['sql', 'capture', 'check', 'verify'])
    parser.add_argument('--expected')
    parser.add_argument('--database')
    parser.add_argument('--target-time', default='')
    args = parser.parse_args()
    if args.mode == 'sql':
        print(SQL)
        return
    if args.mode == 'capture':
        state = collect(iter(sys.stdin))
        print(json.dumps({'version': 1, 'target_time': args.target_time, 'state': state}, sort_keys=True))
        return
    if not args.expected or not args.database:
        parser.error('--expected and --database are required')
    expected, digest = load_expected(args.expected, args.database, args.target_time)
    if args.mode == 'verify':
        actual = collect(iter(sys.stdin))
        if actual != expected['state']:
            raise ValueError('recovered state differs from independent reference')
        print(f'verify_recovered_state=match manifest_sha256={digest} tables={len(actual["tables"])}')
        if args.target_time:
            print('verify_pitr_state=match')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, TypeError, StopIteration, OSError, AttributeError):
        # Never expose rows, expected values or SQL/connection detail on failure.
        print('recovery_verification=failed', file=sys.stderr)
        raise SystemExit(1)
