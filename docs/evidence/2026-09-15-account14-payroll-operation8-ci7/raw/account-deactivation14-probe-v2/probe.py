from pathlib import Path
import hashlib,json,os,re,subprocess,uuid
if not __debug__: raise SystemExit(126)
ROOT=Path('/private/tmp/console-production-tdd-preparation-20260913')
HERE=Path(__file__).parent
DRIVER=Path('/private/tmp/account-deactivation6-1cef89ea-v1-fast/driver.sh')

def classify(log,code,contract):
    rows=re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',log,re.M)
    assert sorted(n for n,_ in rows)==contract['roster'], 'test roster differs'
    counts=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',log)
    assert len(counts)==1 and counts[0][3:]==('0','0','0'), 'incomplete counts'
    failed=[n for n,status in rows if status=='FAILED']
    assert tuple(map(int,counts[0][1:3]))==(14-len(failed),len(failed))
    assert code==(1 if failed else 0) and counts[0][0]==('FAILED' if failed else 'ok')
    blocks=re.findall(r'^---- (\S+) stdout ----\n(.*?)(?=^---- |^failures:|^test result:|\Z)',log,re.M|re.S)
    assert sorted(n for n,_ in blocks)==sorted(failed), 'missing or duplicate failure detail'
    semantic=0
    for name,block in blocks:
        allowed=contract['failures'][name]
        location=re.findall(r'panicked at (.*?):(\d+:\d+):\n([^\n]*)',block)
        assert len(location)==1 and location[0][:2]==(contract['panic_path'],allowed['location'])
        diagnostic=location[0][2]
        if allowed['kind']=='semantic':
            assert diagnostic==allowed['diagnostic'];semantic+=1
        else:
            assert allowed['diagnostic'] in diagnostic and 'code: "42883"' in diagnostic
    assert not failed or semantic>0, 'stage blockers alone are not semantic admission'
    return code

def main():
    data=(HERE/'contract.json').read_bytes()
    assert hashlib.sha256(data).hexdigest()=='28329ea489e991ae32dc8b84ac33e8f0f500ce0d5fb64d2d207fa7d15fa1b478'
    contract=json.loads(data)
    assert hashlib.sha256(DRIVER.read_bytes()).hexdigest()==contract['driver_sha256']
    assert hashlib.sha256((ROOT/contract['source_path']).read_bytes()).hexdigest()==contract['source_sha256']
    sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    out=Path('/private/tmp')/('account-deactivation14-probe-'+uuid.uuid4().hex)
    env=os.environ.copy();env['DOCKER_CONTEXT']='colima-console-custody-20260914'
    result=subprocess.run(['bash',str(DRIVER),str(ROOT),sha,str(out)],cwd=ROOT,env=env)
    print('Account14 evidence:',out,flush=True)
    assert result.returncode in (0,1)
    receipt=json.loads((out/'driver-receipt.json').read_text())
    assert receipt['source_pre_post']=='exactly equal'
    assert receipt['cleanup']=='confirmed owned container absent via successful bounded docker ps -a'
    for name,digest in receipt['artifacts'].items():
        assert hashlib.sha256((out/name).read_bytes()).hexdigest()==digest
    code=classify((out/'probe.log').read_text(),result.returncode,contract)
    print('Account14 classified exit:',code,flush=True)
    raise SystemExit(code)

if __name__=='__main__':
    try:main()
    except Exception as error:
        print('Account14 prerequisite or unreviewed failure:',type(error).__name__,flush=True)
        raise SystemExit(126)
