#!/usr/bin/env python3
"""Custody for declared native People assets; never a product compiler."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tempfile

CLIENT = "clients/desktop-web"
COMMITTED = "backend/crates/payroll/ui/react"
OUTPUTS = ("people.js", "people.css", "people-guard.js")
RECIPES = (".buckconfig", "BUCK", "tools/buck2", "tools/ui/BUCK", "tools/ui/react.bzl",
           "tools/ui/react_bundle.py", "tools/buck/gen_first_party.py", "backend/crates/payroll/ui/BUCK",
           *(CLIENT + "/" + name for name in ("BUCK", "dependencies.lock.json", "dependencies.lock.bzl", "package.json", "tsconfig.json")))


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def file_hash(path):
    try:
        return digest(path.read_bytes())
    except OSError:
        return None


def hashes(inputs):
    return {name: file_hash(path) for name, path in sorted(inputs.items())}


def sha(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def closed(value, keys):
    return isinstance(value, dict) and set(value) == set(keys)


def verify(record, inputs, outputs, lock, typecheck=None):
    if not closed(record, ["format", "inputs", "outputs", "dependency_lock_sha256", "producer"]):
        return ["Invalid React record shape"]
    failures = []
    if type(record["format"]) is not int or record["format"] != 1:
        failures.append("Invalid React record format")
    for key, actual in [("inputs", hashes(inputs)), ("outputs", hashes(outputs))]:
        recorded = record[key]
        if not isinstance(recorded, dict) or any(not sha(v) for v in actual.values()) or recorded != actual:
            failures.append("React " + key + " identity mismatch")
    if record["dependency_lock_sha256"] != digest(canonical(lock)):
        failures.append("React dependency lock mismatch")
    producer = record["producer"]
    if not closed(producer, ["kind", "host", "tools", "versions", "typecheck_sha256"]):
        return failures + ["Invalid native React producer shape"]
    if producer["kind"] != "buck2-native-react-people-v1" or not sha(producer["typecheck_sha256"]):
        failures.append("Missing native compiler/typecheck proof")
    host = producer["host"]
    pin = lock["hosts"].get(host) if isinstance(host, str) else None
    expected_tools = {tool: pin[tool]["executable_sha256"] for tool in ["node", "esbuild"]} if pin else None
    if not closed(producer["tools"], ["node", "esbuild"]) or producer["tools"] != expected_tools:
        failures.append("Unsupported host or wrong native tool pin")
    if not closed(producer["versions"], lock["versions"]) or producer["versions"] != lock["versions"]:
        failures.append("Wrong compiler versions")
    if typecheck is not None:
        try:
            check = json.loads(typecheck.read_text())
            if not isinstance(check, dict) or check.get("version") != lock["versions"]["typescript"]:
                failures.append("Wrong typecheck version")
        except (OSError, ValueError):
            failures.append("Missing or invalid typecheck artifact")
        if producer["typecheck_sha256"] != file_hash(typecheck):
            failures.append("Typecheck artifact mismatch")
    return failures


def load(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise ValueError("Missing or invalid React input record") from error


def output_paths(directory):
    return {name: directory / name for name in OUTPUTS}


def validate_native(candidate, inputs, lock):
    if set(p.name for p in candidate.iterdir()) != {*OUTPUTS, "bundle.lock.json", "typecheck.tsbuildinfo"}:
        raise ValueError("Unexpected or missing native React files")
    record = load(candidate / "bundle.lock.json")
    failures = verify(record, inputs, output_paths(candidate), lock, candidate / "typecheck.tsbuildinfo")
    if failures:
        raise ValueError("; ".join(failures))
    return record


def validate_pair(candidate, committed, inputs, lock):
    native = validate_native(candidate, inputs, lock)
    if set(p.name for p in committed.iterdir()) != {*OUTPUTS, "bundle.lock.json"}:
        raise ValueError("Unexpected or missing committed React files")
    recorded = load(committed / "bundle.lock.json")
    failures = verify(recorded, inputs, output_paths(committed), lock)
    if failures or native["outputs"] != recorded["outputs"]:
        raise ValueError("Committed React bundle differs from declared native compilation: " + "; ".join(failures))


def checkout_inputs(root):
    sources = [p for p in (root / CLIENT / "src").rglob("*") if p.is_file()]
    if not sources or any(p.is_symlink() or p.suffix not in [".ts", ".tsx", ".css"] for p in sources):
        raise ValueError("Undeclared or missing React source")
    inputs = {name: root / name for name in RECIPES}
    inputs.update({p.relative_to(root).as_posix(): p for p in sources})
    return inputs


def checkout_lock(root):
    return load(root / CLIENT / "dependencies.lock.json")


def lock_projection(lock):
    return "# @generated by tools/ui/react_bundle.py generate; do not edit.\nLOCK = " + json.dumps(lock, indent=2) + "\n"


def declared_inputs(path):
    inputs = {name: Path(source) for name, source in load(path).items()}
    lock = load(inputs[CLIENT + "/dependencies.lock.json"])
    if inputs[CLIENT + "/dependencies.lock.bzl"].read_text() != lock_projection(lock):
        raise ValueError("Buck archive pins differ from canonical dependency lock")
    return inputs, lock


def normalized_path(value):
    if not isinstance(value, (str, Path)) or not str(value):
        raise ValueError("Invalid React resolution path")
    if ".." in Path(value).parts:
        raise ValueError("Parent traversal in React resolution path")
    return Path(os.path.abspath(value))


def validate_metafile(path, workspace, sources, packages, outputs, entry):
    meta = load(path)
    if not closed(meta, ["inputs", "outputs"]) or any(
            not isinstance(meta[key], dict) or not meta[key] for key in ["inputs", "outputs"]):
        raise ValueError("Missing React resolution closure")
    inputs = {normalized_path(name): row for name, row in meta["inputs"].items()}
    emitted = {normalized_path(name).resolve(strict=True): row for name, row in meta["outputs"].items()}
    if len(inputs) != len(meta["inputs"]) or len(emitted) != len(meta["outputs"]) or set(emitted) != set(outputs):
        raise ValueError("React resolution output identity mismatch")
    for name in inputs:
        actual = name.resolve(strict=True)
        if not actual.is_file() or not (sources.get(name) == actual or any(
                name.is_relative_to(staged) and actual.is_relative_to(archive)
                for staged, archive in packages.items())):
            raise ValueError("Undeclared React resolution input")
    for rows in [inputs, emitted]:
        for row in rows.values():
            if not isinstance(row, dict) or type(row.get("bytes")) is not int or row["bytes"] < 0 or not isinstance(row.get("imports"), list):
                raise ValueError("Invalid React resolution metadata")
            for edge in row["imports"]:
                if not isinstance(edge, dict) or edge.get("external", False) is not False or not isinstance(edge.get("kind"), str) or not edge["kind"]:
                    raise ValueError("External or invalid React import edge")
                if normalized_path(edge.get("path")) not in inputs:
                    raise ValueError("React import edge leaves declared closure")
    for row in emitted.values():
        contributions = row.get("inputs")
        if not isinstance(contributions, dict) or any(
                normalized_path(name) not in inputs or not isinstance(value, dict)
                or type(value.get("bytesInOutput")) is not int or value["bytesInOutput"] < 0
                for name, value in contributions.items()):
            raise ValueError("Invalid React output input closure")
    expected_entry = workspace / "src" / entry
    if expected_entry not in inputs or normalized_path(emitted[outputs[0]].get("entryPoint")) != expected_entry:
        raise ValueError("React resolution entry identity mismatch")


def validate_resolution(args, inputs, lock):
    try:
        workspace = normalized_path(args.workspace)
        declared = load(args.packages)
        names = [package["name"] for package in lock["packages"]]
        if any(not isinstance(name, str) or not re.fullmatch(r"(?:@[a-z0-9._-]+/)?[a-z0-9._-]+", name) for name in names):
            raise ValueError("Invalid locked React package name")
        names = {name for name in names if not name.startswith("@esbuild/")}
        if not names or not closed(declared, names):
            raise ValueError("React package roots differ from dependency lock")
        packages = {}
        for name, archive in declared.items():
            staged = workspace / "node_modules" / name
            actual = normalized_path(archive).resolve(strict=True)
            if not actual.is_dir() or staged.resolve(strict=True) != actual:
                raise ValueError("React staged package root identity mismatch")
            packages[staged] = actual
        sources = {}
        prefix = CLIENT + "/src/"
        for name, source in inputs.items():
            if name.startswith(prefix):
                staged = normalized_path(workspace / "src" / name[len(prefix):])
                if not staged.is_relative_to(workspace / "src"):
                    raise ValueError("Invalid declared React source path")
                sources[staged] = source.resolve(strict=True)
        output = {name: (args.compiled / name).resolve(strict=True) for name in OUTPUTS}
        validate_metafile(args.runtime_metafile, workspace, sources, packages,
                          [output["people.js"], output["people.css"]], "people.tsx")
        validate_metafile(args.guard_metafile, workspace, sources, packages,
                          [output["people-guard.js"]], "people-guard.ts")
    except (OSError, RuntimeError, KeyError, TypeError) as error:
        raise ValueError("Missing or invalid React resolution inputs") from error


def record_bundle(args):
    inputs, lock = declared_inputs(args.inputs)
    validate_resolution(args, inputs, lock)
    tools = {"node": file_hash(args.node), "esbuild": file_hash(args.esbuild)}
    hosts = [host for host, pin in lock["hosts"].items()
             if all(tools[tool] == pin[tool]["executable_sha256"] for tool in tools)]
    if len(hosts) != 1:
        raise ValueError("Native compiler executable does not match one pinned host")
    record = {"format": 1, "inputs": hashes(inputs), "outputs": hashes(output_paths(args.compiled)),
              "dependency_lock_sha256": digest(canonical(lock)),
              "producer": {"kind": "buck2-native-react-people-v1", "host": hosts[0], "tools": tools,
                           "versions": lock["versions"], "typecheck_sha256": file_hash(args.compiled / "typecheck.tsbuildinfo")}}
    failures = verify(record, inputs, output_paths(args.compiled), lock, args.compiled / "typecheck.tsbuildinfo")
    if failures:
        raise ValueError("; ".join(failures))
    args.out_dir.mkdir(parents=True)
    for name in [*OUTPUTS, "typecheck.tsbuildinfo"]:
        shutil.copyfile(args.compiled / name, args.out_dir / name)
    (args.out_dir / "bundle.lock.json").write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")


def publish(candidate, root):
    inputs, lock = checkout_inputs(root), checkout_lock(root)
    record = validate_native(candidate, inputs, lock)
    destination = root / COMMITTED
    destination.mkdir(parents=True, exist_ok=True)
    for name in OUTPUTS:
        shutil.copyfile(candidate / name, destination / name)
    # A changed source or partial copy never receives a current manifest.
    failures = verify(record, checkout_inputs(root), output_paths(destination), checkout_lock(root))
    if failures:
        raise ValueError("; ".join(failures))
    with tempfile.NamedTemporaryFile(dir=destination, prefix=".bundle-", delete=False) as temporary:
        temporary.write((candidate / "bundle.lock.json").read_bytes())
    Path(temporary.name).replace(destination / "bundle.lock.json")
    validate_pair(candidate, destination, checkout_inputs(root), checkout_lock(root))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["record", "validate", "publish", "generate"])
    for name in ["inputs", "compiled", "node", "esbuild", "out-dir", "candidate", "committed", "root",
                 "workspace", "packages", "runtime-metafile", "guard-metafile"]:
        parser.add_argument("--" + name, type=Path)
    args = parser.parse_args()
    if args.mode == "record":
        record_bundle(args)
    elif args.mode == "validate":
        inputs, lock = declared_inputs(args.inputs)
        validate_pair(args.candidate, args.committed, inputs, lock)
        args.out_dir.mkdir(parents=True)
        for name in OUTPUTS:
            shutil.copyfile(args.candidate / name, args.out_dir / name)
    elif args.mode == "publish":
        publish(args.candidate, args.root)
    else:
        path = args.root / CLIENT / "dependencies.lock.bzl"
        path.write_text(lock_projection(checkout_lock(args.root)))


if __name__ == "__main__":
    main()
