#!/usr/bin/env python3
"""Document/fixture consistency only. Requires jsonschema==4.25.1; no product proof."""
import hashlib
import importlib.metadata
import json
from pathlib import Path
import subprocess
from jsonschema import Draft202012Validator, FormatChecker

root = Path(__file__).resolve().parent
assert importlib.metadata.version("jsonschema") == "4.25.1"
def read(name):
    return json.loads((root / name).read_text())
def sha(data):
    return hashlib.sha256(data).hexdigest()
schema = read("wire.schema.json")
Draft202012Validator.check_schema(schema)
examples = read("examples.json")
for category in ("valid", "invalid"):
    for item in examples[category]:
        validator = Draft202012Validator({**schema, "$ref": "#/$defs/" + item["schema"]}, format_checker=FormatChecker())
        errors = list(validator.iter_errors(item["value"]))
        assert bool(errors) == (category == "invalid"), item["name"]
index = read("fixtures/artifact-index.json")
for item in [index["manifest"], *index["content"]]:
    file = root / item["path"]
    assert file.is_file() and not file.is_symlink() and root in file.resolve().parents
    assert sha(file.read_bytes()) == item["sha256"]
manifest = read(index["manifest"]["path"])
assert manifest["fixture_only"] is True
assert len({x["terms_kind"] for x in manifest["items"]}) == len(manifest["items"])
assert {x["content_sha256"] for x in manifest["items"]} == {x["sha256"] for x in index["content"]}
by_name = {x["name"]: x["value"] for x in examples["valid"]}
assert by_name["terms"]["terms_version"] == index["manifest"]["sha256"]
assert by_name["terms"]["manifest_url"].endswith(index["manifest"]["sha256"])
finish = by_name["finish_structural_only"]
assert finish["accept_terms_version"] == index["manifest"]["sha256"]
assert {x["terms_kind"] for x in finish["accept_items"]} == {x["terms_kind"] for x in manifest["items"]}
assert len(finish["accept_items"]) == len(manifest["items"])
assert all(x["accepted"] is True for x in finish["accept_items"])
assert by_name["session"]["account"] == by_name["me"]
assert by_name["empty_contexts"] == {"contexts": []}
registrations = read("action-registrations.json")["registrations"]
assert {(x["action_key"], x["registration_revision"]) for x in registrations} == {(x["action_key"], x["registration_revision"]) for x in by_name["me"]["permitted_self_actions"]}
for route in read("routes.json")["routes"]:
    assert route["result"] in schema["$defs"]
    assert route["input"] is None or route["input"] in schema["$defs"]
refs = read("inherited-sources.json")
for item in refs["files"]:
    spec = refs["approved_design_sha"] + ":" + item["path"]
    raw = subprocess.check_output(["git", "show", spec], cwd=root)
    assert sha(raw) == item["sha256"]
    assert subprocess.check_output(["git", "rev-parse", spec], cwd=root, text=True).strip() == item["git_blob"]
print(json.dumps({"status": "PASS", "validator": "jsonschema 4.25.1", "valid_structural_examples": len(examples["valid"]), "invalid_structural_examples_rejected": len(examples["invalid"]), "hashed_artifacts": 1 + len(index["content"]), "inherited_sources": len(refs["files"]), "product_tests_executed": 0, "claim": "Document/fixture consistency only; no crypto/runtime/load/formal or legal proof"}, indent=2))
