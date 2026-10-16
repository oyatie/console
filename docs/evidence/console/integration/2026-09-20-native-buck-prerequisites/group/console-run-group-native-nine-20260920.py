import os,json,pathlib,subprocess,re,time
root="/private/tmp/console-client-release-20260919"
entries=json.loads(pathlib.Path("/private/tmp/console-group-native-nine-20260920.json").read_text())
results=[]
for label,test,target in entries:
 env=os.environ.copy()
 env["DOCKER_CONTEXT"]="colima-console-release-20260919"
 env["PATH"]="/var/folders/66/4qlvtbgn6r1gl9bvfttp6sww0000gn/T/console-dotslash/bin:"+env["PATH"]
 env["CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT"]=test
 command=["bash","tools/buck/test_needs_postgres.sh","--num-threads=1",target]
 path=pathlib.Path("/private/tmp/console-group-native-"+label+"-20260920.log")
 start=time.monotonic()
 with path.open("w") as log: done=subprocess.run(command,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
 output=path.read_text()
 counts=re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored",output)
 valid=done.returncode==0 and counts==[("1","0","0")] and test+" ... ok" in output
 result={"label":label,"test":test,"command":command,"exit":done.returncode,"counts":counts,"exact_one_executed_and_passed":valid,"seconds":time.monotonic()-start,"log":str(path)}
 results.append(result)
 pathlib.Path("/private/tmp/console-group-native-nine-results-20260920.json").write_text(json.dumps(results,indent=2)+"\n")
 print(label,"PASS" if valid else "FAIL",flush=True)
 if not valid:raise SystemExit(1)
