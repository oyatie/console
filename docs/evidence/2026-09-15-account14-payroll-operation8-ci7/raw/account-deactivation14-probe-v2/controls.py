from pathlib import Path
import importlib.util,json,re,hashlib
p=Path(__file__).parent
spec=importlib.util.spec_from_file_location('probe',p/'probe.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
c=json.loads((p/'contract.json').read_text());raw=Path('/private/tmp/account-deactivation14-796173fc-red-v1/probe.log').read_text()
assert m.classify(raw,1,c)==1
cases={
'missing-name':raw.replace('test '+c['roster'][0]+' ... FAILED\n',''),
'wrong-name':raw.replace(c['roster'][0],'wrong_case'),
'wrong-count':raw.replace('3 passed; 11 failed','4 passed; 10 failed'),
'ignored':raw.replace('0 ignored;','1 ignored;'),
'filtered':raw.replace('0 filtered out;','1 filtered out;'),
'unknown-panic':raw.replace('COMPANY_DEACTIVATION_CHANGED_ACCOUNT_CUSTODY:', 'UNKNOWN_FIXTURE_FAILURE:'),
'wrong-line':raw.replace('deactivate_revokes_credentials.rs:423:9:', 'deactivate_revokes_credentials.rs:424:9:'),
'wrong-missing-code':raw.replace('code: "42883"','code: "23505"'),
'duplicate-summary':raw+'\ntest result: FAILED. 3 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in1s\n',
}
for name,log in cases.items():
 try:m.classify(log,1,c)
 except (AssertionError,KeyError):pass
 else:raise AssertionError(name)
# Classifier positive control only, not application GREEN evidence.
green='\n'.join('test '+n+' ... ok' for n in c['roster'])+'\ntest result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in1s\n'
assert m.classify(green,0,c)==0
receipt={'checks':11,'passed':11,'meaning':'1actual retainedRED+9malformed-output negative controls+1synthetic parserGREEN;0newRustexecution','probe_sha256':hashlib.sha256((p/'probe.py').read_bytes()).hexdigest(),'contract_sha256':hashlib.sha256((p/'contract.json').read_bytes()).hexdigest()}
(p/'controls.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
