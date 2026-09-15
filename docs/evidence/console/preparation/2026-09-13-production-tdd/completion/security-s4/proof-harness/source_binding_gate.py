"""Exact source/TCB/proof harness checker; no product body or success simulator.
Absent production binding is SOURCE_PREREQUISITE, never failed proof obligation.
"""
import hashlib,json,pathlib,re,subprocess,sys
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
 required={"positive", "wrong_company", "missing_field_filter", "extra_safe_event", "unbound_egress", "deny_valid_a"}
 need(len(runs)==len(required) and {r["name"] for r in runs}==required,"finite required proof run census")
 need(all(r["expected_failure"]==(r["name"]!="positive") for r in runs),"wrong negative classification")
 need(all(r.get("diagnostic_anchor") for r in runs if r["expected_failure"]),"negative obligation anchor missing")
 results=[]
 for run in packet["proof_runs"]:
  source=(root/run["source"]).resolve();need(digest(source.read_bytes())==run["sha256"],"proof input changed")
  command=[str((root/packet["toolchain"]["verus"]["path"]).resolve()),"--crate-type=lib","-V","check-api-safety",str(source)]
  for feature in packet["features"]:
   need(re.fullmatch(r"[A-Za-z0-9_-]+",feature) is not None,"invalid Cargo feature")
   command.extend(["--cfg",'feature="'+feature+'"'])
  result=subprocess.run(command,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=120)
  observation=classify(result.stdout,result.returncode,run["expected_failure"],run.get("diagnostic_anchor"));observation.update(name=run["name"],command=command)
  (root/(run["name"]+".log")).write_text(result.stdout);results.append(observation)
 print(json.dumps({"source_commit":actual,"results":results,"runtime_tests":0},indent=2))
if __name__=="__main__":main()
