import copy,unittest
from source_binding_gate import verify,digest,InvalidBinding,classify,run_inputs,verify_dependencies
from pathlib import Path
NAMES=["bind_active_context","project_authorized_result","build_safe_operational_event","admit_bound_egress"]
def fixture():
 body="\n".join("fn "+n+"(a:u64)->u64{a+1}" for n in NAMES).encode();files={"bound.rs":body,"tool":b"pinned-tool","sinks":b"28-original-sinks"};blobs={"blob":body,"caller":b"actual_owner_call"}
 blobs["a"*40+":owner.rs"]=body;blobs["a"*40+":caller.rs"]=blobs["caller"]
 b=dict(source_path="owner.rs",git_blob="blob",file_sha256=digest(body),executable_spans=[[0,len(body)]],executable_sha256=digest(body),bound_source="bound.rs",bound_source_sha256=digest(body),callers=[dict(source_path="caller.rs",git_blob="caller",sha256=digest(blobs["caller"]))],external_relations={k:[] for k in ["loads","policy_decisions","serializer","emitter","transport"]},trust_sites=[])
 p=dict(schema=1,source_commit="a"*40,owners={n:copy.deepcopy(b) for n in NAMES},approved_trust_sites={n:[] for n in NAMES},toolchain={k:dict(path="tool",sha256=digest(files["tool"])) for k in ["verus","rust_verify","z3"]},features=[],sink_manifest="sinks",sink_manifest_sha256=digest(files["sinks"]));p["toolchain"]["rust_version"]="1.98.1"
 for n in NAMES:p["owners"][n]["symbol"]=n
 return p,blobs,files
class Tests(unittest.TestCase):
 def test_valid_binding_checker_control(self):p,b,f=fixture();self.assertTrue(verify(p,b.__getitem__,f.__getitem__))
 def test_absent_owner(self):p,b,f=fixture();p["owners"][NAMES[0]]=None;self.assertRaisesRegex(InvalidBinding,"SOURCE_PREREQUISITE",verify,p,b.__getitem__,f.__getitem__)
 def test_wrong_body(self):p,b,f=fixture();f["bound.rs"]=b"fn toy(){}";self.assertRaises(InvalidBinding,verify,p,b.__getitem__,f.__getitem__)
 def test_changed_tool(self):p,b,f=fixture();f["tool"]=b"changed";self.assertRaises(InvalidBinding,verify,p,b.__getitem__,f.__getitem__)
 def test_new_trust_site(self):p,b,f=fixture();p["owners"][NAMES[0]]["trust_sites"]=["assume:1"];self.assertRaises(InvalidBinding,verify,p,b.__getitem__,f.__getitem__)
 def test_changed_caller(self):p,b,f=fixture();b["caller"]=b"bypass";self.assertRaises(InvalidBinding,verify,p,b.__getitem__,f.__getitem__)
 def test_changed_sinks(self):p,b,f=fixture();f["sinks"]=b"missing sink";self.assertRaises(InvalidBinding,verify,p,b.__getitem__,f.__getitem__)
 def test_detached_owner_blob(self):p,b,f=fixture();b["a"*40+":owner.rs"]=b"different commit source";self.assertRaisesRegex(InvalidBinding,"exact commit/path",verify,p,b.__getitem__,f.__getitem__)
 def test_wrong_negative_location(self):self.assertRaisesRegex(InvalidBinding,"wrong negative",classify,"assertion failed elsewhere.rs:1\nverification results:: 3 verified, 1 errors",1,True,"intended.rs:12")
 def test_positive_completed(self):self.assertEqual(classify("verification results:: 4 verified, 0 errors",0)["verified"],4)
 def test_actual_assertion_negative(self):self.assertEqual(classify("assertion failed\nverification results:: 3 verified, 1 errors",1,True)["failed_obligations"],1)
 def test_parse_is_not_negative(self):self.assertRaises(InvalidBinding,classify,"unresolved import\nverification results:: 3 verified, 1 errors",1,True)
 def test_empty_is_not_proof(self):self.assertRaises(InvalidBinding,classify,"verification results:: 0 verified, 0 errors",0)


class ExecutedSourceBindingTests(unittest.TestCase):
 def prepared(self):
  p,b,f=fixture();h=Path(__file__).with_name("proof-obligations.rs").read_bytes();f["proof-obligations.rs"]=h
  p.update(proof_entry="proof-obligations.rs",proof_input_closure={"bound.rs":digest(f["bound.rs"]),"proof-obligations.rs":digest(h)},approved_harness_sha256=digest(h))
  run=dict(name="positive",source="proof-obligations.rs",mutation=None,input_sha256=p["proof_input_closure"].copy())
  return p,b,f,run
 def test_positive_uses_verified_owner_extraction_and_frozen_harness(self):
  p,b,f,r=self.prepared();self.assertTrue(verify(p,b.__getitem__,f.__getitem__));self.assertEqual(run_inputs(p,r,f.__getitem__)["bound.rs"],f["bound.rs"])
 def test_real_binding_with_unrelated_positive_proof_is_rejected(self):
  p,b,f,r=self.prepared();self.assertTrue(verify(p,b.__getitem__,f.__getitem__));f["toy.rs"]=b"fn toy(){}";r["source"]="toy.rs"
  self.assertRaisesRegex(InvalidBinding,"unrelated proof entry",run_inputs,p,r,f.__getitem__)
 def test_rehashed_toy_harness_is_rejected(self):
  p,b,f,r=self.prepared();f["proof-obligations.rs"]=b"fn toy(){}";pin=digest(f["proof-obligations.rs"]);p["approved_harness_sha256"]=pin;p["proof_input_closure"]["proof-obligations.rs"]=pin;r["input_sha256"]["proof-obligations.rs"]=pin
  self.assertRaisesRegex(InvalidBinding,"harness substitution",run_inputs,p,r,f.__getitem__)
 def mutant(self):
  p,b,f,r=self.prepared();body=f["bound.rs"];start=body.index(b"a+1");changed=body[:start]+b"a+2"+body[start+3:]
  recipe=dict(path="bound.rs",span=[start,start+3],before_hex=b"a+1".hex(),after_hex=b"a+2".hex(),result_sha256=digest(changed),diagnostic_anchor="reviewed:12")
  p["approved_mutations"]={"wrong_company":recipe};r.update(name="wrong_company",mutation=copy.deepcopy(recipe),diagnostic_anchor="reviewed:12");r["input_sha256"]["bound.rs"]=digest(changed)
  return p,b,f,r,changed
 def test_negative_exact_derivation_is_checked(self):
  p,b,f,r,changed=self.mutant();self.assertEqual(run_inputs(p,r,f.__getitem__)["bound.rs"],changed)
 def test_mislabeled_mutant_is_rejected(self):
  p,b,f,r,_=self.mutant();r["mutation"]["after_hex"]=b"a+3".hex()
  self.assertRaisesRegex(InvalidBinding,"mislabeled",run_inputs,p,r,f.__getitem__)
 def test_wrong_negative_anchor_and_missing_owner_file_refuse(self):
  p,b,f,r,_=self.mutant();r["diagnostic_anchor"]="different:1";self.assertRaises(InvalidBinding,run_inputs,p,r,f.__getitem__)
  p,b,f,r=self.prepared();del p["proof_input_closure"]["bound.rs"];self.assertRaisesRegex(InvalidBinding,"owner extraction omitted",run_inputs,p,r,f.__getitem__)
 def test_compiler_dependency_closure_requires_real_owner_consumed(self):
  self.assertTrue(verify_dependencies("proof.d: /tmp/proof-bound/proof-obligations.rs /tmp/proof-bound/bound.rs\n", "/tmp/proof-bound",{"proof-obligations.rs","bound.rs"}))
  self.assertRaises(InvalidBinding,verify_dependencies,"proof.d: /tmp/proof-bound/proof-obligations.rs\n","/tmp/proof-bound",{"proof-obligations.rs","bound.rs"})
  self.assertRaises(InvalidBinding,verify_dependencies,"proof.d: /tmp/proof-bound/proof-obligations.rs /private/other.rs\n","/tmp/proof-bound",{"proof-obligations.rs","bound.rs"})
 def test_empty_compiler_dependencies_refuse(self):
  self.assertRaises(InvalidBinding,verify_dependencies,"","/tmp/proof-bound",{"proof-obligations.rs","bound.rs"})

if __name__=="__main__":unittest.main()
