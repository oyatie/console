from pathlib import Path
import subprocess, os, time, json, uuid
out = Path(__file__).resolve().parent
env = dict(os.environ, DOCKER_CONTEXT='colima-console-release-20260919')
image = 'sha256:e2f1a0a06026c104902cd4b94ba6e594db957952cf058a9f91de3ee534b1fbc7'
command = ['docker', 'run', '--detach', '--network', 'none', '--name', 'console-release-smoke-' + uuid.uuid4().hex[:12], image]
container = subprocess.check_output(command, env=env, text=True).strip()
receipt = {'command': command, 'container': container, 'expected_database_mode': 'not_configured', 'product_workflow_qualification': False}
try:
    inspected = json.loads(subprocess.check_output(['docker', 'inspect', container], env=env, text=True))[0]
    assert inspected['HostConfig']['NetworkMode'] == 'none'
    assert inspected['HostConfig']['PortBindings'] in (None, {})
    assert inspected['Config']['User'] == '10001'
    assert inspected['Image'] == image
    receipt['configuration'] = {'network_mode': inspected['HostConfig']['NetworkMode'], 'port_bindings': inspected['HostConfig']['PortBindings'], 'user': inspected['Config']['User'], 'image': inspected['Image']}
    for route in ['healthz', 'readyz']:
        request = ['docker', 'exec', container, 'curl', '--silent', '--show-error', '--max-time', '1', '--write-out', '\n%{http_code}', 'http://127.0.0.1:8080/' + route]
        for attempt in range(40):
            result = subprocess.run(request, env=env, capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                break
            time.sleep(0.25)
        assert result.returncode == 0, route
        body, status = result.stdout.rsplit('\n', 1)
        decoded = json.loads(body)
        assert status == '200', route
        assert decoded['status'] == ('ok' if route == 'healthz' else 'ready')
        if route == 'readyz':
            assert decoded['database'] == 'not_configured'
        receipt[route] = {'command': request, 'status': int(status), 'body': decoded}
    receipt['result'] = 'pass: isolated no-database process smoke only'
finally:
    removed = subprocess.run(['docker', 'rm', '--force', container], env=env, capture_output=True, text=True, timeout=15)
    absence = subprocess.run(['docker', 'inspect', container], env=env, capture_output=True, text=True, timeout=15)
    receipt['cleanup'] = {'remove_exit': removed.returncode, 'absence_exit': absence.returncode, 'absence_stderr': absence.stderr.strip()}
    (out / 'runtime-smoke.json').write_text(json.dumps(receipt, indent=2) + '\n')
    assert removed.returncode == 0 and absence.returncode != 0 and ('no such object: ' + container) in absence.stderr.lower()
print('Runtime smoke passed; network none verified; test container removal read back')
