#!/usr/bin/env python3
import pathlib, subprocess, uuid
root=pathlib.Path.cwd()
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
evidence='/private/tmp/account-legacy225-capture-'+uuid.uuid4().hex
print(evidence,flush=True)
raise SystemExit(subprocess.run(['bash','/private/tmp/account-legacy225-capture-driver.sh',str(root),head,evidence],cwd=root).returncode)
