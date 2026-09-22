#!/usr/bin/env python3
"""One integrity implementation for checkout checks and declared Buck actions."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib

REPO = Path(__file__).resolve().parents[2]
CRATE = "backend/crates/payroll/ui"
SRC = CRATE + "/src"
MANIFEST = CRATE + "/bundle.lock.json"
OUTPUTS = [CRATE + "/pkg/console_payroll_ui_bg.wasm", CRATE + "/pkg/console_payroll_ui.js"]
GENERATION_INPUTS = ["backend/Cargo.toml", "backend/Cargo.lock", CRATE + "/Cargo.toml",
                     "rust-toolchain.toml", "third-party/rust/reindeer.toml",
                     "tools/buck/gen_hydrate.py", "tools/ui/wasm_bundle.py"]
REBUILD = "Rebuild with: bash tools/ui/build-payroll-wasm.sh"
RECIPES = [
    "backend/Cargo.toml", "backend/Cargo.lock", CRATE + "/Cargo.toml", "rust-toolchain.toml", ".buckconfig",
    "BUCK", "backend/BUCK", CRATE + "/BUCK", "tools/ui/BUCK", "tools/buck/BUCK", "tools/buck2",
    "tools/ui/build-payroll-wasm.sh", "tools/ui/wasm_bundle.py", "tools/ui/hydration.bzl",
    "tools/buck/gen_hydrate.py", "tools/buck/gen_first_party.py",
    "toolchains/BUCK", "toolchains/rust/BUCK", "toolchains/rust/hermetic.bzl",
    "toolchains/rust/sysroot.bzl", "toolchains/rust/wasm.bzl", "toolchains/rust/lock.bzl",
    "third-party/rust/reindeer.toml", "third-party/rust/hydrate/BUCK",
    "third-party/rust/hydrate/Cargo.toml", "third-party/rust/hydrate/Cargo.lock",
    "third-party/rust/hydrate/reindeer.toml", "third-party/rust/hydrate/root-dependencies.json",
    "third-party/rust/hydrate/input-lock.json",
]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_hash(path):
    try:
        return digest(path.read_bytes())
    except FileNotFoundError:
        return None


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def git(root, *args):
    try:
        return sorted(filter(None, subprocess.check_output(
            ["git", "-C", str(root), *args], stderr=subprocess.PIPE).decode().split("\0")))
    except (OSError, subprocess.CalledProcessError) as error:
        raise ValueError(f"could not read {SRC} with git. This gate needs a checkout with .git.") from error


def checkout_inputs(root, strict=False):
    sources = git(root, "ls-files", "-z", "--", SRC)
    if SRC + "/lib.rs" not in sources:
        raise ValueError(f"git listed {len(sources)} file(s) under {SRC} and {SRC}/lib.rs was not among them. Refusing a source-blind manifest.")
    if strict:
        untracked = git(root, "ls-files", "--others", "--exclude-standard", "-z", "--", SRC)
        if untracked:
            raise ValueError("untracked file(s) under " + SRC + ": " + ", ".join(untracked))
        hidden = [p for p in git(root, "ls-files", "--others", "--ignored", "--exclude-standard", "-z", "--", SRC)
                  if p.endswith((".rs", ".js"))]
        if hidden:
            raise ValueError("gitignored source file(s) under " + SRC + ": " + ", ".join(hidden))
    # Both canonical and mechanically projected fixups influence the graph.
    fixups = []
    for prefix in ("third-party/rust/fixups", "third-party/rust/hydrate/fixups"):
        fixups += git(root, "ls-files", "-z", "--", prefix)
    return {rel: root / rel for rel in sorted(set(RECIPES + sources + fixups))}


def hashes(inputs):
    return {name: file_hash(path) for name, path in sorted(inputs.items())}


def declared_inputs(path):
    inputs = {}
    for name, source in json.loads(path.read_text()).items():
        source = Path(source)
        if source.is_dir():
            inputs.update({name + "/" + item.relative_to(source).as_posix(): item
                           for item in sorted(source.rglob("*")) if item.is_file()})
        else:
            inputs[name] = source
    return inputs


def declarations(inputs):
    pin = tomllib.loads(inputs["rust-toolchain.toml"].read_text())["toolchain"]["channel"]
    crate = tomllib.loads(inputs[CRATE + "/Cargo.toml"].read_text())
    return pin, crate["dependencies"]["wasm-bindgen"]["version"]


def verify(recorded, inputs, outputs):
    failures = []
    if not isinstance(recorded, dict):
        shape = "an array" if isinstance(recorded, list) else ("number" if isinstance(recorded, (int, float)) else type(recorded).__name__)
        return [f"{MANIFEST} is not a JSON object (parsed as {shape}). {REBUILD}"]
    now = hashes(inputs)
    pin, bindgen = declarations(inputs)
    if recorded.get("toolchain_channel") != pin:
        failures.append(f"the bundle was built with channel {recorded.get('toolchain_channel')} but rust-toolchain.toml says {pin}. {REBUILD}")
    if not recorded.get("wasm_bindgen_cli"):
        failures.append(f"{MANIFEST} records no wasm-bindgen version. {REBUILD}")
    elif recorded["wasm_bindgen_cli"] != bindgen:
        failures.append(f"the bundle's glue was emitted by wasm-bindgen CLI {recorded['wasm_bindgen_cli']} but {CRATE}/Cargo.toml declares {bindgen}. {REBUILD}")
    old = recorded.get("inputs") if isinstance(recorded.get("inputs"), dict) else {}
    for name in now.keys() - old.keys():
        failures.append(f"{name} is a source file the committed bundle was not built from. {REBUILD}")
    for name in old.keys() - now.keys():
        failures.append(f"{name} was built into the committed bundle but no longer exists. {REBUILD}")
    for name in old.keys() & now.keys():
        if now[name] is None or old[name] != now[name]:
            failures.append(f"{name} has changed since the bundle was built. {REBUILD}")
    old_outputs = recorded.get("outputs") if isinstance(recorded.get("outputs"), dict) else {}
    for name in OUTPUTS:
        if not old_outputs.get(name):
            failures.append(f"{MANIFEST} records no hash for {name}. {REBUILD}")
        elif old_outputs[name] != file_hash(outputs[name]):
            failures.append(f"{name} does not match the hash recorded when it was built — it was edited by hand or partially rebuilt. {REBUILD}")
    producer = recorded.get("producer", {})
    if (recorded.get("format") != 2 or not isinstance(producer, dict)
            or producer.get("kind") != "buck2-native-hydration-v1"
            or producer.get("source_snapshot_sha256") != digest(canonical(old))
            or any(not re.fullmatch(r"[0-9a-f]{64}", str(producer.get(key, "")))
                   for key in ("compiled_wasm_sha256", "bindgen_executable_sha256"))):
        failures.append(f"{MANIFEST} has no complete native producer record. {REBUILD}")
    return sorted(failures)


def load_manifest(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise ValueError(f"{MANIFEST} is missing or unparseable. {REBUILD}") from error


def require_valid(recorded, inputs, outputs):
    failures = verify(recorded, inputs, outputs)
    if failures:
        raise ValueError("Committed hydration bundle is stale:\n" + "\n".join("- " + f for f in failures))


def snapshot(inputs, output):
    names = set(GENERATION_INPUTS) | {name for name in inputs if name.startswith("third-party/rust/fixups/")}
    generation = json.loads(inputs["third-party/rust/hydrate/input-lock.json"].read_text())
    if generation.get("inputs") != hashes({name: inputs[name] for name in names}):
        raise ValueError("hydration dependency metadata is stale; regenerate with tools/buck/gen_hydrate.py")
    output.mkdir(parents=True)
    for name, source in inputs.items():
        target = output / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    # Hash the immutable copied action output actually consumed by rustc.
    contents = hashes({name: output / name for name in inputs})
    (output / "inputs.json").write_bytes(canonical(contents))


def produce(source_snapshot, wasm, bindgen, output):
    recorded_inputs = json.loads((source_snapshot / "inputs.json").read_text())
    inputs = {name: source_snapshot / name for name in recorded_inputs}
    if hashes(inputs) != recorded_inputs:
        raise ValueError("immutable compiler source snapshot was changed")
    pin, version = declarations(inputs)
    actual = subprocess.check_output([str(bindgen), "--version"], text=True).strip()
    if actual != "wasm-bindgen " + version:
        raise ValueError("declared bindgen executable version does not match the crate")
    if wasm.read_bytes()[:8] != b"\0asm\x01\0\0\0":
        raise ValueError("native compiler did not emit a WebAssembly module")
    output.mkdir(parents=True)
    subprocess.run([str(bindgen), str(wasm), "--out-dir", str(output), "--out-name", "console_payroll_ui", "--target", "web"], check=True)
    for path in output.glob("*.d.ts"):
        path.unlink()
    outputs = {name: output / Path(name).name for name in OUTPUTS}
    if any(not p.is_file() for p in outputs.values()):
        raise ValueError("bindgen failed to emit the complete output pair")
    record = {"format": 2, "toolchain_channel": pin, "wasm_bindgen": version,
              "wasm_bindgen_cli": version, "inputs": recorded_inputs,
              "outputs": hashes(outputs), "producer": {
                  "kind": "buck2-native-hydration-v1",
                  "source_snapshot_sha256": digest(canonical(recorded_inputs)),
                  "compiled_wasm_sha256": file_hash(wasm),
                  "bindgen_executable_sha256": file_hash(bindgen)}}
    require_valid(record, inputs, outputs)
    (output / "bundle.lock.json").write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")


def publish(candidate, root):
    inputs = checkout_inputs(root, strict=True)
    record = load_manifest(candidate / "bundle.lock.json")
    outputs = {name: candidate / Path(name).name for name in OUTPUTS}
    require_valid(record, inputs, outputs)
    # Manifest last: any interruption leaves a pair the consumer gate rejects.
    # Never recompute provenance from the now-mutable checkout.
    for name, source in outputs.items():
        target = root / name
        pending = target.with_name(target.name + ".pending")
        shutil.copyfile(source, pending)
        pending.replace(target)
    # Recheck current source and candidate bytes after output copying.
    require_valid(record, checkout_inputs(root, strict=True), outputs)
    target = root / MANIFEST
    pending = target.with_name(target.name + ".pending")
    shutil.copyfile(candidate / "bundle.lock.json", pending)
    pending.replace(target)


def validate(manifest, inputs, outputs, native_candidate, output):
    recorded = load_manifest(manifest)
    require_valid(recorded, inputs, outputs)
    native_record = load_manifest(native_candidate / "bundle.lock.json")
    native_outputs = {name: native_candidate / Path(name).name for name in OUTPUTS}
    require_valid(native_record, inputs, native_outputs)
    # A publisher may use a different supported host. Preserve its host tool
    # and compiler-input hashes as evidence, while requiring the current native
    # action to reproduce the shipped bytes and portable source/version identity.
    portable = ("format", "inputs", "outputs", "toolchain_channel", "wasm_bindgen", "wasm_bindgen_cli")
    if (any(recorded.get(key) != native_record.get(key) for key in portable)
            or any(outputs[name].read_bytes() != native_outputs[name].read_bytes() for name in OUTPUTS)):
        raise ValueError("committed hydration bundle does not match the native producer action. " + REBUILD)
    output.mkdir(parents=True)
    for name, source in native_outputs.items():
        shutil.copyfile(source, output / Path(name).name)
    require_valid(native_record, inputs, {name: output / Path(name).name for name in OUTPUTS})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("check", "snapshot", "produce", "validate", "publish"))
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--root", type=Path, default=REPO)
    parser.add_argument("--inputs", type=Path)
    parser.add_argument("--snapshot", type=Path)
    parser.add_argument("--input", type=Path)
    parser.add_argument("--bindgen", type=Path)
    parser.add_argument("--out-dir", type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--js", type=Path)
    parser.add_argument("--wasm", type=Path)
    parser.add_argument("--candidate", type=Path)
    args = parser.parse_args()
    if args.mode == "check":
        inputs = checkout_inputs(args.root, strict=args.write)
        if args.write:
            raise ValueError("--write requires an action-bound native producer bundle; standalone resealing is refused. " + REBUILD)
        require_valid(load_manifest(args.root / MANIFEST), inputs, {p: args.root / p for p in OUTPUTS})
        print("hydration bundle matches its recorded inputs and output hashes")
    elif args.mode == "snapshot":
        snapshot(declared_inputs(args.inputs), args.out_dir)
    elif args.mode == "produce":
        produce(args.snapshot, args.input, args.bindgen, args.out_dir)
    elif args.mode == "validate":
        inputs = declared_inputs(args.inputs)
        outputs = {OUTPUTS[0]: args.wasm, OUTPUTS[1]: args.js}
        validate(args.manifest, inputs, outputs, args.candidate, args.out_dir)
    elif args.mode == "publish":
        publish(args.candidate, args.root)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
