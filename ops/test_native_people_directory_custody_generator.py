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
CAPTURE_OUTPUTS = {'ops/postgres-native-people-directory-owner.sql': '3dd0524bc751eba2aa806ee0dc670dc3cecbf8838c20b83896773a0c5b6016a0', 'ops/postgres-capture-native-people-directory-custody.sql': 'bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7', 'ops/postgres-capture-native-people-directory-staged-custody.sql': '890ebc034fe9836f45e26c13d05352085db753f17e4a72b1ec05b6ecbe0a0e36'}

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
        self.assertEqual(len(originals), 9)
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


# Frozen independently reviewed predecessor bytes, not recomputed expectations.
DIRECTORY_ROW_LOCK_PREDECESSOR_OUTPUTS = {'backend/app/src/account_custody_state.sql': '5fb0a2608c5904fe0c1d58723290b9603afb9af08eef19c410d023a484230865', 'ops/postgres-finalize-account-custody.sql': 'fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8', 'ops/account-custody-migrations.sha384': '25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325', 'ops/postgres-verify-account-native.sql': 'c4e30ee9560b4d443acad1813a63667cbab9a32aadfbd893542f4b054bf6b308', 'ops/postgres-company-enrollment-input.sql': 'cbc685bff861fec809b930e57673cce5426a771a236323471955510de0631290', 'ops/postgres-company-enrollment-schema.sql': '7e1dbee080cb3f09d0133ac9a30857ee5101a076e1cec171ec06849116e672bb', 'ops/postgres-company-enrollment-intake.sql': '2d8e5bc991e35ca1c8195d5ee9f0b5c2f367a96e4d77cf613945f9d533cc7532', 'ops/postgres-company-enrollment-guards.sql': 'e7ad2c8343fa596073ad82842b8eae9e2a67eb0aba8903daf86b853e59a4575a', 'ops/postgres-company-enrollment-owner.sql': '2a7fffb57eedff3c3ba5981015e22f53ea80972e3222c38ff1c6ace1f0858d2d', 'ops/postgres-capture-company-enrollment-custody.sql': '6ef7b2754176952da89a0b61a7ba999d64b4c12e70fc949add18c63171b0abb5', 'backend/app/src/company_enrollment_custody_state.sql': 'fb2030a4d2891b206c41454ce8258d65920034e8ef817c5f7a812569aae614bd', 'ops/postgres-finalize-company-enrollment.sql': 'bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f', 'backend/app/src/account_credential_custody_state.sql': '0bc6fe6579414ebed546d076c0ec1b7f1cc5ba30f4fe5e135159f4e75c0416a2', 'ops/postgres-finalize-account-credentials.sql': '2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9', 'ops/postgres-native-company-policy-owner.sql': '833300caeac6874f35ebb08017bfbb7891a5fe765a4c761b139466ae221573d5', 'ops/postgres-capture-native-company-policy-custody.sql': 'd5d2c45c691383ddde9ac213d9fe2674db0750f3ceabdc733eaaf2a570d937e3', 'ops/postgres-native-company-policy-custody-state.sql': '072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479', 'backend/app/src/native_company_policy_custody_state.sql': '072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479', 'ops/postgres-finalize-native-company-policy.sql': '4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a', 'ops/postgres-native-company-policy-v2-owner.sql': 'f2050f21ef8151339289b2f013abdd543e90f8803fb6f6d4b9f004abf4409602', 'ops/postgres-capture-native-company-policy-v2-custody.sql': '7534375fbae287ccf5e5015815e788ef0d7a379eed623af9a72f5db0d89614b4', 'ops/postgres-native-company-policy-v2-custody-state.sql': 'e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc', 'backend/app/src/native_company_policy_v2_custody_state.sql': 'e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc', 'ops/postgres-finalize-native-company-policy-v2.sql': 'ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e', 'ops/postgres-native-people-directory-owner.sql': '3dd0524bc751eba2aa806ee0dc670dc3cecbf8838c20b83896773a0c5b6016a0', 'ops/postgres-capture-native-people-directory-custody.sql': 'bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7', 'ops/postgres-capture-native-people-directory-staged-custody.sql': '890ebc034fe9836f45e26c13d05352085db753f17e4a72b1ec05b6ecbe0a0e36', 'ops/postgres-native-people-directory-custody-state.sql': '20c96bc2a9264d5ed4b86cb948cbe0574450243509263469c45af955d5aff3dd', 'backend/app/src/native_people_directory_custody_state.sql': '20c96bc2a9264d5ed4b86cb948cbe0574450243509263469c45af955d5aff3dd', 'ops/postgres-finalize-native-people-directory.sql': 'f4f99cf873c2ab970789e44ccf9737f2dd38f6dc6b05f1849fbd4461bf6a2357'}

class NativeDirectoryRowLockGeneration(unittest.TestCase):
    def test_corrupted_predecessor_and_current_ledgers_are_refused(self):
        outputs = GENERATOR.generated_files()
        name = 'ops/account-custody-migrations.sha384'
        records = outputs[name].splitlines(keepends=True)
        corrupted = [
            '9' + outputs[name][1:],  # Changed historical version bytes.
            ''.join(records[:-1]) + records[-1].replace('231\t', '232\t', 1),
            ''.join(records[:-1]) + records[-1].replace('\t', '\t0', 1),
            ''.join(records[:-1]),  # Missing current expansion.
            outputs[name] + records[-1],  # Unreviewed extra record.
        ]
        for ledger in corrupted:
            with self.subTest(ledger_sha256=hashlib.sha256(ledger.encode()).hexdigest()):
                with patch.object(GENERATOR, 'generated_files', return_value={**outputs, name: ledger}):
                    with self.assertRaises(AssertionError):
                        self.test_all_thirty_predecessor_outputs_and_embedded_finalizer_remain_exact()

    def test_all_thirty_predecessor_outputs_and_embedded_finalizer_remain_exact(self):
        outputs = GENERATOR.generated_files()
        self.assertEqual(len(DIRECTORY_ROW_LOCK_PREDECESSOR_OUTPUTS), 30)
        for name, expected in DIRECTORY_ROW_LOCK_PREDECESSOR_OUTPUTS.items():
            content = outputs[name].encode()
            if name == 'ops/account-custody-migrations.sha384':
                records = content.splitlines(keepends=True)
                self.assertEqual(len(records), 231, 'exact reviewed migration roster')
                # The original independent digest continues to attest every
                # predecessor byte; the current expansion has its own exact pin.
                self.assertEqual(hashlib.sha256(b''.join(records[:230])).hexdigest(), expected, name)
                self.assertEqual(hashlib.sha256(content).hexdigest(),
                                 '42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357',
                                 'reviewed 231-migration ledger drift')
            else:
                self.assertEqual(hashlib.sha256(content).hexdigest(), expected, name)
        self.assertEqual(set(outputs) - set(DIRECTORY_ROW_LOCK_PREDECESSOR_OUTPUTS), {
            'ops/postgres-native-people-directory-row-lock-custody-state.sql',
            'backend/app/src/native_people_directory_row_lock_custody_state.sql',
            'ops/postgres-finalize-native-people-directory-row-lock.sql',
        })
        self.assertEqual(outputs['ops/postgres-native-people-directory-row-lock-custody-state.sql'],
                         outputs['backend/app/src/native_people_directory_row_lock_custody_state.sql'])
        envelope = outputs['ops/postgres-finalize-native-people-directory-row-lock.sql']
        pieces = envelope.split('$native_people_directory_v1_input$')
        self.assertEqual(len(pieces), 3)
        self.assertEqual(pieces[1], outputs['ops/postgres-finalize-native-people-directory.sql'])
        self.assertEqual(hashlib.sha256(pieces[1].encode()).hexdigest(),
                         'f4f99cf873c2ab970789e44ccf9737f2dd38f6dc6b05f1849fbd4461bf6a2357')

    def test_corrected_pair_requires_two_distinct_measured_profiles(self):
        self.assertEqual(GENERATOR.native_people_directory_row_lock_profiles(), (
            ('e9891784422768abcb07f731adb1295c17b40c026236c1c4ea5b9993f6ddba9b',
             'bf87ef1475ec983c4e1bd286337687ead135b76fe70e28f79fe8cd430a1c95bc'),
            ('b0d8ced14929a0c4ef041dfceb57519c64663cb87f39c1dccf61d61227e2278e',
             '2c69786d88b784ca348725dc85730b9d8be1bc835e1069f64de7b7d3ec6ac80d')))
        correct = GENERATOR.NATIVE_DIRECTORY_ROW_LOCK_FINALIZED_SHA256
        for invalid in [(), correct[:1], (correct[0], correct[0]),
                        (GENERATOR.NATIVE_DIRECTORY_FINALIZED_SHA256[0], correct[1]),
                        ('x' * 64, correct[1]), (None, correct[1])]:
            with self.subTest(invalid=invalid):
                with patch.object(GENERATOR, 'NATIVE_DIRECTORY_ROW_LOCK_FINALIZED_SHA256', invalid):
                    with self.assertRaisesRegex(SystemExit, 'independently reviewed paired captures'):
                        GENERATOR.native_people_directory_row_lock_files()

    def test_correction_source_missing_changed_or_symlink_is_rejected(self):
        original = GENERATOR.native_people_directory_row_lock_source_sql()
        self.assertEqual(hashlib.sha256(original.encode()).hexdigest(),
                         'd3c48ec3134fd8f67241f0eb4a19d76b02ab51266821a1a1faec530893f67f79')
        with tempfile.TemporaryDirectory() as temporary:
            temporary = Path(temporary)
            target = temporary / GENERATOR.NATIVE_DIRECTORY_ROW_LOCK_SOURCE
            target.parent.mkdir(parents=True)
            with patch.object(GENERATOR, 'ROOT', temporary):
                with self.assertRaisesRegex(SystemExit, 'must be a regular file'):
                    GENERATOR.native_people_directory_row_lock_source_sql()
                target.write_text(original)
                self.assertEqual(GENERATOR.native_people_directory_row_lock_source_sql(), original)
                target.write_text(original + '\n')
                with self.assertRaisesRegex(SystemExit, 'differs from reviewed bytes'):
                    GENERATOR.native_people_directory_row_lock_source_sql()
                target.unlink()
                exact = temporary / 'exact-correction.sql'
                exact.write_text(original)
                target.symlink_to(exact)
                with self.assertRaisesRegex(SystemExit, 'must be a regular file'):
                    GENERATOR.native_people_directory_row_lock_source_sql()

    def test_changed_historical_sql_is_not_silently_reembedded(self):
        historical = GENERATOR.native_people_directory_finalizer_sql()
        with patch.object(GENERATOR, 'native_people_directory_finalizer_sql', return_value=historical + '\n'):
            with self.assertRaisesRegex(SystemExit, 'differs from reviewed historical bytes'):
                GENERATOR.native_people_directory_row_lock_finalizer_sql()

    def test_delimiter_collision_rejected_even_if_future_source_digest_is_updated(self):
        changed = GENERATOR.native_people_directory_finalizer_sql() + '$native_people_directory_v1_input$'
        with patch.object(GENERATOR, 'native_people_directory_finalizer_sql', return_value=changed), \
             patch.object(GENERATOR, 'NATIVE_DIRECTORY_V1_FINALIZER_SHA256', hashlib.sha256(changed.encode()).hexdigest()):
            with self.assertRaisesRegex(SystemExit, 'embedding delimiter collision'):
                GENERATOR.native_people_directory_row_lock_finalizer_sql()

if __name__ == '__main__':
    unittest.main()
