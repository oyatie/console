"""Reproduce retained-form schema projection; never public admission schemas."""
import copy, json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
I64_MAX=2**63-1
BOUNDS={"PositiveRevision":(1,I64_MAX),"RevisionOrZero":(0,I64_MAX),
        "UnixMicroseconds":(-2**63,I64_MAX),"CalculationRevision":(1,2**31-1),
        "Ordinal":(0,255),"Hours":(1,I64_MAX)}
def project(value):
    if isinstance(value,list):return [project(v) for v in value]
    if not isinstance(value,dict):return value
    out={k:project(v) for k,v in value.items()}
    if value.get("type")=="integer":
        out["type"]="string";out["pattern"]="^(0|-?[1-9][0-9]*)$"
        out["x-decimal-min"]=out.pop("minimum",-2**63)
        out["x-decimal-max"]=out.pop("maximum",I64_MAX)
    if isinstance(value.get("const"),int) and not isinstance(value["const"],bool):
        out["const"]=str(value["const"])
    if "enum" in value:
        out["enum"]=[str(v) if isinstance(v,int) and not isinstance(v,bool) else v for v in value["enum"]]
    return out
if __name__=="__main__":
    count=0
    for source in sorted((ROOT/"source").rglob("*.schema.json")):
        result=project(json.loads(source.read_text()))
        for name,definition in result.get("$defs",{}).items():
            if name in BOUNDS and definition.get("type")=="string":
                definition["x-decimal-min"],definition["x-decimal-max"]=BOUNDS[name]
            if name=="Date" and definition.get("type")=="string":definition["format"]="date"
            if name=="EditingToken":
                definition["properties"].pop("nonce")
                definition["required"].remove("nonce")
        target=ROOT/"normalized-source"/source.relative_to(ROOT/"source")
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n");count+=1
    print(json.dumps({"projected_schema_files":count,"purpose":"retained-form validation only"}))
