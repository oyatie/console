"""Scratch reference oracle for proposed native28 framing. Not production execution."""
import hashlib, json, struct, uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parent
SOURCE=ROOT/"source"
LIMIT=65536
INPUT_TAG=b"console.payroll.native28.input-stream.v1\0"
CALC_TAG=b"console.payroll.native28.calculation-stream.v1\0"
OWNER_TAG=b"console.owner28.submission.v1\0"
class Reject(ValueError): pass
def require(ok, why):
    if not ok: raise Reject(why)
def u32(n): return struct.pack(">I",n)
def sha(b): return hashlib.sha256(b).hexdigest()
def canonical(value):
    def visit(v,depth):
        require(depth<=8,"depth")
        if isinstance(v,dict):
            require(len(v)<=128,"fields")
            for k,x in v.items():
                require(k.isascii() and "\0" not in k,"key")
                visit(x,depth+isinstance(x,(list,dict)))
        elif isinstance(v,list):
            require(len(v)<=256,"list")
            for x in v:visit(x,depth+isinstance(x,(list,dict)))
        elif isinstance(v,str):require("\0" not in v,"nul")
        else:require(v is None or isinstance(v,bool),"JSON number")
    visit(value,1)
    try:return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(",",":"),allow_nan=False).encode("utf-8")
    except (UnicodeError,ValueError) as e:raise Reject(str(e))
def parse_json(raw):
    def pairs(xs):
        out={}
        for k,v in xs:
            require(k not in out,"duplicate key");out[k]=v
        return out
    try:v=json.loads(raw.decode("utf-8"),object_pairs_hook=pairs)
    except (UnicodeError,ValueError) as e:raise Reject(str(e))
    require(canonical(v)==raw,"not canonical")
    return v
def h18(kind,value):return sha(("console.payroll.native18."+kind).encode()+b"\0"+canonical(value))
def component(v):
    b=canonical(v);require(len(b)<=LIMIT,"leaf cap");return u32(len(b))+b
def schema(v,name):
    # jsonschema is a test dependency, never an implementation-layer suggestion.
    from jsonschema import Draft202012Validator
    from referencing import Registry,Resource
    def retrieve(uri):
        from urllib.parse import urlparse,unquote
        p=Path(unquote(urlparse(uri).path));return Resource.from_contents(json.loads(p.read_text()))
    path=SOURCE/"2026-09-12-native-manifest-contract-18/native-types.schema.json"
    root=json.loads(path.read_text());root["$id"]=path.as_uri()
    validator=Draft202012Validator({"$ref":path.as_uri()+"#/$defs/"+name},registry=Registry(retrieve=retrieve).with_resource(path.as_uri(),Resource.from_contents(root)))
    errors=list(validator.iter_errors(v));require(not errors,"schema "+name+": "+("; ".join(str(e.validator)+" at /"+"/".join(map(str,e.path)) for e in errors[:2])))
    def scalar_walk(x,s):
        if "$ref" in s:
            k=s["$ref"].split("/")[-1]
            if k in {"PositiveRevision","RevisionOrZero","UnixMicroseconds","CalculationRevision","Ordinal","Hours","Count"}:
                n=int(x);require(-(2**63)<=n<2**63,"checked i64")
                if k=="CalculationRevision":require(n<=2147483647,"calculation bound")
                if k=="Ordinal":require(n<=255,"source ordinal bound")
            if k=="Date":
                from datetime import date
                try:date.fromisoformat(x)
                except ValueError as e:raise Reject("Gregorian date") from e
            scalar_walk(x,root["$defs"][k]);return
        for op in ("oneOf","anyOf"):
            if op in s:
                for branch in s[op]:
                    check=Draft202012Validator(dict(root,**branch),registry=validator._registry)
                    if check.is_valid(x):scalar_walk(x,branch);break
                return
        for branch in s.get("allOf",[]):scalar_walk(x,branch)
        if isinstance(x,dict):
            for k,z in x.items():
                if k in s.get("properties",{}):scalar_walk(z,s["properties"][k])
        if isinstance(x,list):
            for z in x:scalar_walk(z,s.get("items",{}))
    scalar_walk(v,root["$defs"][name])
def input_frame(header,pairs):
    return INPUT_TAG+component(header)+bytes.fromhex(h18("input-header",header))+u32(len(pairs))+b"".join(u32(i)+component(s)+component(c) for i,(s,c) in enumerate(pairs))
def calc_frame(header,pairs):
    return CALC_TAG+component(header)+bytes.fromhex(h18("batch-header",header))+u32(len(pairs))+b"".join(u32(i)+component(c)+component(o) for i,(c,o) in enumerate(pairs))
def manifest18(header,rows,calculation=False):
    tag="calculation-batch" if calculation else "input-manifest"
    hk="batch-header" if calculation else "input-header"
    rk="calculation-outcome" if calculation else "unit-custody"
    pre=("console.payroll.native18."+tag).encode()+b"\0"+bytes.fromhex(h18(hk,header))+u32(len(rows))
    return sha(pre+b"".join(uuid.UUID(r["employee_id"]).bytes+bytes.fromhex(h18(rk,r)) for r in rows))
class Reader:
    def __init__(self,raw):self.raw=raw;self.pos=0
    def take(self,n):
        require(0<=n<=len(self.raw)-self.pos,"truncated")
        b=self.raw[self.pos:self.pos+n];self.pos+=n;return b
    def n(self):return struct.unpack(">I",self.take(4))[0]
    def leaf(self):
        n=self.n();require(n<=LIMIT,"leaf cap before read");return parse_json(self.take(n))
def validate_blockers(values):
    encoded=[canonical(x) for x in values]
    require(encoded==sorted(set(encoded)),"blocker canonical set")
    for blocker in values:
        slots=[(x["kind"],x["selection"],int(x["ordinal"])) for x in blocker["source_slots"]]
        require(slots==sorted(set(slots)),"blocker source_slots canonical set")
def decode_native(raw,calculation=False,input_pairs=None,input_header=None):
    tag=CALC_TAG if calculation else INPUT_TAG
    # Derived custody cap: 4 UUIDs, i64 positive revision, hex64, fixed closed keys.
    max_custody={"org_id":"ffffffff-ffff-4fff-bfff-ffffffffffff","run_id":"ffffffff-ffff-4fff-bfff-ffffffffffff","input_revision":"9223372036854775807","employee_id":"ffffffff-ffff-4fff-bfff-ffffffffffff","line_id":"ffffffff-ffff-4fff-bfff-ffffffffffff","semantic_digest":"f"*64}
    custody_cap=len(canonical(max_custody))
    require(len(raw)<=len(tag)+4+LIMIT+32+4+2000*(12+LIMIT+custody_cap),"whole stream cap")
    r=Reader(raw);require(r.take(len(tag))==tag,"tag")
    header=r.leaf();schema(header,"BatchHeader" if calculation else "InputHeader")
    require(r.take(32).hex()==h18("batch-header" if calculation else "input-header",header),"header digest")
    n=r.n();require(1<=n<=2000,"unit count")
    if not calculation:
        families=[w["family"] for w in header["guard_witnesses"]]
        require(families==sorted({"ARTIFACT","ATTENDANCE","CALENDAR","ELIGIBILITY","EMPLOYMENT","LEAVE","POPULATION","QUALIFICATION","TAX_EVIDENCE","WAGE"}),"guard witness canonical family set")
    require(str(n)==header["member_count" if calculation else "membership_count"],"header count")
    rows=[];previous=None
    for i in range(n):
        require(r.n()==i,"ordinal");a=r.leaf();b=r.leaf()
        c,o=(a,b) if calculation else (b,None)
        schema(c,"UnitCustody");require(len(canonical(c))<=custody_cap,"custody cap")
        for k in ("org_id","run_id","input_revision"):require(c[k]==header[k],"custody "+k)
        employee=uuid.UUID(c["employee_id"]).bytes
        require(previous is None or previous<employee,"employee ordering");previous=employee
        if calculation:
            schema(o,"CalculationOutcome")
            validate_blockers(o["blockers"])
            for k in ("org_id","run_id","input_revision","employee_id","line_id"):require(o[k]==c[k],"outcome "+k)
            require(o["calculation_revision"]==header["calculation_revision"],"calculation revision")
            require(o["input_unit_digest"]==h18("unit-custody",c),"outcome custody digest")
            require(input_pairs is not None and i<len(input_pairs) and c==input_pairs[i][1],"exact closed input member")
        else:
            schema(a,"UnitSemantic")
            refs=a["basis"]["source_refs"]
            keys=[(x["kind"],x["selection"],int(x["ordinal"])) for x in refs]
            require(keys==sorted(keys) and len(keys)==len(set(keys)),"source reference canonical slot order")
            require(len({canonical(x) for x in refs})==len(refs),"duplicate full source ref")
            validate_blockers(a["basis"]["blockers"])
            for kind in {x["kind"] for x in refs}:
                for selection in {x["selection"] for x in refs if x["kind"]==kind}:
                    chosen=[x for x in refs if x["kind"]==kind and x["selection"]==selection]
                    if selection=="CANDIDATE":
                        require([canonical(x["ref"]) for x in chosen]==sorted(canonical(x["ref"]) for x in chosen),"candidate ref canonical ordering")
                        require([int(x["ordinal"]) for x in chosen]==list(range(1,len(chosen)+1)),"candidate ordinal sequence")
                    elif kind=="EMPLOYMENT_REVISION":
                        require([int(x["ordinal"]) for x in chosen]==list(range(1,len(chosen)+1)),"employment ordinal sequence")
                        require([int(x["ref"]["from_us"]) for x in chosen]==sorted(int(x["ref"]["from_us"]) for x in chosen),"employment temporal ordering")
                    else:require(len(chosen)==1 and chosen[0]["ordinal"]=="0","singleton selected ordinal")
            require(a["org_id"]==c["org_id"] and a["employee_id"]==c["employee_id"] and a["context"]==header["context"],"semantic subject/context")
            require(c["semantic_digest"]==h18("unit-semantic",a),"semantic digest")
        rows.append((a,b))
    require(r.pos==len(raw),"trailing bytes")
    if calculation:
        require(input_header is not None,"closed input header required")
        require(header["input_manifest_digest"]==manifest18(input_header,[c for s,c in input_pairs]),"closed input manifest digest")
        require(len(input_pairs)==n,"closed input member count")
        successes=sum(o["state"]=="SUCCESS" for c,o in rows)
        require(header["success_count"]==str(successes) and header["blocked_count"]==str(n-successes),"outcome counts")
    digest=manifest18(header,[b for a,b in rows],calculation)
    return header,rows,digest


def normalize_typed(value):
    # Call only AFTER validating the concrete closed RegisteredInput schema and
    # resolving CONTROL_REFERENCE. No raw-number admission is implied.
    if isinstance(value,bool) or value is None:return value
    if isinstance(value,int):return str(value)
    if isinstance(value,list):return [normalize_typed(x) for x in value]
    if isinstance(value,dict):
        return {k:({a:normalize_typed(b) for a,b in v.items() if a!="nonce"} if k=="editing_token" else normalize_typed(v)) for k,v in value.items()}
    return value

def canonical_command(value):
    action=value.get("owner_action")
    def walk(v,path,depth):
        require(depth<=8,"depth")
        if isinstance(v,dict):
            require(len(v)<=128,"fields")
            for k,x in v.items():
                require(k.isascii() and "\0" not in k,"key");walk(x,path+(k,),depth+isinstance(x,(dict,list)))
        elif isinstance(v,list):
            cap=2000 if action=="payroll.review.release" and path==("body","targets") else 256
            require(len(v)<=cap,"list")
            for i,x in enumerate(v):walk(x,path+(str(i),),depth+isinstance(x,(dict,list)))
        elif isinstance(v,str):require("\0" not in v,"nul")
        else:require(v is None or isinstance(v,bool),"JSON number")
    walk(value,(),1)
    return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(",",":"),allow_nan=False).encode("utf-8")

def registered_digest(kind,normalized_body):
    # Use same ordered writer with only this registration's targets exception.
    wrapped={"owner_action":kind,"body":normalized_body}
    canonical_command(wrapped) # checks the payload's bounded shape
    raw=json.dumps({"kind":kind,"body":normalized_body},ensure_ascii=False,sort_keys=True,separators=(",",":"),allow_nan=False).encode()
    return sha(b"console.action30.registered-input\0"+raw)

def validate_normalized(value,path,fragment):
    from jsonschema import Draft202012Validator,FormatChecker,ValidationError,validators
    from referencing import Registry,Resource
    from urllib.parse import urlparse,unquote
    def minimum(validator,bound,v,schema):
        if isinstance(v,str):
            try:
                if int(v)<bound:yield ValidationError("checked decimal minimum")
            except ValueError:yield ValidationError("checked decimal")
    def maximum(validator,bound,v,schema):
        if isinstance(v,str):
            try:
                if int(v)>bound:yield ValidationError("checked decimal maximum")
            except ValueError:yield ValidationError("checked decimal")
    Checker=validators.extend(Draft202012Validator,{"x-decimal-min":minimum,"x-decimal-max":maximum})
    def retrieve(uri):
        p=Path(unquote(urlparse(uri).path)).resolve()
        require(p.is_relative_to(ROOT/"normalized-source"),"schema custody")
        d=json.loads(p.read_text());d["$id"]=p.as_uri();return Resource.from_contents(d)
    p=(ROOT/"normalized-source"/path).resolve()
    errors=list(Checker({"$ref":p.as_uri()+"#"+fragment},registry=Registry(retrieve=retrieve),format_checker=FormatChecker()).iter_errors(value))
    require(not errors,"normalized schema: "+("; ".join(str(e.validator)+" at /"+"/".join(map(str,e.path)) for e in errors[:2])))

def canonical_uuid(x):
    import re
    return isinstance(x,str) and re.fullmatch(r"[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}",x) is not None and uuid.UUID(x).int!=0

def positive_decimal(x):
    import re
    return isinstance(x,str) and re.fullmatch(r"[1-9][0-9]{0,18}",x) is not None and int(x)<=2**63-1

def check_owner_shape(command):
    bindings=json.loads((SOURCE/"2026-09-13-integrated-design-30/owner-bindings.json").read_text())["bindings"]
    b=next((b for b in bindings if b["key"]==command["owner_action"]),None)
    require(b is not None,"unregistered action")
    require(command["owner_key"]==b["owner"],"fixed owner dispatch")
    for k in ("org_id","command_id","actor_account_id","object_type_id","action_type_id"):
        require(canonical_uuid(command[k]),"command UUID "+k)
    require(positive_decimal(command["action_registration_revision"]),"registration revision")
    import re
    require(isinstance(command["action_registration_manifest_digest"],str) and re.fullmatch("[0-9a-f]{64}",command["action_registration_manifest_digest"]) is not None,"registration digest")
    validate_normalized(command["input_schema_ref"],"2026-09-13-integrated-design-30/action-types.schema.json","/$defs/InputSchemaRef")
    p,fragment=b["input"].split("#")
    path=str((Path("2026-09-13-integrated-design-30")/p))
    validate_normalized(command["body"],path,fragment)
    target=command["target"]
    require(isinstance(target,dict) and target.get("object_kind")==b["registered_object_kind"],"registered target kind")
    if target.get("kind")=="Existing":
        require(set(target)=={"kind","object_kind","object_id"} and canonical_uuid(target["object_id"]),"Existing target")
    elif target.get("kind")=="Create":
        require(set(target)=={"kind","object_kind","allocated_object_id","scope_ref"} and canonical_uuid(target["allocated_object_id"]),"Create target")
        s=target["scope_ref"]
        require(isinstance(s,dict) and set(s)=={"org_id","object_kind","object_id","revision"} and canonical_uuid(s["org_id"]) and canonical_uuid(s["object_id"]) and positive_decimal(s["revision"]) and isinstance(s["object_kind"],str) and len(s["object_kind"])>0,"Create scope")
    else:raise Reject("target discriminant")
    def company(v):
        if isinstance(v,dict):
            for k,x in v.items():
                if k=="org_id":require(x==command["org_id"],"Company reference binding")
                else:company(x)
        elif isinstance(v,list):
            for x in v:company(x)
    company(command)

COMMAND_FIELDS=set("protocol org_id command_id actor_account_id owner_key owner_action object_type_id action_type_id input_schema_ref codec_version target body expected reason attempt gate_ref action_registration_revision action_registration_manifest_digest".split())
SOURCE_ACTIONS={"calendar.publish","calendar.assign","attendance.resolve_coverage","attendance.create_close_basis","source.propose_qualification","source.verify_qualification","source.revoke_qualification","eligibility.propose","eligibility.verify","eligibility.revoke","payroll.admit_tax_evidence","payroll.verify_tax_evidence","payroll.revoke_tax_evidence"}
def owner_frame(command):
    require(set(command)==COMMAND_FIELDS,"command fields")
    check_owner_shape(command)
    require(command["protocol"]=="CCF1" and command["codec_version"]=="action30-v1","dispatch")
    require(command["owner_action"] not in SOURCE_ACTIONS and not command["owner_action"].startswith("group.") and command["owner_action"]!="attempt.execute","specialized owner")
    require(command["expected"]=={"kind":"BODY_EXACT","registered_input_digest":registered_digest(command["owner_action"],command["body"])},"expected body binding")
    require(command["reason"]==command["body"].get("reason"),"reason equality")
    g=command["gate_ref"]
    require(g is None or set(g)=={"request_id","request_revision"},"stable gate only")
    if g is not None:
        require(canonical_uuid(g["request_id"]),"gate UUID")
        require(g["request_revision"]=="1","initial gate revision")
    a=command["attempt"]
    require(a is None or set(a)=={"attempt_id","draft_id","source_revision_id","editing_epoch","assignment_generation"},"attempt shape")
    if a is not None:
        require(all(canonical_uuid(a[k]) for k in ("attempt_id","draft_id","source_revision_id")) and positive_decimal(a["editing_epoch"]) and (a["assignment_generation"] is None or positive_decimal(a["assignment_generation"])),"attempt scalar fields")
    # Private admission validates actual DIRECT/ATTEMPT custody; bytes cannot prove it.
    raw=canonical_command(command)
    cap=8388608 if command["owner_action"]=="payroll.review.release" else 65536
    frame=OWNER_TAG+u32(len(raw))+raw
    require(len(frame)<=cap,"whole owner frame cap")
    return frame

def decode_owner(raw):
    r=Reader(raw);require(r.take(len(OWNER_TAG))==OWNER_TAG,"owner tag")
    n=r.n();require(n<=8388608-len(OWNER_TAG)-4,"owner cap before read")
    b=r.take(n);require(r.pos==len(raw),"owner trailing")
    def pairs(xs):
        d={}
        for k,v in xs:require(k not in d,"duplicate key");d[k]=v
        return d
    try:c=json.loads(b.decode("utf-8"),object_pairs_hook=pairs)
    except (UnicodeError,ValueError) as e:raise Reject(str(e))
    require(owner_frame(c)==raw,"owner canonical")
    return c


def validate_prepared_input(prepared,raw,selector):
    require(selector=={"kind":"NATIVE_INPUT_STREAM","command":prepared["preparation_command"]},"stream selector")
    h,p,d=decode_native(raw)
    require(prepared["header"]==h and prepared["unit_count"]==len(p) and prepared["manifest_digest"]==d,"prepared metadata")
    require(prepared["preparation_command"]=={"org_id":h["org_id"],"command_id":h["command_id"]},"producer command")
    a=prepared["semantic_manifest"];b=prepared["custody_manifest"]
    require(a==b,"paired stream EvidenceRef aliases")
    require(a["evidence_digest"]==sha(raw) and a["encoded_size"]==len(raw),"evidence digest/size")
    require(a["unit"]["org_id"]==h["org_id"],"unit company")
    identity=sha(b"console.retained.native28.stream-identity\0"+canonical(selector))
    require(a["unit"]["identity_digest"]==identity,"stream identity digest")
    return h,p
