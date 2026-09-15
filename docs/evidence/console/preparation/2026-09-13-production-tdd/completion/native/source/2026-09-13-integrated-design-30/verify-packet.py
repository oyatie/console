#!/usr/bin/env python3
"""Static evidence validation only; not SQL, schema-instance, product or load tests."""
import csv, hashlib, json, subprocess
from pathlib import Path
P=Path(__file__).resolve().parent
ROOT=P.parents[4]
count={"json_documents":0,"schema_references":0,"source_blobs":0,"owner_bindings":0,"ui_bindings":0,"fixtures":0}
def require(ok,why):
    if not ok: raise ValueError(why)
def walk(value,file):
    if isinstance(value,dict):
        if "$ref" in value:
            spec=value["$ref"]; fn,_,pointer=spec.partition("#")
            target=(file.parent/fn).resolve() if fn else file
            require(target.is_relative_to(ROOT),"reference outside repository")
            data=json.loads(target.read_text())
            for key in pointer.lstrip("/").split("/") if pointer else []:
                data=data[key.replace("~1","/").replace("~0","~")]
            count["schema_references"]+=1
        for child in value.values():walk(child,file)
    elif isinstance(value,list):
        for child in value:walk(child,file)
for file in sorted(P.glob("*.json")):
    data=json.loads(file.read_text());count["json_documents"]+=1;walk(data,file)
sources=json.loads((P/"source-identities.json").read_text())
for source in sources["sources"]:
    path=source["path"]
    mode=subprocess.check_output(["git","ls-tree",sources["base_sha"],"--",path],cwd=ROOT,text=True).split()
    require(len(mode)>=3 and mode[0] in ("100644","100755") and mode[1]=="blob","nonregular/missing source "+path)
    raw=subprocess.check_output(["git","show",sources["base_sha"]+":"+path],cwd=ROOT)
    require(mode[2]==source["git_blob"] and len(raw)==source["bytes"] and hashlib.sha256(raw).hexdigest()==source["sha256"],"source drift "+path)
    count["source_blobs"]+=1
bindings=json.loads((P/"owner-bindings.json").read_text())["bindings"]
require(len({b["key"] for b in bindings})==len(bindings),"duplicate owner key")
for b in bindings:
    require("." not in b["local_key"],"multi-segment local key")
    for field in ("input","result"):
        spec=b[field];fn,_,frag=spec.partition("#");data=json.loads((P/fn).read_text())
        for key in frag.lstrip("/").split("/"):data=data[key]
count["owner_bindings"]=len(bindings)
ui=list(csv.DictReader((P/"action-bindings.csv").open()))
require(all(r["registered_owner_action"] in {b["key"] for b in bindings} for r in ui),"unbound UI mutation")
require(any(r["ui_key"]=="REOPEN_INPUTS" for r in ui),"missing reopen path")
count["ui_bindings"]=len(ui)
for name,column in (("source-census.csv","path"),("source-sink-map.csv","path"),("actor-migration.csv","source")):
    for row in csv.DictReader((P/name).open()):require((ROOT/row[column]).is_file(),"missing source "+row[column])
fixtures=json.loads((P/"fixtures.json").read_text())["fixtures"]
require(len({x["id"] for x in fixtures})==len(fixtures),"duplicate fixture")
count["fixtures"]=len(fixtures)
state=json.loads((P/"readiness.json").read_text())
require(not state["whole_design_approved"] and not state["implementation_authorized"],"working candidate cannot self-approve")
require(state["product_tests_executed"]==state["load_experiments_executed"]==state["verus_proofs_executed"]==0 and not state["sql_executed"],"unsupported execution claim")
for item in state["items"]:require((P/item["artifact"]).is_file(),"missing closure artifact")
run=json.loads((P/"run-manifest.json").read_text())
require(run["result"]=="NOT_ASSESSABLE","unexecuted run cannot pass")
require(all(v["pg_type"]!="UNRESOLVED" for r in json.loads((P/"storage-mapping.json").read_text())["records"] for v in r["columns"]),"untyped physical field")
print(json.dumps({"status":"STATIC_PACKET_PASS","counts":count,"schema_instance_validation":False,"product_tests_discovered":0,"product_tests_executed":0,"sql_executed":False,"load_experiments_executed":0,"verus_proofs_executed":0},indent=2))
