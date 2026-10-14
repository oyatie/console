#!/usr/bin/env python3
"""Behavior locks for the repository-owned Reindeer bootstrap and closures."""

import hashlib
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class ReindeerBootstrapTests(unittest.TestCase):
    def read(self, relative_path: str) -> str:
        return (ROOT / relative_path).read_text()

    def test_reindeer_bootstrap_is_pinned_and_does_not_fall_back_to_path(self) -> None:
        lock = self.read("third-party/rust/reindeer/upstream.lock")
        bootstrap = self.read("third-party/rust/reindeer/bootstrap.sh")

        self.assertIn("REINDEER_COMMIT=681727ced54a853977ac495e147ac54e1c0db115", lock)
        self.assertIn(
            "REINDEER_ARCHIVE_SHA256=79c09407900ed8d2b70b620af24cb2e2c6b0661d75804326ae1ee77fe908259a",
            lock,
        )
        self.assertIn("REINDEER_TOOLCHAIN=nightly-2026-02-28", lock)
        self.assertIn("--fuzz=0", bootstrap)
        self.assertNotIn("command -v reindeer", bootstrap)

    def test_workspace_dev_closure_is_generated_from_direct_roots(self) -> None:
        config = self.read("third-party/rust/reindeer.toml")
        patch = self.read(
            "third-party/rust/reindeer/patches/0001-workspace-dev-dependency-roots.patch"
        )
        graph = self.read("third-party/rust/BUCK")

        self.assertIn("include_workspace_dev_dependencies = true", config)
        self.assertIn("workspace_dev_dependency_roots", patch)
        self.assertIn("DepKind::Dev", patch)
        self.assertIn("TargetReq::Lib", patch)
        self.assertIn('name = "webauthn-authenticator-rs"', graph)
        self.assertIn('name = "tempfile"', graph)

    def test_reindeer_bootstrap_uses_security_fixed_cargo_graph(self) -> None:
        bootstrap = self.read("third-party/rust/reindeer/bootstrap.sh")
        cargo_patch = self.read(
            "third-party/rust/reindeer/patches/0002-cargo-security-uplift.patch"
        )
        cargo_lock = self.read("third-party/rust/reindeer/Cargo.lock")

        self.assertIn('cargo = "0.98"', cargo_patch)
        self.assertIn("0002-cargo-security-uplift.patch", bootstrap)
        self.assertIn('name = "cargo"\nversion = "0.98.0"', cargo_lock)
        self.assertIn('name = "gix"\nversion = "0.83.0"', cargo_lock)
        self.assertIn('name = "gix-fs"\nversion = "0.21.2"', cargo_lock)
        self.assertIn('name = "gix-pack"\nversion = "0.70.0"', cargo_lock)
        self.assertIn('name = "gix-validate"\nversion = "0.11.3"', cargo_lock)

    def test_openssl_product_closure_is_vendored_and_uses_supported_cfgs(self) -> None:
        auth_manifest = self.read("backend/crates/platform/auth/Cargo.toml")
        openssl_sys = self.read("third-party/rust/fixups/openssl-sys/fixups.toml")
        openssl = self.read("third-party/rust/fixups/openssl/fixups.toml")
        graph = self.read("third-party/rust/BUCK")

        self.assertIn('openssl = { version = "0.10.80", features = ["vendored"] }', auth_manifest)
        self.assertEqual(
            tomllib.loads(openssl_sys)["buildscript"]["run"],
            {
                "env": {
                    "CARGO_MAKEFLAGS": "-j4",
                    "OPENSSL_RUST_USE_NASM": "0",
                },
                "rustc_link_lib": True,
                "rustc_link_search": True,
            },
        )
        openssl_sys_run = graph.split(
            'name = "openssl-sys-0.9-build-script-main-run",', 1
        )[1].split("    features = [", 1)[0]
        generated_env = openssl_sys_run.split("    env = ", 1)[1]
        self.assertEqual(
            generated_env,
            '{\n'
            '        "CARGO_MAKEFLAGS": "-j4",\n'
            '        "CARGO_MANIFEST_LINKS": "openssl",\n'
            '        "CARGO_PKG_AUTHORS": "Alex Crichton <alex@alexcrichton.com>:Steven Fackler <sfackler@gmail.com>",\n'
            '        "CARGO_PKG_DESCRIPTION": "FFI bindings to OpenSSL",\n'
            '        "CARGO_PKG_HOMEPAGE": "",\n'
            '        "CARGO_PKG_README": "README.md",\n'
            '        "CARGO_PKG_REPOSITORY": "https://github.com/rust-openssl/rust-openssl",\n'
            '        "CARGO_PKG_RUST_VERSION": "1.80.0",\n'
            '        "CARGO_PKG_VERSION_MAJOR": "0",\n'
            '        "CARGO_PKG_VERSION_MINOR": "9",\n'
            '        "CARGO_PKG_VERSION_PATCH": "117",\n'
            '        "CARGO_PKG_VERSION_PRE": "",\n'
            '        "OPENSSL_RUST_USE_NASM": "0",\n'
            "    },\n",
        )
        self.assertIn('name = "openssl-src-300.6.1+3.6.3.crate"', graph)
        self.assertIn('name = "openssl-src-300"', graph)
        self.assertIn('osslconf="OPENSSL_NO_IDEA"', openssl)
        self.assertIn('osslconf="OPENSSL_NO_SEED"', openssl)
        self.assertNotIn("ossl360", openssl)
        self.assertNotIn("ossl361", openssl)

    def test_active_tls_provider_propagates_real_native_link_outputs(self) -> None:
        workspace = tomllib.loads(self.read("backend/Cargo.toml"))["workspace"]["dependencies"]
        self.assertFalse(workspace["rustls"]["default-features"])
        self.assertIn("aws_lc_rs", workspace["rustls"]["features"])
        self.assertNotIn("ring", workspace["rustls"]["features"])
        self.assertIn("tls-rustls-aws-lc-rs", workspace["sqlx"]["features"])
        fixup = tomllib.loads(self.read("third-party/rust/fixups/aws-lc-sys/fixups.toml"))
        self.assertEqual(fixup["buildscript"]["run"], {
            "rustc_link_lib": True, "rustc_link_search": True,
            "env": {"NUM_JOBS": "4", "AWS_LC_SYS_USE_SYSTEM": "0"},
        })
        graph = self.read("third-party/rust/BUCK")
        native = graph.split('name = "aws-lc-sys-0.45-build-script-main-run",', 1)[1].split("\n)\n", 1)[0]
        self.assertIn('"AWS_LC_SYS_USE_SYSTEM": "0"', native)
        self.assertIn("rustc_link_lib = True,", native)
        self.assertIn("rustc_link_search = True,", native)
        rustls = graph.split('name = "rustls-0.23",', 1)[1].split("\n)\n", 1)[0]
        self.assertIn('":aws-lc-rs-1"', rustls)
        self.assertNotIn('":ring-0.17"', rustls)


class ReindeerColdPatchTests(unittest.TestCase):
    # Exact post-images of the original patches applied to the authenticated
    # archive with Apple patch; changing context must preserve these bytes.
    POST_IMAGES = {
        'Cargo.toml': '9e8dec432b6830633403b3d5a1115f14b452e4e670ba7346fb9de01830b981da',
        'src/buckify.rs': 'cac71351a23e78ff21263a6b234dfd388b28cce6d3590885494762ea79525101',
        'src/cargo.rs': '3de5446764cde0758dd225fd998e533801c5729bfd3e05c93711ed4142bce94b',
        'src/config.rs': '153c72c9dc36001835ac2e6487b2951be7e677a84b853f9b752c81acba6417bb',
        'src/index.rs': '11aa73245b0d4d329a8ed3d16e931a6716ef2ed9a885692867bf5b527a837ed5',
    }

    def archive(self):
        lock = dict(line.split('=', 1) for line in
                    (ROOT / 'third-party/rust/reindeer/upstream.lock').read_text().splitlines()
                    if line.startswith('REINDEER_'))
        fixture = ROOT / ('tools/buck/fixtures/reindeer-upstream-'
                          '681727ced54a853977ac495e147ac54e1c0db115.tar.gz')
        self.assertEqual(lock['REINDEER_ARCHIVE_SHA256'],
                         hashlib.sha256(fixture.read_bytes()).hexdigest())
        return fixture, 'reindeer-' + lock['REINDEER_COMMIT']

    def test_cold_patch_fixture_is_the_exact_pinned_upstream_archive(self):
        fixture, prefix = self.archive()
        with tarfile.open(fixture) as archive:
            for relative in self.POST_IMAGES:
                self.assertTrue(archive.getmember(prefix + '/' + relative).isfile())

    def test_both_cold_patches_preserve_exact_source_on_available_patch_tools(self):
        fixture, prefix = self.archive()
        tools = []
        for name in ['patch', 'gpatch']:
            executable = shutil.which(name)
            if executable and str(Path(executable).resolve()) not in tools:
                tools.append(str(Path(executable).resolve()))
        self.assertTrue(tools, 'cold patch proof requires an executable patch tool')
        for executable in tools:
            with self.subTest(patch=executable), tempfile.TemporaryDirectory(
                    prefix='reindeer-cold-patch-') as tmp:
                with tarfile.open(fixture) as archive:
                    archive.extractall(tmp, filter='data')
                tree = Path(tmp) / prefix
                expected = {entry.relative_to(tree).as_posix():
                            hashlib.sha256(entry.read_bytes()).hexdigest()
                            for entry in tree.rglob('*')
                            if entry.is_file() and not entry.is_symlink()}
                expected.update(self.POST_IMAGES)
                for patch_name in ['0001-workspace-dev-dependency-roots.patch',
                                   '0002-cargo-security-uplift.patch']:
                    patch_file = ROOT / 'third-party/rust/reindeer/patches' / patch_name
                    argv = [executable, '--batch', '--fuzz=0', '--forward',
                            '-d', str(tree), '-p1', '--input', str(patch_file)]
                    result = subprocess.run(argv, capture_output=True, text=True, timeout=10)
                    self.assertEqual(0, result.returncode,
                                     patch_name + '\n' + result.stdout + result.stderr)
                for relative, sha in expected.items():
                    self.assertEqual(sha, hashlib.sha256((tree / relative).read_bytes()).hexdigest(),
                                     relative)


if __name__ == "__main__":
    unittest.main()
