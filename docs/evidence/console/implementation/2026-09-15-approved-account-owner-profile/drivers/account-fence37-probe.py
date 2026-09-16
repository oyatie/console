#!/usr/bin/env python3
import pathlib,subprocess,uuid
root=pathlib.Path('/private/tmp/console-production-tdd-preparation-20260913')
head=subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip()
evidence='/private/tmp/account-fence37-'+uuid.uuid4().hex
print(evidence,flush=True)
raise SystemExit(subprocess.run(['bash','/private/tmp/account-fence37-driver.sh',str(root),head,evidence],cwd=root).returncode)
