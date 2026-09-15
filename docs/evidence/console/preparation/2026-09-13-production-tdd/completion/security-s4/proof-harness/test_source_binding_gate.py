import copy,unittest
from source_binding_gate import verify,digest,InvalidBinding,classify
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
if __name__=="__main__":unittest.main()
