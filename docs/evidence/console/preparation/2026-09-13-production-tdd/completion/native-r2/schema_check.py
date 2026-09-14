import json
from pathlib import Path
from urllib.parse import urlparse,unquote
from jsonschema import Draft202012Validator
from referencing import Registry,Resource
ROOT=Path(__file__).resolve().parent
def retrieve(uri):
 p=Path(unquote(urlparse(uri).path)).resolve()
 if not p.is_relative_to(ROOT/"source"):raise ValueError("external schema: "+uri)
 d=json.loads(p.read_text());d["$id"]=p.as_uri();return Resource.from_contents(d)
REG=Registry(retrieve=retrieve)
def validator(path,fragment):
 p=path.resolve();return Draft202012Validator({"$ref":p.as_uri()+"#"+fragment},registry=REG)
def validate_payload(k,value):
 a=ROOT/"source/2026-09-13-integrated-design-30"
 bindings=json.loads((a/"owner-bindings.json").read_text())["bindings"]
 b=next(b for b in bindings if b["key"]==k);f,frag=b["input"].split("#")
 return list(validator(a/f,frag).iter_errors(value))
if __name__=="__main__":
 failures=[];n=0
 for p in sorted((ROOT/"payloads").glob("*.json")):
  v=json.loads(p.read_text());es=validate_payload(v["kind"],v["body"]);n+=1
  if es:failures.append((v["kind"],[{"path":"/"+"/".join(map(str,e.path)),"message":e.message} for e in es]))
 print(json.dumps({"discovered":n,"passed":n-len(failures),"failed":len(failures),"failures":failures},ensure_ascii=False,indent=2))
 raise SystemExit(bool(failures))
