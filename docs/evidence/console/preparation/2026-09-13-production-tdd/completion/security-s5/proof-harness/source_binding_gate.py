"""Exact source/TCB/proof harness checker; no product body or success simulator.
Absent production binding is SOURCE_PREREQUISITE, never failed proof obligation.
"""
import hashlib,json,pathlib,re,subprocess,sys,tempfile,shlex
class InvalidBinding(ValueError):pass
def need(value,reason):
 if not value:raise InvalidBinding(reason)
def digest(data):return hashlib.sha256(data).hexdigest()
def verify(packet,read_blob,read_file):
 need(packet["schema"]==1,"binding schema")
 need(set(packet["owners"])=={"bind_active_context","project_authorized_result","build_safe_operational_event","admit_bound_egress"},"exact four owner obligations")
 need(packet["source_commit"] and len(packet["source_commit"])==40,"source commit pin")
 for name,binding in packet["owners"].items():
  need(binding is not None,"SOURCE_PREREQUISITE: "+name)
  blob=read_blob(binding["git_blob"])
  need(blob==read_blob(packet["source_commit"]+":"+binding["source_path"]),"owner blob not at exact commit/path")
  need(digest(blob)==binding["file_sha256"],"source file bytes changed")
  spans=binding["executable_spans"]
  need(spans and all(0<=a<b<=len(blob) for a,b in spans),"actual executable spans")
  need(spans==sorted(spans) and all(spans[n][1]<=spans[n+1][0] for n in range(len(spans)-1)),"overlap or reordered source spans")
  body=b"\n".join(blob[a:b] for a,b in spans)
  need(digest(body)==binding["executable_sha256"],"actual executable body/struct identity changed")
  need(binding["symbol"]==name,"unreviewed owner rename")
  need(re.search(rb"\bfn\s+"+name.encode()+rb"\s*(?:<|\()",body) is not None,"owner symbol absent from exact executable binding")
  extracted=read_file(binding["bound_source"])
  need(digest(extracted)==binding["bound_source_sha256"],"bound source file changed")
  # Reviewer-approved extraction spans include every struct field and executable
  # body byte. Ghost spec insertion may occur only outside these exact segments.
  cursor=0
  for a,b in spans:
   at=extracted.find(blob[a:b],cursor);need(at>=0,"toy/replaced executable body");cursor=at+b-a
  need(binding["callers"],"source has no real caller binding")
  for caller in binding["callers"]:
   actual=read_blob(caller["git_blob"]);need(actual==read_blob(packet["source_commit"]+":"+caller["source_path"]),"caller blob not at exact commit/path");need(digest(actual)==caller["sha256"],"caller source changed")
  need(set(binding["external_relations"])=={"loads","policy_decisions","serializer","emitter","transport"},"incomplete external relation census")
  need(binding["trust_sites"]==packet["approved_trust_sites"][name],"unreviewed assume/external/unsafe/FFI change")
  # Exact file hashes bind trust-site census to reviewed source; this is not a
  # claim that text search proves a complete compiler/FFI security boundary.
 for kind in ["verus","rust_verify","z3"]:
  pin=packet["toolchain"][kind];need(digest(read_file(pin["path"]))==pin["sha256"],"tool pin changed: "+kind)
 need(packet["toolchain"]["rust_version"] and packet["features"] is not None,"toolchain/features missing")
 need(packet["sink_manifest_sha256"]==digest(read_file(packet["sink_manifest"])),"sink enrollment changed")
 return True

REQUIRED_RUNS={"positive", "wrong_company", "missing_field_filter", "extra_safe_event", "unbound_egress", "deny_valid_a"}
def relative(path):
 p=pathlib.PurePosixPath(path)
 need(path and not p.is_absolute() and ".." not in p.parts and str(p)==path,"noncanonical proof input path")
 return path

def run_inputs(packet,run,read_file):
 """Build exact compiler inputs; source paths are never independently substitutable."""
 closure=packet["proof_input_closure"]
 entry=relative(packet["proof_entry"])
 need(entry=="proof-obligations.rs","unreviewed harness entry")
 need(entry in closure,"entry absent from reviewed input closure")
 need(run["source"]==entry,"unrelated proof entry")
 bound={b["bound_source"] for b in packet["owners"].values()}
 need(bound <= set(closure),"owner extraction omitted from compiler closure")
 inputs={}
 for path,pin in closure.items():
  relative(path);data=read_file(path)
  need(digest(data)==pin,"reviewed proof closure changed")
  inputs[path]=data
 need(digest(inputs[entry])==packet["approved_harness_sha256"]==digest(pathlib.Path(__file__).with_name("proof-obligations.rs").read_bytes()),"unreviewed harness substitution")
 if run["name"]=="positive":
  need(run.get("mutation") is None,"positive must use unmodified owner extraction")
 else:
  approved=packet["approved_mutations"][run["name"]]
  need(run["mutation"]==approved,"mislabeled or unreviewed negative recipe")
  need(run["diagnostic_anchor"]==approved["diagnostic_anchor"],"negative diagnostic changed from reviewed recipe")
  path=relative(approved["path"])
  need(path in bound,"negative changes harness or unbound file instead of executable owner/caller extraction")
  original=inputs[path];start,end=approved["span"]
  before=bytes.fromhex(approved["before_hex"]);after=bytes.fromhex(approved["after_hex"])
  need(0<=start<end<=len(original) and original[start:end]==before and before!=after,"negative patch preimage/span")
  inputs[path]=original[:start]+after+original[end:]
  need(digest(inputs[path])==approved["result_sha256"],"negative recipe result mismatch")
 need(set(run["input_sha256"])==set(inputs),"run omits or adds compiler inputs")
 for path,data in inputs.items():need(digest(data)==run["input_sha256"][path],"executed input differs from extraction/recipe")
 return inputs

def verify_dependencies(text,workspace,expected):
 """Rust dep-info is an execution observation, never a packet-supplied assertion."""
 # Rust emits a primary rule followed by optional empty phony rules. POSIX source
 # paths are emitted with Make escaping. Refuse ambiguous/malformed output.
 lines=text.replace("\\\n", " ").splitlines()
 need(lines,"compiler dependency output absent")
 rule=lines[0]
 need(": " in rule,"compiler dependency output absent or malformed")
 words=shlex.split(rule.split(": ",1)[1])
 need(words,"compiler dependency closure empty")
 root=pathlib.Path(workspace).resolve();observed=set()
 for word in words:
  path=pathlib.Path(word);path=(root/path).resolve() if not path.is_absolute() else path.resolve()
  try:rel=str(path.relative_to(root))
  except ValueError:raise InvalidBinding("unreviewed external compiler input")
  observed.add(rel)
 need(observed==set(expected),"executed compiler closure differs from exact reviewed owner/harness inputs")
 return True

def classify(stdout,code,expected_failure=False,diagnostic_anchor=None):
 m=re.search(r"verification results::\s*(\d+) verified,\s*(\d+) errors",stdout)
 need(m is not None,"not a completed verifier result")
 verified,failed=map(int,m.groups())
 need(verified>0,"no actual verified unit")
 if expected_failure:
  if diagnostic_anchor is not None:need(diagnostic_anchor in stdout,"wrong negative obligation location")
  need(code!=0 and failed>0 and ("assertion failed" in stdout or "postcondition not satisfied" in stdout),"not a designated assertion/postcondition failure")
  need(not any(x in stdout for x in ["unresolved import","cannot find", "error: expected", "unsupported option"]),"toolchain/parse failure is not negative proof")
 else:need(code==0 and failed==0,"positive source proof incomplete")
 return {"verified":verified,"failed_obligations":failed,"exit":code}

def main():
 packet_path=pathlib.Path(sys.argv[1]).resolve();root=packet_path.parent;packet=json.loads(packet_path.read_text())
 checkout=pathlib.Path(sys.argv[2]).resolve()
 actual=subprocess.check_output(["git","-C",str(checkout),"rev-parse","HEAD"],text=True).strip();need(actual==packet["source_commit"],"source checkout head differs")
 def read_file(path):return (root/path).read_bytes()
 def read_blob(blob):return subprocess.check_output(["git","-C",str(checkout),"cat-file","blob",blob])
 verify(packet,read_blob,read_file)
 runs=packet["proof_runs"]
 need(len(runs)==len(REQUIRED_RUNS) and {r["name"] for r in runs}==REQUIRED_RUNS,"finite required proof run census")
 need(set(packet["approved_mutations"])==REQUIRED_RUNS-{"positive"},"exact reviewed negative recipe census")
 need(all(r["expected_failure"]==(r["name"]!="positive") for r in runs),"wrong negative classification")
 need(all(r.get("diagnostic_anchor") for r in runs if r["expected_failure"]),"negative obligation anchor missing")
 results=[]
 for run in runs:
  inputs=run_inputs(packet,run,read_file)
  with tempfile.TemporaryDirectory(prefix="console-bound-verus-") as directory:
   workspace=pathlib.Path(directory)
   for path,data in inputs.items():
    target=workspace/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
   depfile=workspace/"compiler-inputs.d"
   command=[str((root/packet["toolchain"]["verus"]["path"]).resolve()),"--crate-type=lib","-V","check-api-safety", "--emit=dep-info="+str(depfile), str(workspace/packet["proof_entry"])]
   for feature in packet["features"]:
    need(re.fullmatch(r"[A-Za-z0-9_-]+",feature) is not None,"invalid Cargo feature")
    command.extend(["--cfg",'feature="'+feature+'"'])
   result=subprocess.run(command,cwd=workspace,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=120)
   (root/(run["name"]+".log")).write_text(result.stdout)
   need(depfile.is_file(),"SOURCE_PREREQUISITE: verifier did not emit actual compiler input closure")
   dependencies=depfile.read_text();verify_dependencies(dependencies,workspace,inputs)
   for path,data in inputs.items():need((workspace/path).read_bytes()==data,"compiler input mutated during run")
   (root/(run["name"]+".dependencies.txt")).write_text(dependencies)
   observation=classify(result.stdout,result.returncode,run["expected_failure"],run.get("diagnostic_anchor"));observation.update(name=run["name"],command=command,input_sha256={p:digest(b) for p,b in inputs.items()})
   results.append(observation)
 print(json.dumps({"source_commit":actual,"results":results,"runtime_tests":0},indent=2))
if __name__=="__main__":main()
