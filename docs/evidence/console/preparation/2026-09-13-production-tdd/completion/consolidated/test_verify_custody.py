import gzip,hashlib,importlib.util,json,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('custody',Path(__file__).with_name('verify-custody.py'));v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
def sha(b):return hashlib.sha256(b).hexdigest()
class Custody(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
  self.members=[];self.mapping=[]
  for name,body,target in [('fixture.rs',b'// TEST_ONLY retained source\n','fixture.rs'),('run.log',b'PASS\n','run.log.gz')]:
   retained=gzip.compress(body,mtime=0) if target.endswith('.gz') else body
   (self.root/target).write_bytes(retained);self.members.append({'path':name,'sha256':sha(body),'bytes':len(body)})
   self.mapping.append({'original':name,'retained':target,'original_sha256':sha(body),'retained_sha256':sha(retained)})
  self.write()
 def write(self):
  raw=json.dumps({'files':self.members}).encode();(self.root/'freeze-manifest.json').write_bytes(raw)
  self.location={'manifest_sha256':sha(raw),'files':self.mapping};self.save()
 def save(self):(self.root/'relocation-verification.json').write_text(json.dumps(self.location))
 def rejects(self):
  with self.assertRaises((AssertionError,ValueError,KeyError,FileNotFoundError,OSError)):v.verify(self.root)
 def test_positive_two_exact_originals(self):self.assertEqual(v.verify(self.root),2)
 def test_members_manifest_positive(self):
  raw=json.dumps({'members':self.members}).encode();(self.root/'freeze-manifest.json').write_bytes(raw);self.location['manifest_sha256']=sha(raw);self.save();self.assertEqual(v.verify(self.root),2)
 def test_ambiguous_manifest_refuses(self):
  raw=json.dumps({'members':self.members,'files':self.members}).encode();(self.root/'freeze-manifest.json').write_bytes(raw);self.location['manifest_sha256']=sha(raw);self.save();self.rejects()
 def test_changed_source(self):(self.root/'fixture.rs').write_bytes(b'changed');self.rejects()
 def test_missing_source(self):(self.root/'fixture.rs').unlink();self.rejects()
 def test_omitted_request_mapping(self):self.mapping.pop();self.save();self.rejects()
 def test_duplicate_member(self):self.members.append(dict(self.members[0]));self.write();self.rejects()
 def test_duplicate_mapping(self):self.mapping[1]=dict(self.mapping[0]);self.save();self.rejects()
 def test_manifest_digest_mismatch(self):(self.root/'freeze-manifest.json').write_bytes(b'{"files":[]}');self.rejects()
 def test_raw_byte_length_mismatch(self):self.members[0]['bytes']+=1;self.write();self.rejects()
 def test_recomputed_compressed_hash_cannot_hide_wrong_raw_bytes(self):
  raw=gzip.compress(b'FAIL',mtime=0);(self.root/'run.log.gz').write_bytes(raw);self.mapping[1]['retained_sha256']=sha(raw);self.save();self.rejects()
 def test_path_escape(self):self.mapping[0]['retained']='../fixture.rs';self.save();self.rejects()
 def test_symlink_even_matching_bytes(self):
  target=self.root/'real-source';target.write_bytes((self.root/'fixture.rs').read_bytes());(self.root/'fixture.rs').unlink();(self.root/'fixture.rs').symlink_to(target);self.rejects()
 def test_legacy_manifest_member_positive(self):
  raw=(self.root/'freeze-manifest.json').read_bytes();self.mapping.append({'original':'freeze-manifest.json','retained':'freeze-manifest.json','original_sha256':sha(raw),'retained_sha256':sha(raw)});self.save();self.assertEqual(v.verify(self.root),3)
 def test_legacy_mapping_shape_positive(self):
  self.location={'source_manifest_sha256':self.location['manifest_sha256'],'mapping':self.mapping};self.save();self.assertEqual(v.verify(self.root),2)
 def test_legacy_raw_only_positive(self):
  self.location={'original_manifest_sha256':self.location['manifest_sha256'],'files':[{'source_path':e['original'],'retained_path':e['retained'],'sha256':e['original_sha256']} for e in self.mapping]};self.save();self.assertEqual(v.verify(self.root),2)
class Roster(unittest.TestCase):
 def test_empty_roster_refuses(self):
  with tempfile.TemporaryDirectory() as d:
   with self.assertRaises(AssertionError):v.verify_all(Path(d),{'packets':[]})
 def test_missing_required_folder_refuses(self):
  with tempfile.TemporaryDirectory() as d:
   with self.assertRaises(AssertionError):v.verify_all(Path(d),{'packets':[{'packet':'missing','manifest_sha256':'a'*64}]})
 def test_omitted_existing_packet_refuses(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);(root/'extra').mkdir();(root/'extra/relocation-verification.json').write_text('{}')
   with self.assertRaises(AssertionError):v.verify_all(root,{'packets':[{'packet':'required','manifest_sha256':'a'*64}]})
if __name__=='__main__':unittest.main(verbosity=2)
