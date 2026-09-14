"""Author-time framing of frozen18 literals. Golden files are checked in, not regenerated in tests."""
import copy,json
from oracle import *
from author_fixtures import uid,ORG,ACCOUNT,named,B,PAYLOADS
OUT=ROOT/"goldens";OUT.mkdir(exist_ok=True)
a=json.loads((SOURCE/"2026-09-12-native-manifest-contract-18/normalization-vectors.json").read_text());v={x["id"]:x["value"] for x in a["vectors"]}
cases=[("input-blocked-two",False,v["V08"],[(v["V04"],v["V06"]),(v["V05"],v["V10"])],None),("input-resolved-one",False,v["V14"],[(v["V12"],v["V13"])],None),("calculation-blocked-one",True,v["V17"],[(v["V13"],v["V16"])],"input-resolved-one")]
h=copy.deepcopy(v["V17"]);h.update(calculation_revision="2",command_id=uid("successful-calculation-command"),success_count="1",blocked_count="0")
o=copy.deepcopy(v["V16"]);o.update(calculation_revision="2",state="SUCCESS",result_ref={"calculation_id":uid("immutable-result"),"version":"1"},blockers=[])
cases.append(("calculation-success-one",True,h,[(v["V13"],o)],"input-resolved-one"))
index=[]
for name,calc,header,pairs,input_name in cases:
 raw=(calc_frame if calc else input_frame)(header,pairs)
 (OUT/(name+".bin")).write_bytes(raw)
 (OUT/(name+".json")).write_text(json.dumps({"header":header,"pairs":pairs},ensure_ascii=False,indent=2)+"\n")
 index.append({"id":name,"kind":"calculation" if calc else "input","input":input_name,"bytes":len(raw),"sha256":sha(raw),"manifest_digest":manifest18(header,[b for a,b in pairs],calc)})
for name,key,mode,gate in [("owner-direct-wage","payroll.create_contract_wage","DIRECT",False),("owner-attempt-create-run","payroll.create_run","ATTEMPT",False),("owner-gated-calculate","payroll.calculate_run","ATTEMPT",True),("owner-publication","payroll.review.release","ATTEMPT",False),("owner-case-response","payroll.review.respond","ATTEMPT",False)]:
 body=normalize_typed(PAYLOADS[key]);schema=named("InputSchemaRef");b=B[key]
 command={"protocol":"CCF1","org_id":ORG,"command_id":uid(name+"-command"),"actor_account_id":ACCOUNT,"owner_key":b["owner"],"owner_action":key,"object_type_id":uid(b["registered_object_kind"]),"action_type_id":uid(key),"input_schema_ref":schema,"codec_version":"action30-v1","target":{"kind":"Existing","object_kind":b["registered_object_kind"],"object_id":uid("target-"+key)},"body":body,"expected":{"kind":"BODY_EXACT","registered_input_digest":registered_digest(key,body)},"reason":body.get("reason"),"attempt":None,"gate_ref":None,"action_registration_revision":"1","action_registration_manifest_digest":sha(key.encode())}
 if "create" in key:command["target"]={"kind":"Create","object_kind":b["registered_object_kind"],"allocated_object_id":uid("target-"+key),"scope_ref":{"org_id":ORG,"object_kind":"company","object_id":ORG,"revision":"1"}}
 if mode=="ATTEMPT":command["attempt"]={"attempt_id":uid(name+"-attempt"),"draft_id":uid(name+"-draft"),"source_revision_id":uid(name+"-revision"),"editing_epoch":"1","assignment_generation":None}
 if gate:command["gate_ref"]={"request_id":uid(name+"-gate"),"request_revision":"1"}
 raw=owner_frame(command);(OUT/(name+".bin")).write_bytes(raw);(OUT/(name+".json")).write_text(json.dumps(command,ensure_ascii=False,indent=2)+"\n")
 index.append({"id":name,"kind":"owner","bytes":len(raw),"sha256":sha(raw),"command_fingerprint":sha(b"console.command.ccf1\0"+canonical_command(command)),"classification":"byte fixture; actual owner custody and referenced row realization are separate execution prerequisites"})
(OUT/"index.json").write_text(json.dumps(index,indent=2)+"\n")
print(json.dumps(index,indent=2))
