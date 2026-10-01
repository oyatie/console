#!/usr/bin/env python3
"""Actual installer regression; local download transport and npm/deps are mocked.

Required fixture: CONSOLE_BROWSER_INSTALL_TEST_ARCHIVE names the retained official
Linux archive. Hashing and extraction are real. No Linux browser is executed.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
INSTALLER = ROOT / "tools/browser/prepare_native_browser.sh"
URL = "https://cdn.playwright.dev/builds/cft/153.0.8010.12/linux64/chrome-headless-shell-linux64.zip"
ARCHIVE_SHA = "a9da028861a0cf789ff25c2fed45f5f1aaf969ed9247835b6a7821a4f7af9d1d"
EXECUTABLE_SHA = "ded93a9c9a53a1ae040f08124badcca95c938e9d5015ff340c3b5538c41bf39e"
EVIDENCE = "docs/evidence/console/integration/2026-09-19-root-entry-browser/"
PACKAGE_FILES = {
    "playwright-package.json": "8d57d95d41a1c2833b846f382610db55b8d193c20e3b2473c1e59f43e798a9b9",
    "playwright-package-lock.json": "a9c22966fb530b30d45f4f17faca408679a0405f3978fdaa9abd6b1857578044",
    "playwright-browsers.json": "545d52f8382c391e605562c330e9c1c534a16045898203037a49bb8bd769a946",
}
DRIVERS = {
    "account.cjs": "3e59f4f63cce565fee6cd7da94c4f6dc81bba05d7a4e6b309ecc2e13f88c161d",
    "company.cjs": "2615f27c3609f6be183ddb6b0c0a868a20a364767ef02f05f8fea011367ca77c",
    "company-preview.cjs": "a1c4c3ad5b1cf5a5c01c86795b4e0db10d3b6c233a7a7d2f84065735ea919d90",
    "hydration.cjs": "fcb0d0b95981833017d47f2723459879478640e1faed8f4065cac1a0d6a5ac1c",
}
COMPANIONS = {
    "native_header.cjs": "3e8cf19a45086dc07cf0ddba012404e9d1c9614b43a0c7363eb18deefeab00e3",
    "people_journey.cjs": "7d6a445929d5b905d02fc60b063fa89a67db7bd3cd4b1a853ed666db150cdea2",
    "policy_journey.cjs": "766da87a7ad3c7c6c41dd98725ddbda951477d1e7b43e5961e213d92c4d5f4ea",
    "recovery_controls.cjs": "fe3afc43196cc034d6a5f9bd0b12ad787eaeddf2cb1b1b837b595cb9f49036c6",
}
FAMILIES = {
    "CONSOLE_BROWSER_JOURNEY": "account.cjs",
    "CONSOLE_COMPANY_BROWSER": "company.cjs",
    "CONSOLE_COMPANY_PREVIEW_BROWSER": "company-preview.cjs",
    "CONSOLE_HYDRATION_BROWSER": "hydration.cjs",
}
SENTINEL = b"EXISTING_REVIEWED_VALUE=preserve-me\n"
SECRET = "INSTALLER_TEST_ONLY_SECRET_CANARY"


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


# Installed as four isolated executable names. curl only moves retained bytes;
# unzip records invocation then delegates to the actual host unzip unchanged.
MOCK_TOOL = r'''#!/usr/bin/env python3
import json, os, pathlib, shutil, subprocess, sys
name = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
events = pathlib.Path(os.environ["CONSOLE_BROWSER_INSTALL_TEST_EVENTS"])
def record(kind, **fields):
    with events.open("a") as stream:
        stream.write(json.dumps({"kind":kind, **fields}) + "\n")
if name == "uname":
    system = os.environ.get("CONSOLE_BROWSER_INSTALL_TEST_SYSTEM", "Linux")
    machine = os.environ.get("CONSOLE_BROWSER_INSTALL_TEST_MACHINE", "x86_64")
    if args == ["-s"]: print(system)
    elif args == ["-m"]: print(machine)
    elif args in (["-sm"], ["-s", "-m"]): print(system + " " + machine)
    else: sys.exit("unexpected uname invocation")
elif name == "curl":
    url = os.environ["CONSOLE_BROWSER_INSTALL_TEST_URL"]
    assert args.count(url) == 1, "installer must use the exact reviewed URL"
    assert [arg for arg in args if arg.startswith("https://")] == [url]
    assert "--fail" in args and "--location" in args
    output_flags = [i for i, arg in enumerate(args) if arg in ("-o", "--output")]
    assert len(output_flags) == 1
    destination = pathlib.Path(args[output_flags[0] + 1])
    assert destination.is_absolute()
    record("curl", url=url, destination=str(destination))
    shutil.copyfile(os.environ["CONSOLE_BROWSER_INSTALL_TEST_ARCHIVE"], destination)
elif name == "unzip":
    record("unzip", args=args)
    os.execv(os.environ["CONSOLE_BROWSER_INSTALL_TEST_REAL_UNZIP"], ["unzip", *args])
elif name == "npm":
    if args == ["--version"]:
        print("10.9.0")
        sys.exit(0)
    if "--prefix" in args:
        i = args.index("--prefix")
        runtime = pathlib.Path(args[i + 1])
        args = args[:i] + args[i + 2:]
    else:
        runtime = pathlib.Path.cwd()
    assert args[0] == "ci"
    assert set(args[1:]) == {"--ignore-scripts", "--no-audit", "--no-fund"}
    assert json.loads((runtime / "package.json").read_text())["dependencies"]["playwright"] == "1.63.0"
    assert (runtime / "package-lock.json").is_file()
    record("npm-ci", runtime=str(runtime))
    playwright = runtime / "node_modules/playwright"
    core = runtime / "node_modules/playwright-core"
    playwright.mkdir(parents=True)
    core.mkdir(parents=True)
    (playwright / "package.json").write_text(json.dumps({"name":"playwright","version":"1.63.0"}))
    (core / "package.json").write_text(json.dumps({"name":"playwright-core","version":"1.63.0"}))
    shutil.copyfile(os.environ["CONSOLE_BROWSER_INSTALL_TEST_BROWSERS"], core / "browsers.json")
    (playwright / "cli.js").write_text("""
const fs = require('node:fs');
if (JSON.stringify(process.argv.slice(2)) !== JSON.stringify(['install-deps','chromium-headless-shell'])) process.exit(91);
fs.appendFileSync(process.env.CONSOLE_BROWSER_INSTALL_TEST_EVENTS, JSON.stringify({kind:'install-deps', args:process.argv.slice(2)})+'\\n');
""")
else:
    sys.exit("unexpected test tool")
'''


class PrepareNativeBrowserTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        value = os.environ.get("CONSOLE_BROWSER_INSTALL_TEST_ARCHIVE")
        if not value:
            raise RuntimeError("prerequisite: set CONSOLE_BROWSER_INSTALL_TEST_ARCHIVE to the retained official archive")
        cls.archive = Path(value).resolve(strict=True)
        if sha256(cls.archive) != ARCHIVE_SHA:
            raise RuntimeError("prerequisite: retained Linux archive differs from reviewed custody")
        cls.unzip = shutil.which("unzip")
        for tool in ("python3", "node", "openssl", "bash"):
            if shutil.which(tool) is None:
                raise RuntimeError("prerequisite: missing host tool " + tool)
        if cls.unzip is None:
            raise RuntimeError("prerequisite: missing actual unzip")
        for name, expected in PACKAGE_FILES.items():
            if sha256(ROOT / EVIDENCE / name) != expected:
                raise RuntimeError("prerequisite: package source pin mismatch " + name)
        for name, expected in {**DRIVERS, **COMPANIONS}.items():
            if sha256(ROOT / "tools/browser" / name) != expected:
                raise RuntimeError("prerequisite: reviewed browser source pin mismatch " + name)

    def setUp(self):
        self.assertTrue(INSTALLER.is_file(), "NATIVE_BROWSER_INSTALLER_REQUIRED")
        temporary = tempfile.TemporaryDirectory(prefix="console-browser-installer-regression-")
        self.addCleanup(temporary.cleanup)
        self.scratch = Path(temporary.name).resolve()
        self.repo = self.scratch / "checkout"
        self.runner = self.scratch / "runner"
        self.runner.mkdir()
        self.script = self.repo / "tools/browser/prepare_native_browser.sh"
        self.script.parent.mkdir(parents=True)
        shutil.copy2(INSTALLER, self.script)
        for name in {**DRIVERS, **COMPANIONS}:
            shutil.copy2(ROOT / "tools/browser" / name, self.script.parent / name)
        sources = self.repo / EVIDENCE
        sources.mkdir(parents=True)
        for name in PACKAGE_FILES:
            shutil.copy2(ROOT / EVIDENCE / name, sources / name)
        self.env_file = self.scratch / "github.env"
        self.env_file.write_bytes(SENTINEL)
        self.events = self.scratch / "events.jsonl"
        self.events.write_text("")
        fake_bin = self.scratch / "mock-bin"
        fake_bin.mkdir()
        for name in ("uname", "curl", "npm", "unzip"):
            script = fake_bin / name
            script.write_text(MOCK_TOOL)
            script.chmod(0o755)
        # Preserve normal tool discovery but exclude shell/runtime injection knobs.
        self.env = {key: value for key, value in os.environ.items()
                    if key not in {"BASH_ENV", "ENV", "NODE_OPTIONS", "NODE_PATH", "SHELLOPTS", "CDPATH"}}
        self.env.update({
            "PATH": str(fake_bin) + os.pathsep + os.environ.get("PATH", ""),
            "RUNNER_TEMP": str(self.runner), "GITHUB_ENV": str(self.env_file),
            "CONSOLE_BROWSER_INSTALL_TEST_ARCHIVE": str(self.archive),
            "CONSOLE_BROWSER_INSTALL_TEST_REAL_UNZIP": self.unzip,
            "CONSOLE_BROWSER_INSTALL_TEST_EVENTS": str(self.events),
            "CONSOLE_BROWSER_INSTALL_TEST_URL": URL,
            "CONSOLE_BROWSER_INSTALL_TEST_BROWSERS": str(sources / "playwright-browsers.json"),
            "CONSOLE_BROWSER_INSTALL_TEST_SYSTEM": "Linux",
            "CONSOLE_BROWSER_INSTALL_TEST_MACHINE": "x86_64",
            "DATABASE_URL": "postgres://fixture:" + SECRET + "@localhost/unused",
        })

    def invoke(self, env=None):
        result = subprocess.run(["bash", str(self.script)], cwd=self.repo,
                                env=self.env if env is None else env,
                                capture_output=True, text=True, timeout=120)
        self.assertNotIn(SECRET, result.stdout + result.stderr)
        self.assertNotIn(SECRET.encode(), self.env_file.read_bytes())
        return result

    def calls(self):
        return [json.loads(line) for line in self.events.read_text().splitlines()]

    def assert_failure(self, env=None):
        result = self.invoke(env)
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertEqual(SENTINEL, self.env_file.read_bytes(), "failed installer published partial environment")
        for family in FAMILIES:
            for suffix in ("DRIVER", "SHA256", "OUTPUT"):
                self.assertNotIn(family + "_" + suffix + "=", result.stdout + result.stderr)
        return result

    def verify_stage(self):
        raw = self.env_file.read_bytes()
        self.assertTrue(raw.startswith(SENTINEL))
        lines = raw[len(SENTINEL):].decode().splitlines()
        self.assertEqual(12, len(lines), "only twelve public browser settings may be published")
        pairs = [line.split("=", 1) for line in lines]
        self.assertTrue(all(len(pair) == 2 for pair in pairs))
        values = dict(pairs)
        expected_names = {family + "_" + suffix for family in FAMILIES for suffix in ("DRIVER", "SHA256", "OUTPUT")}
        self.assertEqual(expected_names, set(values))
        stage = Path(values["CONSOLE_BROWSER_JOURNEY_DRIVER"]).parent
        self.assertTrue(stage.is_absolute())
        self.assertEqual(self.runner, stage.parent)
        self.assertTrue(stage.name.startswith("console-native-browser."))
        self.assertEqual(0o700, stat.S_IMODE(stage.stat().st_mode))
        outputs = set()
        for family, name in FAMILIES.items():
            driver = Path(values[family + "_DRIVER"])
            output = Path(values[family + "_OUTPUT"])
            self.assertEqual(stage / name, driver)
            self.assertFalse(driver.is_symlink())
            self.assertEqual(DRIVERS[name], sha256(driver))
            self.assertEqual(DRIVERS[name], values[family + "_SHA256"])
            self.assertTrue(output.is_absolute() and output.is_relative_to(stage))
            self.assertTrue(output.parent.is_dir())
            self.assertFalse(output.exists(), "browser leaf output must remain fresh")
            outputs.add(output)
        self.assertEqual(4, len(outputs))
        for name, expected in COMPANIONS.items():
            companion = stage / name
            self.assertFalse(companion.is_symlink())
            self.assertEqual(expected, sha256(companion))
        runtime = stage / "runtime"
        self.assertEqual(PACKAGE_FILES["playwright-package.json"], sha256(runtime / "package.json"))
        self.assertEqual(PACKAGE_FILES["playwright-package-lock.json"], sha256(runtime / "package-lock.json"))
        self.assertEqual(PACKAGE_FILES["playwright-browsers.json"], sha256(runtime / "node_modules/playwright-core/browsers.json"))
        binary = runtime / "browser/chrome-headless-shell-linux64/chrome-headless-shell"
        self.assertEqual(EXECUTABLE_SHA, sha256(binary))
        self.assertTrue(os.access(binary, os.X_OK))
        # ZIP siblings are operational inputs, not merely a single copied executable.
        self.assertTrue((binary.parent / "icudtl.dat").is_file())
        self.assertTrue((binary.parent / "v8_context_snapshot.bin").is_file())
        evidence_bytes = (stage / "prerequisites.json").read_bytes()
        self.assertNotIn(SECRET.encode(), evidence_bytes)
        evidence = json.loads(evidence_bytes)
        for tool in ("node", "npm", "openssl"):
            self.assertIsInstance(evidence["tools"][tool], str)
            self.assertTrue(evidence["tools"][tool])
        self.assertEqual("1.63.0", evidence["versions"]["playwright"])
        self.assertEqual("153.0.8010.12", evidence["versions"]["chromium"])
        self.assertEqual("1243", str(evidence["versions"]["revision"]))
        for key, expected in {
            "package": PACKAGE_FILES["playwright-package.json"],
            "lock": PACKAGE_FILES["playwright-package-lock.json"],
            "browsers": PACKAGE_FILES["playwright-browsers.json"],
            "archive": ARCHIVE_SHA, "executable": EXECUTABLE_SHA,
        }.items():
            self.assertEqual(expected, evidence["digests"][key])
        self.assertEqual(DRIVERS, evidence["digests"]["drivers"])
        self.assertEqual(COMPANIONS, evidence["digests"]["companions"])
        return stage

    def test_real_archive_staging_is_pinned_fresh_and_publishes_only_after_success(self):
        stages = []
        for _ in range(2):
            self.env_file.write_bytes(SENTINEL)
            self.events.write_text("")
            result = self.invoke()
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            stages.append(self.verify_stage())
            kinds = [call["kind"] for call in self.calls()]
            for required in ("curl", "unzip", "npm-ci", "install-deps"):
                self.assertEqual(1, kinds.count(required), kinds)
        self.assertNotEqual(stages[0], stages[1], "second invocation reused prior evidence or outputs")
        self.assertEqual(DRIVERS["account.cjs"], sha256(stages[0] / "account.cjs"))

    def test_unsupported_platform_publishes_nothing(self):
        self.env["CONSOLE_BROWSER_INSTALL_TEST_SYSTEM"] = "FreeBSD"
        self.env["CONSOLE_BROWSER_INSTALL_TEST_MACHINE"] = "riscv64"
        self.assert_failure()
        self.assertEqual([], self.calls())

    def test_bad_archive_is_rejected_before_real_unzip(self):
        bad = self.scratch / "wrong-browser.zip"
        bad.write_bytes(b"deliberately not the reviewed archive\n")
        self.env["CONSOLE_BROWSER_INSTALL_TEST_ARCHIVE"] = str(bad)
        self.assert_failure()
        kinds = [call["kind"] for call in self.calls()]
        self.assertEqual(1, kinds.count("curl"))
        self.assertNotIn("unzip", kinds, "untrusted archive reached extraction")

    def test_changed_driver_publishes_nothing(self):
        with (self.script.parent / "account.cjs").open("a") as stream:
            stream.write("\n// injected source corruption\n")
        self.assert_failure()

    def test_changed_company_driver_publishes_nothing(self):
        with (self.script.parent / "company.cjs").open("a") as stream:
            stream.write("\n// injected Company source corruption\n")
        self.assert_failure()

    def test_changed_people_companion_publishes_nothing(self):
        with (self.script.parent / "people_journey.cjs").open("a") as stream:
            stream.write("\n// injected People source corruption\n")
        self.assert_failure()

    def test_changed_companion_publishes_nothing(self):
        with (self.script.parent / "native_header.cjs").open("a") as stream:
            stream.write("\n// injected source corruption\n")
        self.assert_failure()

    def test_missing_companion_publishes_nothing(self):
        (self.script.parent / "people_journey.cjs").unlink()
        self.assert_failure()

    def test_newline_runner_path_publishes_nothing(self):
        unsafe = self.scratch / "runner\nINJECTED=value"
        unsafe.mkdir()
        self.env["RUNNER_TEMP"] = str(unsafe)
        self.assert_failure()
        self.assertEqual([], self.calls())

    def test_missing_github_env_publishes_nothing(self):
        env = dict(self.env)
        env.pop("GITHUB_ENV")
        self.assert_failure(env)
        self.assertEqual([], self.calls())


if __name__ == "__main__":
    unittest.main()
