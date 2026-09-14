import copy,importlib.util,pathlib,unittest,base64
s=importlib.util.spec_from_file_location("oracle",pathlib.Path(__file__).with_name("logical-observation-oracle.py"));o=importlib.util.module_from_spec(s);s.loader.exec_module(o)
def fixture():
 return dict(complete=True,public_environment={"clock":"same","random_tape":"same"},http=[dict(status=200,headers=[dict(name="content-type",value_base64=base64.b64encode(b"application/json").decode())],body_base64=base64.b64encode(b'allowed').decode(),body_length=7)],ssr=[],cache=[],live=[],events=[],egress=[],allowed_markers=["allowed"],forbidden_markers=["secret"])
class Tests(unittest.TestCase):
 def test_equal_nonvacuous(self):x=fixture();o.paired(x,copy.deepcopy(x))
 def test_private_bytes(self):x=fixture();x["http"][0].update(body_base64=base64.b64encode(b'allowed secret').decode(),body_length=14);self.assertRaises(o.InvalidObservation,o.validate,x)
 def test_extra_sanitized_event(self):x=fixture();y=copy.deepcopy(x);y["events"]=[dict(ordinal=0,code="safe")];self.assertRaises(o.InvalidObservation,o.paired,x,y)
 def test_length(self):x=fixture();x["http"][0]["body_length"]+=1;self.assertRaises(o.InvalidObservation,o.validate,x)
 def test_headers(self):x=fixture();y=copy.deepcopy(x);y["http"][0]["headers"].append(dict(name="etag",value_base64=base64.b64encode(b"hidden").decode()));self.assertRaises(o.InvalidObservation,o.paired,x,y)
 def test_unbound_destination(self):x=fixture();y=copy.deepcopy(x);y["egress"]=[dict(ordinal=0,destination="B",body="allowed")];self.assertRaises(o.InvalidObservation,o.paired,x,y)
 def test_incomplete(self):x=fixture();x["complete"]=False;self.assertRaises(o.InvalidObservation,o.validate,x)
 def test_vacuous(self):x=fixture();x["http"]=[];self.assertRaises(o.InvalidObservation,o.validate,x)
 def test_uncoupled_clock(self):x=fixture();y=copy.deepcopy(x);y["public_environment"]["clock"]="different";self.assertRaises(o.InvalidObservation,o.paired,x,y)
if __name__=="__main__":unittest.main()
