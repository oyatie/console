"""Author-time fixture scaffolding plus explicit workflow choices. Never an acceptance oracle."""
import copy, hashlib, json, pathlib, re, uuid
ROOT=pathlib.Path(__file__).resolve().parent
SRC=ROOT/"source";D30=SRC/"2026-09-13-integrated-design-30"
A=D30/"action-types.schema.json";T=D30/"types.schema.json";S=SRC/"2026-09-12-source-payload-contract-19/source-types.schema.json"
def read(p):return json.loads(p.read_text())
def uid(key):return str(uuid.uuid5(uuid.UUID("11111111-1111-4111-8111-111111111111"),key))
ORG=uid("company");ACCOUNT=uid("original-account")
IDS={k:uid(k) for k in ["org_id","account_id","employee_id","employment_id","person_id","run_id","draft_id","expected_revision_id","source_revision_id","request_id","case_id","task_id","item_id","group_id","operation_id","plan_id","command_id","attempt_id","calculation_id","publication_id","slot_id","cycle_id"]};IDS["org_id"]=ORG;IDS["account_id"]=ACCOUNT
DOCS={}
def resolve(p,s):
 while "$ref" in s:
  f,frag=s["$ref"].split("#");p=(p.parent/f).resolve() if f else p
  if p not in DOCS:DOCS[p]=read(p)
  s=DOCS[p]
  for part in frag.strip("/").split("/"):s=s[part]
 return p,s
def make(p,s,key="value",path=""):
 p,s=resolve(p,s)
 if "const" in s:return s["const"]
 if "enum" in s:return s["enum"][0]
 if "oneOf" in s or "anyOf" in s:
  choices=s.get("oneOf",s.get("anyOf"));null=next((x for x in choices if x.get("type")=="null"),None)
  return None if null is not None else make(p,choices[0],key,path)
 if "allOf" in s:
  out={}
  for x in s["allOf"]:out.update(make(p,x,key,path))
  return out
 t=s.get("type")
 if t=="object":return {k:make(p,s["properties"][k],k,path+"/"+k) for k in s.get("required",s.get("properties",{}))}
 if t=="array":return [make(p,s["items"],key,path+"/"+str(i)) for i in range(s.get("minItems",0))]
 if t=="null":return None
 if t=="boolean":return True
 if t=="integer":return max(1,s.get("minimum",0))
 if t=="string":
  pattern=s.get("pattern","")
  if "[0-9a-f]" in pattern and ("{8}" in pattern or "{4}" in pattern):return IDS.get(key,uid(key))
  if "{64}" in pattern:return hashlib.sha256((path+key).encode()).hexdigest()
  if s.get("format")=="date" or "[0-9]{4}" in pattern:return "2026-09-01"
  if "[0-9]" in pattern and "[A-Za-z" not in pattern and "[A-Z" not in pattern:
   return "1788220800000000" if key.endswith("_us") or key in ("from","until") else "1"
  if s.get("format")=="uuid":return IDS.get(key,uid(key))
  if "ASCII" in s.get("description","") or "[A-Za-z" in pattern or "[A-Z" in pattern:return "NONPRODUCTION_FIXTURE_V1"
  return "검토 근거를 확인했습니다." if key in ("reason","narrative","explanation","description","answer","source_note") else "fixture"
 raise ValueError((p,s,key))
def named(name,p=A):return make(p,{"$ref":"#/$defs/"+name})
B={b["key"]:b for b in read(D30/"owner-bindings.json")["bindings"]}
PAYLOADS={k:make(D30/path.split("#")[0],{"$ref":"#"+path.split("#")[1]}) for k,b in B.items() for path in [b["input"]]}
# Explicit common facts and meaningful positive choices; all fixtures run in isolated clean cells.
period={"start":"2026-09-01","end":"2026-09-30"}
def fix_dates(v):
 if isinstance(v,dict):
  for k,x in v.items():
   if k=="period":v[k]=copy.deepcopy(period)
   elif k in ("pay_date",):v[k]="2026-10-05"
   elif k in ("end","to_date_exclusive","to_exclusive"):v[k]="2026-10-01" if k!="end" else "2026-09-30"
   elif k in ("to_us","until","query_to_us","earning_end_us"):v[k]="1790780400000000"
   elif k in ("from_us","query_from_us","earning_start_us"):v[k]="1788188400000000"
   else:fix_dates(x)
 elif isinstance(v,list):
  for x in v:fix_dates(x)
for v in PAYLOADS.values():fix_dates(v)
for v in PAYLOADS.values():
 if "editing_token" in v:v["editing_token"]["draft_id"]=v["draft_id"]
# Draft patch changes an actual TEXT field and carries no current full-draft body.
p=PAYLOADS["draft.save"];p["patches"]=[{"kind":"SET","value":{"kind":"TEXT","value":"계약 근거 메모"},"address":{"field_id":uid("source-note-field"),"parent_item_ids":[],"item_id":None}}]
PAYLOADS["draft.restore"]["selected_field_ids"]=[uid("source-note-field")]
PAYLOADS["draft.start"]["custody"]={"kind":"ACCOUNT","account_id":ACCOUNT}
# Source24 optional only on source sealing; this positive seal is an ordinary wage command.
PAYLOADS["draft.seal"]["prepared_submission_unit"]=None
PAYLOADS["draft.seal"]["gate_request_intent"]=None
PAYLOADS["payroll.create_contract_wage"].update(amount_won="3000000",monthly_standard_hours=209,subject_source_generation="1")
# Header expectations reflect explicit stages, not an owner-current default.
for k,v in PAYLOADS.items():
 if "expected" in v and isinstance(v["expected"],dict) and "run_id" in v["expected"]:
  e=v["expected"];e.update(input_revision="3",calculation_revision=None,close_basis=None,review_cycle_id=None)
  if k in ("payroll.calculate_run","payroll.open_review_cycle","payroll.supersede_approved_draft"):
   e["close_basis"]=named("AttendanceCloseBasisRef",SRC/"2026-09-12-native-manifest-contract-18/native-types.schema.json")
  if k in ("payroll.open_review_cycle","payroll.supersede_approved_draft"):e["calculation_revision"]="1"
  if k=="payroll.supersede_approved_draft":e["review_cycle_id"]=IDS["cycle_id"]
PAYLOADS["review.decide"]["decision"]="APPROVE"
PAYLOADS["governance.execution_gate.decide"]["decision"]="PERMIT"
PAYLOADS["payroll.review.respond"]["dispositions"]=[{"kind":"ANSWERED","item_id":IDS["item_id"],"evidence":[],"answer":"해당 계산에 사용된 계약 금액을 확인했습니다."}]
# One exact source19 calendar with distinct weekdays, valid dates, retained qualification.
v19=read(SRC/"2026-09-12-source-payload-contract-19/normalization-vectors.json")
cal=copy.deepcopy(v19["vectors"][0]["value"]["payload"])
def reorg(v):
 if isinstance(v,dict):
  for k,x in v.items():
   if k=="org_id":v[k]=ORG
   else:reorg(x)
 elif isinstance(v,list):
  for x in v:reorg(x)
reorg(cal);PAYLOADS["calendar.publish"]=cal
PAYLOADS["calendar.assign"].update(from_date="2026-09-01",to_date_exclusive="2026-10-01",state="ACTIVE")
cov=copy.deepcopy(v19["vectors"][-1]["value"]["payload"]);reorg(cov);cov["employee_id"]=IDS["employee_id"];cov["employment_id"]=IDS["employment_id"];PAYLOADS["attendance.resolve_coverage"]=cov
PAYLOADS["attendance.create_close_basis"]["member_count"]="1"
# Admission registrations select their own discriminated family, never the first union branch accidentally.
sdoc=read(S)["$defs"]
for k,index in [("source.propose_qualification",0),("eligibility.propose",1),("payroll.admit_tax_evidence",2)]:
 v=make(S,sdoc["AdmissionPayload"]["oneOf"][index]);fix_dates(v);PAYLOADS[k]=v
for prefix,index in [("source",0),("eligibility",1),("payroll",2)]:
 vk={"source":"source.verify_qualification","eligibility":"eligibility.verify","payroll":"payroll.verify_tax_evidence"}[prefix];rk={"source":"source.revoke_qualification","eligibility":"eligibility.revoke","payroll":"payroll.revoke_tax_evidence"}[prefix]
 ref=make(S,sdoc["AdmissionRef"]["oneOf"][index]);PAYLOADS[vk]["admission_ref"]=copy.deepcopy(ref);PAYLOADS[rk]["head_ref"]=copy.deepcopy(ref)
# Explicit no-charge consultation is a valid alternative to fabricating an approval charge witness.
PAYLOADS["leave.decide"].update(decision="TIME_CHANGE_CONSULT",charge_resolution=None,consultation_proposal_dates=["2026-09-15"],supporting_evidence=[])
# The upload fixture has real literal bytes and digest, and the worker verifies that immutable version.
UPLOAD=b"employee_id,amount_won\nfixture,3000000\n";PAYLOADS["artifact.request_upload"].update(mime_type="text/csv",expected_bytes=len(UPLOAD),expected_digest=hashlib.sha256(UPLOAD).hexdigest())
(ROOT/"upload.csv").write_bytes(UPLOAD)
# Explicit synthetic source facts use all distinct required scheme/obligation members.
e=PAYLOADS["eligibility.propose"]["payload"]
for x,scheme in zip(e["insurance"],sdoc["InsuranceFact"]["properties"]["scheme"]["enum"]):x["scheme"]=scheme
for x,family in zip(e["obligations"],sdoc["ObligationFact"]["properties"]["family"]["enum"]):x["family"]=family
for name,amount in [("pension_basis","2000000"),("remuneration_basis","3000000"),("taxable_basis","3000000")]:e[name]["amount_won"]=amount
selection=named("TaxSelection",S);selection.update(taxable_monthly_won="3000000",dependent_count_including_self="1",eligible_child_count="0",withholding_option="P100");e["selection"]=copy.deepcopy(selection)
q=named("CredentialPayload",S);q.update(subject_person_id=uid("credentialed-reviewer-person"),valid_from_us="1788188400000000",valid_to_us="1790780400000000",verification_method="SIGNED_ISSUER_RESPONSE");PAYLOADS["source.propose_qualification"]["payload"]=q
for k in ("source.verify_qualification","eligibility.verify","payroll.verify_tax_evidence"):PAYLOADS[k]["reviewer_credential_ref"]=named("QualificationRef",S)
tax=PAYLOADS["payroll.admit_tax_evidence"]["payload"];tax["selection"]=copy.deepcopy(selection);tax["income_band"]={"lower_inclusive_won":"3000000","upper_exclusive_won":"3010000"};tax["amounts"]["income_tax_won"]="0";tax["amounts"]["local_tax_won"]="0"
for k,col in [("income_locator","income_tax"),("local_locator","local_tax")]:tax["amounts"][k]={"kind":"CSV_RECORD","record_ordinal":"1","column_name":col,"locator_schema_ref":named("QualificationRef",S)}
PAYLOADS["payroll.correct_contract_wage"].update(amount_won="3100000",monthly_standard_hours="209")
PAYLOADS["group.operation.create"]["slots"][0]["target"]["object_id"]=IDS["run_id"]
if __name__=="__main__":
 out=ROOT/"payloads";out.mkdir(exist_ok=True)
 for k,v in PAYLOADS.items():(out/(k+".json")).write_text(json.dumps({"kind":k,"body":v},ensure_ascii=False,indent=2)+"\n")
 print(len(PAYLOADS),"authored payload files")
