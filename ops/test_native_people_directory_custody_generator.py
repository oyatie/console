"""New capture generation only; actual database custody is independently tested."""
import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('directory_custody_generator', ROOT / 'ops/generate-account-custody.py')
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)

OLD_POLICY_OUTPUTS = {'ops/postgres-native-company-policy-owner.sql': '833300caeac6874f35ebb08017bfbb7891a5fe765a4c761b139466ae221573d5', 'ops/postgres-capture-native-company-policy-custody.sql': 'd5d2c45c691383ddde9ac213d9fe2674db0750f3ceabdc733eaaf2a570d937e3', 'ops/postgres-native-company-policy-custody-state.sql': '072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479', 'backend/app/src/native_company_policy_custody_state.sql': '072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479', 'ops/postgres-finalize-native-company-policy.sql': '4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a', 'ops/postgres-native-company-policy-v2-owner.sql': 'f2050f21ef8151339289b2f013abdd543e90f8803fb6f6d4b9f004abf4409602', 'ops/postgres-capture-native-company-policy-v2-custody.sql': '7534375fbae287ccf5e5015815e788ef0d7a379eed623af9a72f5db0d89614b4', 'ops/postgres-native-company-policy-v2-custody-state.sql': 'e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc', 'backend/app/src/native_company_policy_v2_custody_state.sql': 'e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc', 'ops/postgres-finalize-native-company-policy-v2.sql': 'ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e'}
CAPTURE_OUTPUTS = {'ops/postgres-native-people-directory-owner.sql': '86ab1703f6a236952764f048aed3dd879daa44daa9c0abfe1a1c7a8501efb92a', 'ops/postgres-capture-native-people-directory-custody.sql': 'bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7', 'ops/postgres-capture-native-people-directory-staged-custody.sql': '890ebc034fe9836f45e26c13d05352085db753f17e4a72b1ec05b6ecbe0a0e36'}

class NativeDirectoryCaptureGeneration(unittest.TestCase):
    def test_exact_declared_capture_outputs_and_rosters(self):
        outputs = GENERATOR.native_people_directory_capture_files()
        self.assertEqual(set(outputs), set(CAPTURE_OUTPUTS))
        for name, digest in CAPTURE_OUTPUTS.items():
            self.assertEqual(hashlib.sha256(outputs[name].encode()).hexdigest(), digest, name)
        self.assertEqual(len(GENERATOR.native_people_directory_routine_names()), 27)
        self.assertEqual(len(set(GENERATOR.NATIVE_DIRECTORY_ADDED_RELATIONS)), 12)

    def test_old_policy_outputs_keep_original_meaning_and_bytes(self):
        outputs = {**GENERATOR.native_company_policy_generated_files(),
                   **GENERATOR.native_company_policy_v2_capture_files(),
                   **GENERATOR.native_company_policy_v2_finalized_files()}
        for name, digest in OLD_POLICY_OUTPUTS.items():
            self.assertEqual(hashlib.sha256(outputs[name].encode()).hexdigest(), digest, name)

    def test_altered_declared_source_is_rejected_before_capture_generation(self):
        originals = GENERATOR.native_people_directory_source_files()
        self.assertEqual(len(originals), 8)
        with tempfile.TemporaryDirectory() as temporary:
            temporary = Path(temporary)
            for name, content in originals.items():
                target = temporary / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content)
            with patch.object(GENERATOR, 'ROOT', temporary):
                self.assertEqual(GENERATOR.native_people_directory_source_files(), originals)
                target = temporary / 'ops/native-people-directory/activation-v1.sql'
                target.write_text(originals['ops/native-people-directory/activation-v1.sql'] + '\n')
                with self.assertRaisesRegex(SystemExit, 'differs from reviewed bytes'):
                    GENERATOR.native_people_directory_capture_files()

    def test_symlink_declared_source_is_rejected_even_with_identical_bytes(self):
        originals = GENERATOR.native_people_directory_source_files()
        with tempfile.TemporaryDirectory() as temporary:
            temporary = Path(temporary)
            for name, content in originals.items():
                target = temporary / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content)
            target = temporary / 'ops/native-people-directory/activation-v1.sql'
            copied = temporary / 'exact-bytes.sql'
            target.rename(copied)
            target.symlink_to(copied)
            with patch.object(GENERATOR, 'ROOT', temporary):
                with self.assertRaisesRegex(SystemExit, 'must be a regular file'):
                    GENERATOR.native_people_directory_capture_files()

if __name__ == '__main__':
    unittest.main()
