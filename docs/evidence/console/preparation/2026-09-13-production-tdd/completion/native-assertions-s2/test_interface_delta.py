"""Independent schema-delta fixtures only; not Rust/application execution."""
import json, unittest
from pathlib import Path
from jsonschema import Draft202012Validator
ROOT=Path(__file__).parent
DELTA=json.loads((ROOT/'mime-contract-delta.json').read_text())
class MimeDelta(unittest.TestCase):
 def validate(self,value):
  schema={'type':'object','properties':{'mime_type':{'enum':DELTA['replacement']}},'required':['mime_type'],'additionalProperties':False}
  return list(Draft202012Validator(schema).iter_errors(value))
 def test_old_three_are_preserved(self):
  self.assertEqual(DELTA['replacement'][:3],DELTA['expected'])
  for mime in DELTA['expected']:self.assertFalse(self.validate({'mime_type':mime}))
 def test_exact_source19_ndjson_is_allowed(self):self.assertFalse(self.validate({'mime_type':'application/x-ndjson'}))
 def test_raw_jwt_is_not_mislabeled_or_allowed(self):self.assertTrue(self.validate({'mime_type':'application/jwt'}))
 def test_no_wildcards(self):
  for mime in ['*/*','application/*']:self.assertTrue(self.validate({'mime_type':mime}))
 def test_no_case_or_parameter_alias(self):
  for mime in ['Application/x-ndjson','application/x-ndjson; charset=utf-8','application/json\n']:
   self.assertTrue(self.validate({'mime_type':mime}))
 def test_no_unknown_type(self):self.assertTrue(self.validate({'mime_type':'application/octet-stream'}))
 def test_exact_member_required(self):self.assertTrue(self.validate({}))
 def test_no_extra_member(self):self.assertTrue(self.validate({'mime_type':'application/x-ndjson','verified':True}))
 def test_no_coercion(self):
  for value in [None,4,True,[]]:self.assertTrue(self.validate({'mime_type':value}))
if __name__=='__main__':unittest.main(verbosity=2)
