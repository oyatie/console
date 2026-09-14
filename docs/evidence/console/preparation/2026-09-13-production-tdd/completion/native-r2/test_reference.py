"""Reference-contract tests only; no product owner execution claims."""
import copy,json,unittest
from oracle import *
from schema_check import validate_payload
G=ROOT/"goldens"
INDEX=json.loads((G/"index.json").read_text())
def pair_input():return json.loads((G/"input-resolved-one.json").read_text())["pairs"]
def header_input():return json.loads((G/"input-resolved-one.json").read_text())["header"]
class Reference(unittest.TestCase):
 def test_duplicate_blocker_source_slots_input_and_calculation(self):
  self.assert_blocker_slots_rejected(True)
 def test_reversed_blocker_source_slots_input_and_calculation(self):
  self.assert_blocker_slots_rejected(False)
 def assert_blocker_slots_rejected(self,duplicate):
  first={"kind":"WAGE","selection":"CANDIDATE","ordinal":"1"};second=copy.deepcopy(first);second["ordinal"]="2"
  slots=[first,copy.deepcopy(first)] if duplicate else [second,first]
  f=json.loads((G/"input-blocked-two.json").read_text());s,c=f["pairs"][0];s["basis"]["blockers"][0]["source_slots"]=slots;c["semantic_digest"]=h18("unit-semantic",s)
  with self.assertRaisesRegex(Reject,"blocker source_slots"):decode_native(input_frame(f["header"],f["pairs"]))
  f=json.loads((G/"calculation-blocked-one.json").read_text());f["pairs"][0][1]["blockers"][0]["source_slots"]=slots
  with self.assertRaisesRegex(Reject,"blocker source_slots"):decode_native(calc_frame(f["header"],f["pairs"]),True,pair_input(),header_input())
 def test_literal_native_offsets(self):
  b=(G/"input-resolved-one.bin").read_bytes()
  self.assertEqual(b[:len(INPUT_TAG)],b"console.payroll.native28.input-stream.v1\0")
  r=Reader(b);r.take(len(INPUT_TAG));h=r.leaf();digest=r.take(32)
  self.assertEqual(digest.hex(),"fffda763df9d4b5b004d2bdfbc98f65c65e9777e8f5e54c75b86abb4b4dcb964")
  self.assertEqual(r.n(),1);self.assertEqual(r.n(),0)
 def test_frozen18_leaves_and_manifest_literals(self):
  a=json.loads((SOURCE/"2026-09-12-native-manifest-contract-18/normalization-vectors.json").read_text())
  for v in a["vectors"]:
   with self.subTest(v["id"]):
    self.assertEqual(canonical(v["value"]).hex(),v["canonical_utf8_hex"])
    self.assertEqual(sha(v["tag"].encode()+b"\0"+bytes.fromhex(v["canonical_utf8_hex"])),v["sha256"])
  for k in ("manifest_frame","resolved_input_frame","batch_frame"):
   self.assertEqual(sha(bytes.fromhex(a[k]["raw_hex"])),a[k]["sha256"])
 def test_frozen19_leaf_literals(self):
  a=json.loads((SOURCE/"2026-09-12-source-payload-contract-19/normalization-vectors.json").read_text())
  for v in a["vectors"]:
   with self.subTest(v["id"]):
    self.assertEqual(canonical(v["value"]).hex(),v["canonical_utf8_hex"])
    self.assertEqual(sha(v["tag"].encode()+b"\0"+bytes.fromhex(v["canonical_utf8_hex"])),v["sha256"])
 def test_frozen24_full_frames(self):
  a=json.loads((SOURCE/"2026-09-13-submission-custody-contract-24/framing-vectors.json").read_text())
  for v in a["vectors"]:
   raw=b"console.source24.submission\0"+b"\1"+component(v["metadata"])+b"\2"+component(v["command"])+b"\3"+component(v["input"])
   self.assertEqual(raw.hex(),v["frame_hex"]);self.assertEqual(sha(raw),v["snapshot_digest"])
   with self.assertRaises(Reject):decode_owner(raw)
 def test_frozen14_command_literals(self):
  a=json.loads((SOURCE/"2026-09-12-common-action-technical-14/fingerprint-vectors.json").read_text())
  for v in a["vectors"]:
   if "preimage_hex" in v:self.assertEqual(sha(bytes.fromhex(v["preimage_hex"])),v["sha256"])
 def test_trailing_resultref_is_rejected(self):
  raw=(G/"calculation-success-one.bin").read_bytes()+component({"calculation_id":"00000000-0000-4000-8000-000000000042","version":"1"})
  with self.assertRaisesRegex(Reject,"trailing"):decode_native(raw,True,pair_input(),header_input())
 def test_custody_cap_arithmetic(self):
  c=copy.deepcopy(pair_input()[0][1]);c["input_revision"]="9223372036854775807"
  self.assertEqual(len(canonical(c)),323)
  self.assertLess(len(CALC_TAG)+4+65536+32+4+2000*(12+65536+323),128*1024*1024)
 def test_source_ordinal255_is_not_stream_bound(self):
  h,p,_=decode_native((G/"input-blocked-two.bin").read_bytes())
  rows=[]
  for i in range(257):
   s,c=copy.deepcopy(p[0]);employee=str(uuid.UUID(int=(0x40008000000000000000+i+1)))
   s["employee_id"]=employee;c["employee_id"]=employee;c["semantic_digest"]=h18("unit-semantic",s);rows.append((s,c))
  h["membership_count"]="257";self.assertEqual(len(decode_native(input_frame(h,rows))[1]),257)
 def test_alias_mapping_rejects_split_evidence(self):
  raw=(G/"input-resolved-one.bin").read_bytes();h=header_input();command={"org_id":h["org_id"],"command_id":h["command_id"]};selector={"kind":"NATIVE_INPUT_STREAM","command":command}
  e={"unit":{"org_id":h["org_id"],"unit_id":"00000000-0000-4000-8000-000000000042","identity_digest":sha(b"console.retained.native28.stream-identity\0"+canonical(selector))},"evidence_digest":sha(raw),"encoded_size":len(raw)}
  p={"header":h,"unit_count":1,"semantic_manifest":e,"custody_manifest":copy.deepcopy(e),"manifest_digest":"9f7b6c4dde388f570f6170d122bd0af48b9cff018af9e4d11598624ca0a9b140","preparation_command":command}
  validate_prepared_input(p,raw,selector)
  p["custody_manifest"]["evidence_digest"]="f"*64
  with self.assertRaisesRegex(Reject,"aliases"):validate_prepared_input(p,raw,selector)
 def test_nonce_excluded_original_epoch_bound(self):
  x={"editing_token":{"draft_id":"d","editing_epoch":"1","assignment_epoch":None,"nonce":"a"*64}}
  y=copy.deepcopy(x);y["editing_token"]["nonce"]="b"*64;self.assertEqual(normalize_typed(x),normalize_typed(y))
  y["editing_token"]["editing_epoch"]="2";self.assertNotEqual(normalize_typed(x),normalize_typed(y))
 def test_owner_gate_future_binding_rejected(self):
  c=json.loads((G/"owner-gated-calculate.json").read_text());c["gate_ref"]["binding_digest"]="f"*64
  with self.assertRaises(Reject):owner_frame(c)
 def test_owner_input_digest_recomputed(self):
  c=json.loads((G/"owner-direct-wage.json").read_text());c["body"]["amount_won"]="3100000"
  with self.assertRaisesRegex(Reject,"expected body"):owner_frame(c)
 def test_publication_only_list_exception(self):
  c=json.loads((G/"owner-publication.json").read_text());c["body"]["targets"]*=257;c["expected"]["registered_input_digest"]=registered_digest(c["owner_action"],c["body"])
  self.assertGreater(len(owner_frame(c)),65536)
  c["owner_action"]="payroll.review.respond"
  with self.assertRaises(Reject):owner_frame(c)
 def test_owner_frame_overflow_counts_tag_and_length(self):
  c=json.loads((G/"owner-direct-wage.json").read_text());c["owner_key"]="ontology";c["owner_action"]="draft.save";c["target"]={"kind":"Existing","object_kind":"action_input_draft","object_id":"00000000-0000-4000-8000-000000000042"}
  c["body"]=normalize_typed(json.loads((ROOT/"payloads/draft.save.json").read_text())["body"])
  c["body"]["patches"]=[{"kind":"SET","value":{"kind":"TEXT","value":"x"*4096},"address":{"field_id":str(uuid.UUID(int=i+1)),"parent_item_ids":[],"item_id":None}} for i in range(17)]
  c["expected"]["registered_input_digest"]=registered_digest(c["owner_action"],c["body"])
  with self.assertRaisesRegex(Reject,"whole owner"):owner_frame(c)
 def test_calc_membership_not_matching_closed_input(self):
  p=pair_input();p[0][1]["line_id"]="00000000-0000-4000-8000-000000000041"
  with self.assertRaisesRegex(Reject,"closed input"):decode_native((G/"calculation-blocked-one.bin").read_bytes(),True,p,header_input())

 def test_review_challenge_duplicate_guard_witnesses(self):
  f=json.loads((G/"input-resolved-one.json").read_text());f["header"]["guard_witnesses"][1]=copy.deepcopy(f["header"]["guard_witnesses"][0])
  with self.assertRaisesRegex(Reject,"guard witness"):decode_native(input_frame(f["header"],f["pairs"]))
 def test_review_challenge_reversed_source_refs_rehashed(self):
  f=json.loads((G/"input-resolved-one.json").read_text());s,c=f["pairs"][0];s["basis"]["source_refs"].reverse();c["semantic_digest"]=h18("unit-semantic",s)
  with self.assertRaisesRegex(Reject,"canonical slot order"):decode_native(input_frame(f["header"],f["pairs"]))
 def test_review_challenge_bad_company_uuid(self):
  c=json.loads((G/"owner-direct-wage.json").read_text());c["org_id"]="not-a-uuid"
  with self.assertRaisesRegex(Reject,"UUID"):owner_frame(c)
 def test_review_challenge_registration_revision_zero(self):
  c=json.loads((G/"owner-direct-wage.json").read_text());c["action_registration_revision"]="0"
  with self.assertRaisesRegex(Reject,"registration revision"):owner_frame(c)
 def test_review_challenge_unregistered_action_rehashed(self):
  c=json.loads((G/"owner-direct-wage.json").read_text());c["owner_action"]="unregistered.action";c["expected"]["registered_input_digest"]=registered_digest(c["owner_action"],c["body"])
  with self.assertRaisesRegex(Reject,"unregistered"):owner_frame(c)
 def test_owner_unknown_field_or_numeric_value_rejected(self):
  for field,value in [("new_field","unapproved"),("amount_won",3000000),("monthly_standard_hours","000209")]:
   c=json.loads((G/"owner-direct-wage.json").read_text());c["body"][field]=value
   with self.assertRaises(Reject):owner_frame(c)
 def test_owner_invalid_required_null_cannot_rehash(self):
  c=json.loads((G/"owner-gated-calculate.json").read_text());c["body"]["expected"].pop("close_basis");c["expected"]["registered_input_digest"]=registered_digest(c["owner_action"],c["body"])
  with self.assertRaises(Reject):owner_frame(c)

def golden_test(case):
 def test(self):
  raw=(G/(case["id"]+".bin")).read_bytes();self.assertEqual(len(raw),case["bytes"]);self.assertEqual(sha(raw),case["sha256"])
  literal=json.loads((G/(case["id"]+".json")).read_text())
  if case["kind"]=="owner":
   self.assertEqual(decode_owner(raw),literal);self.assertEqual(sha(b"console.command.ccf1\0"+canonical_command(literal)),case["command_fingerprint"])
  else:
   h,p,d=decode_native(raw,case["kind"]=="calculation",pair_input() if case["input"] else None,header_input() if case["input"] else None)
   self.assertEqual(h,literal["header"]);self.assertEqual([list(x) for x in p],literal["pairs"]);self.assertEqual(d,case["manifest_digest"])
 return test
for c in INDEX:setattr(Reference,"test_golden_"+c["id"].replace("-","_"),golden_test(c))
def payload_test(k):
 def test(self):
  value=json.loads((ROOT/"payloads"/(k+".json")).read_text());errors=validate_payload(k,value["body"]);self.assertEqual(errors,[])
 return test
for p in (ROOT/"payloads").glob("*.json"):setattr(Reference,"test_schema_"+p.stem.replace(".","_"),payload_test(p.stem))
def raw_rejection(change):
 def test(self):
  b=(G/"input-resolved-one.bin").read_bytes();raw=change(b)
  with self.assertRaises(Reject):decode_native(raw)
 return test
r=Reader((G/"input-resolved-one.bin").read_bytes());r.take(len(INPUT_TAG));r.leaf();header_end=r.pos;count_at=header_end+32;ordinal_at=count_at+4;semantic_len_at=ordinal_at+4
mutations={"trailing":lambda b:b+b"x","truncate":lambda b:b[:-1],"wrong_tag":lambda b:b"X"+b[1:],"header_hash":lambda b:b[:header_end]+bytes([b[header_end]^1])+b[header_end+1:],"zero_units":lambda b:b[:count_at]+u32(0)+b[count_at+4:],"too_many_units":lambda b:b[:count_at]+u32(2001)+b[count_at+4:],"ordinal_gap":lambda b:b[:ordinal_at]+u32(1)+b[ordinal_at+4:],"oversized_leaf":lambda b:b[:semantic_len_at]+u32(65537)+b[semantic_len_at+4:]}
for k,fn in mutations.items():setattr(Reference,"test_reject_"+k,raw_rejection(fn))
if __name__=="__main__":unittest.main(verbosity=2)
