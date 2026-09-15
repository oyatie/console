import json
from pathlib import Path
from jsonschema import Draft202012Validator
s=json.loads((Path(__file__).parent/'wire.schema.json').read_text());s['$ref']='#/$defs/TermsItem'
v=Draft202012Validator(s)
base={'terms_kind':'test.account.service','title':'x','locale':'ko','content_sha256':'ab'*32,'required':True}
cases=[('title','x',True),('title','가'*128,True),('title','',False),('title','가'*129,False),('locale','ko',True),('locale','가'*32,True),('locale','가',False),('locale','가'*33,False),('content_sha256','http://127.0.0.1/terms',False),('content_url','http://127.0.0.1/terms',False)]
for field,value,expected in cases:
 item=dict(base);item[field]=value;assert v.is_valid(item)==expected,(field,len(value),expected)
assert len(('가'*21845).encode()+b'x')==65536
assert len(('가'*21846).encode())==65538
print('PASS 10 approved schema cases + 2 UTF8 byte vectors; not publisher execution')
