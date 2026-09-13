"""Bounded source-custody tests, not an end-to-end information-flow gate."""

import hashlib
import json
from pathlib import Path
import unittest


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
RECORD = json.loads((HERE / "verus-source-binding.json").read_text())
SOURCE = ROOT / RECORD["source_path"]


def declaration(text, name):
    marker = f"pub struct {name} {{"
    if text.count(marker) != 1:
        raise ValueError("ambiguous or missing declaration")
    start = text.index(marker)
    end = text.index("\n}", start) + 2
    return text[start:end]


def method(text, verified=False):
    marker = "    pub const fn satisfies("
    if text.count(marker) != 1:
        raise ValueError("ambiguous or missing method")
    start = text.index(marker)
    end = text.index("\n    }", start) + 6
    full = text[start:end]
    signature = (
        "    pub const fn satisfies(self, required: SubjectFreshnessRequirement)"
    )
    expected = signature + (" -> (ok: bool)\n" if verified else " -> bool {\n")
    if not full.startswith(expected):
        raise ValueError("signature changed")
    body_marker = "\n    {\n" if verified else " {\n"
    begin = full.index(body_marker) + len(body_marker)
    return full, full[begin : full.rindex("\n    }")]


def assert_bound(source, projection, verified):
    full, body = method(source)
    if hashlib.sha256(full.encode()).hexdigest() != RECORD["method_sha256"]:
        raise ValueError("production source changed since reviewed extraction")
    for name in ("SubjectFreshness", "SubjectFreshnessRequirement"):
        if declaration(source, name) != declaration(projection, name):
            raise ValueError("field/type mismatch")
    if body != method(projection, verified)[1]:
        raise ValueError("executable body mismatch")


class SourceBinding(unittest.TestCase):
    def test_full_production_source_matches_reviewed_git_blob(self):
        raw = SOURCE.read_bytes()
        blob = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
        self.assertEqual(blob, RECORD["source_git_blob"])

    def test_all_reviewed_fixtures_match_frozen_hashes(self):
        self.assertEqual(set(RECORD["fixture_sha256"]), {f"source-bound-{name}.rs" for name in ("native", "positive", "known-false", "mutant")})
        for name, expected in RECORD["fixture_sha256"].items():
            with self.subTest(name=name):
                self.assertEqual(hashlib.sha256((HERE / name).read_bytes()).hexdigest(), expected)

    def test_actual_native_and_verified_bodies_are_identical(self):
        source = SOURCE.read_text()
        for file, verified in (("native", False), ("positive", True), ("known-false", True)):
            with self.subTest(file=file):
                assert_bound(source, (HERE / f"source-bound-{file}.rs").read_text(), verified)

    def test_security_mutation_is_not_accepted_as_production_body(self):
        with self.assertRaisesRegex(ValueError, "body mismatch"):
            assert_bound(SOURCE.read_text(), (HERE / "source-bound-mutant.rs").read_text(), True)

    def test_changed_production_source_invalidates_previous_proof(self):
        source = SOURCE.read_text().replace("Some(actual_step_up) => actual_step_up >= required_step_up", "Some(actual_step_up) => actual_step_up > required_step_up", 1)
        with self.assertRaisesRegex(ValueError, "source changed"):
            assert_bound(source, (HERE / "source-bound-positive.rs").read_text(), True)

    def test_field_type_substitution_is_rejected(self):
        projection = (HERE / "source-bound-positive.rs").read_text().replace("pub policy_version: u64", "pub policy_version: u32", 1)
        with self.assertRaisesRegex(ValueError, "field/type mismatch"):
            assert_bound(SOURCE.read_text(), projection, True)

    def test_missing_or_duplicate_method_is_not_silently_selected(self):
        projection = (HERE / "source-bound-positive.rs").read_text()
        for malformed in (projection.replace("pub const fn satisfies", "pub const fn other"), projection + projection):
            with self.subTest(malformed=malformed[:32]), self.assertRaises(ValueError):
                assert_bound(SOURCE.read_text(), malformed, True)


if __name__ == "__main__":
    unittest.main()
