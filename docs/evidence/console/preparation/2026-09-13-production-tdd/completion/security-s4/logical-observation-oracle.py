"""Independent logical-sink oracle. Never builds runtime results or capabilities.
All observations must come from real collectors; synthetic inputs are checker tests.
Clock/random coupling is explicit input evidence, never a regex redaction of output.
"""
import base64
class InvalidObservation(ValueError):pass
def require(x,message):
 if not x:raise InvalidObservation(message)
def validate(o):
 require(o["complete"] is True,"incomplete collector")
 require(set(o)=={"complete","public_environment","http","ssr","cache","live","events","egress","allowed_markers","forbidden_markers"},"closed observation shape")
 for sink in ["http","ssr","cache","live","events","egress"]:
  require(isinstance(o[sink],list),"sink array")
 for r in o["http"]+o["ssr"]:
  b=base64.b64decode(r["body_base64"],validate=True)
  require(len(b)==r["body_length"],"body length mismatch")
  require(isinstance(r["headers"],list),"headers preserve duplicates/order")
 for sink in ["live","events","egress"]:
  require([r["ordinal"] for r in o[sink]]==list(range(len(o[sink]))) ,"missing/reordered event")
 import json
 serialized=json.dumps({k:o[k] for k in ["http","ssr","cache","live","events","egress"]},sort_keys=True).encode()
 bodies=b"\n".join(base64.b64decode(r["body_base64"],validate=True) for r in o["http"]+o["ssr"])+b"\n".join(base64.b64decode(h["value_base64"],validate=True) for r in o["http"]+o["ssr"] for h in r["headers"])
 for m in o["forbidden_markers"]:require(m.encode() not in serialized+bodies,"private marker disclosed")
 for m in o["allowed_markers"]:require(m.encode() in serialized+bodies,"vacuous allowed projection")
 return o
def paired(left,right):
 validate(left);validate(right)
 require(left["public_environment"]==right["public_environment"],"uncoupled public inputs")
 for sink in ["http","ssr","cache","live","events","egress"]:require(left[sink]==right[sink],f"logical {sink} observation differs")
