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


# Additive prerequisite tests for the independently reviewed Organization V2
# contract, SHA256 36e94f5fc1d068c5fafcdadc3e7c65850e00e44d611dae3da0879335a4567ded.
# Import through unittest's module loader: the preserved direct-script entry
# above intentionally retains its original ten-test behavior.
import json as _org_closed_json
import re as _org_closed_re
import shutil as _org_closed_shutil
import subprocess as _org_closed_subprocess
import sys as _org_closed_sys

_ORG_CLOSED_MODE = '--native-org-unit-closed-perimeter-custody'
_ORG_CLOSED_SCRIPT = 'ops/generate-account-custody.py'
_ORG_CLOSED_SOURCE = 'ops/native-org-unit/closed-perimeter-v1.sql'
_ORG_CLOSED_SOURCE_SHA256 = 'd57ec1df27184c7c0f8b6359a4ea72c08b7ce89e88c4d6054ca335d7e744cbcb'
_ORG_CLOSED_OUTPUTS = ('ops/postgres-native-org-unit-closed-perimeter-v1-custody-state.sql', 'backend/app/src/native_org_unit_closed_perimeter_v1_custody_state.sql', 'ops/postgres-finalize-native-org-unit-closed-perimeter-v1.sql')
_ORG_CLOSED_DEFAULTS = {'backend/app/src/account_credential_custody_state.sql': '0bc6fe6579414ebed546d076c0ec1b7f1cc5ba30f4fe5e135159f4e75c0416a2', 'backend/app/src/account_custody_state.sql': '5fb0a2608c5904fe0c1d58723290b9603afb9af08eef19c410d023a484230865', 'backend/app/src/company_enrollment_custody_state.sql': 'fb2030a4d2891b206c41454ce8258d65920034e8ef817c5f7a812569aae614bd', 'backend/app/src/native_company_policy_custody_state.sql': '072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479', 'backend/app/src/native_company_policy_v2_custody_state.sql': 'e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc', 'backend/app/src/native_people_directory_custody_state.sql': '20c96bc2a9264d5ed4b86cb948cbe0574450243509263469c45af955d5aff3dd', 'backend/app/src/native_people_directory_row_lock_custody_state.sql': '781cc446ce26525cbad6cd281c66239d1a7366eb9369d914068f39886cad3d6e', 'ops/account-custody-migrations.sha384': '42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357', 'ops/postgres-capture-company-enrollment-custody.sql': '6ef7b2754176952da89a0b61a7ba999d64b4c12e70fc949add18c63171b0abb5', 'ops/postgres-capture-native-company-policy-custody.sql': 'd5d2c45c691383ddde9ac213d9fe2674db0750f3ceabdc733eaaf2a570d937e3', 'ops/postgres-capture-native-company-policy-v2-custody.sql': '7534375fbae287ccf5e5015815e788ef0d7a379eed623af9a72f5db0d89614b4', 'ops/postgres-capture-native-people-directory-custody.sql': 'bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7', 'ops/postgres-capture-native-people-directory-staged-custody.sql': '890ebc034fe9836f45e26c13d05352085db753f17e4a72b1ec05b6ecbe0a0e36', 'ops/postgres-company-enrollment-guards.sql': 'e7ad2c8343fa596073ad82842b8eae9e2a67eb0aba8903daf86b853e59a4575a', 'ops/postgres-company-enrollment-input.sql': 'cbc685bff861fec809b930e57673cce5426a771a236323471955510de0631290', 'ops/postgres-company-enrollment-intake.sql': '2d8e5bc991e35ca1c8195d5ee9f0b5c2f367a96e4d77cf613945f9d533cc7532', 'ops/postgres-company-enrollment-owner.sql': '2a7fffb57eedff3c3ba5981015e22f53ea80972e3222c38ff1c6ace1f0858d2d', 'ops/postgres-company-enrollment-schema.sql': '7e1dbee080cb3f09d0133ac9a30857ee5101a076e1cec171ec06849116e672bb', 'ops/postgres-finalize-account-credentials.sql': '2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9', 'ops/postgres-finalize-account-custody.sql': 'fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8', 'ops/postgres-finalize-company-enrollment.sql': 'bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f', 'ops/postgres-finalize-native-company-policy-v2.sql': 'ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e', 'ops/postgres-finalize-native-company-policy.sql': '4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a', 'ops/postgres-finalize-native-people-directory-row-lock.sql': '040652e06b9ae491514898964ff9cd6db03ffe860db3e14dfd9ccf8ea85b221f', 'ops/postgres-finalize-native-people-directory.sql': 'f4f99cf873c2ab970789e44ccf9737f2dd38f6dc6b05f1849fbd4461bf6a2357', 'ops/postgres-native-company-policy-custody-state.sql': '072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479', 'ops/postgres-native-company-policy-owner.sql': '833300caeac6874f35ebb08017bfbb7891a5fe765a4c761b139466ae221573d5', 'ops/postgres-native-company-policy-v2-custody-state.sql': 'e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc', 'ops/postgres-native-company-policy-v2-owner.sql': 'f2050f21ef8151339289b2f013abdd543e90f8803fb6f6d4b9f004abf4409602', 'ops/postgres-native-people-directory-custody-state.sql': '20c96bc2a9264d5ed4b86cb948cbe0574450243509263469c45af955d5aff3dd', 'ops/postgres-native-people-directory-owner.sql': '3dd0524bc751eba2aa806ee0dc670dc3cecbf8838c20b83896773a0c5b6016a0', 'ops/postgres-native-people-directory-row-lock-custody-state.sql': '781cc446ce26525cbad6cd281c66239d1a7366eb9369d914068f39886cad3d6e', 'ops/postgres-verify-account-native.sql': 'c4e30ee9560b4d443acad1813a63667cbab9a32aadfbd893542f4b054bf6b308'}
_ORG_CLOSED_SPECIAL_MODES = {'--company-provenance-capture': {'ops/postgres-company-provenance-v1-owner.sql': 'e813293ace00c46358a0c1c62093e0911aae4da1b93c4dd1577dcb351c5398c2', 'ops/postgres-capture-company-provenance-v1-custody.sql': '0fc02c2bd70375acb0b6ddc86b66892af4069003c88dc58455347887cdf28ab2'}, '--company-provenance-custody': {'ops/postgres-company-provenance-v1-custody-state.sql': '12e91abb9fa850bef99c709d2a01963161c6b134511b4dbedf3b958d10240a4b', 'backend/app/src/company_provenance_v1_custody_state.sql': '12e91abb9fa850bef99c709d2a01963161c6b134511b4dbedf3b958d10240a4b'}, '--native-org-unit-closed-perimeter-capture': {'ops/postgres-native-org-unit-closed-perimeter-v1-owner.sql': 'aba221ad2cfa03390eca638ccd290901877fef615450b17cf14a96304e9f2609', 'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql': '6be2e3d095d59bbdb9e1b932dac8da48bde261601455cdcd166c6f2a649e6010'}}
_ORG_CLOSED_PHASE_PAIRS = (('plain', 'de87fafa527398d64a1930288ef1a0a56d017db6b56bc877f8b716c714afd90a', 'efc7f14dee39011c6ed5e68b97bd8374543b1307afe3d936b828ef5a52af51e7', 'fe0f65aebe362a969202e13d79c21d4e49f75834fd7b254a16a85a270e4e3c98', 'be4e86175dcd561150beb68db36de84fd3f224ca39d3128b1dcc958fa319a46d'), ('observer', '8011bd8141ec1a0497319773d73bc1df97a924f99e8aa84b8ef7c9cb4c821b37', 'a47330ee0efb705f72744a93263e65d400fbe525d70b829d2c7eedf5bfa75bdf', 'be18fc18f7bc6df0b5371ff01140d6a06c0a6a0438e7323d3eddaa7b070851e1', '7bf64f46c07608b2fba7a39be765a80e45db727b424caa55342103dc180dfcc9'))
_ORG_CLOSED_RELATIONS = ('account_context_candidates', 'account_security', 'account_security_events', 'account_terms_acceptances', 'account_terms_head', 'account_terms_release_receipts', 'accounts', 'audit_events', 'auth_bootstrap_credentials', 'auth_device_login_handoffs', 'auth_refresh_token_families', 'auth_refresh_tokens', 'auth_webauthn_ceremonies', 'auth_webauthn_ceremony_bindings', 'auth_webauthn_credentials', 'cedar_policy_catalog_entries', 'company_actors', 'company_authority_heads', 'company_enrollment_effect_bindings', 'company_enrollment_receipts', 'company_enrollment_request_events', 'company_enrollment_requests', 'deployment_operator_head', 'deployment_operator_receipts', 'employee_employment_profiles', 'employee_lifecycle_events', 'employee_person_bindings', 'employees', 'employment_revisions', 'employment_source_bindings', 'group_authority_heads', 'group_membership_revisions', 'group_memberships', 'group_role_grants', 'groups', 'leave_balance_import_receipts', 'native_company_action_refs', 'native_company_catalog_installs', 'native_company_object_refs', 'native_company_policy_inputs_v1', 'native_company_policy_receipts_v1', 'native_company_property_refs', 'native_people_inputs_v1', 'native_people_terminals_v1', 'ont_action_command_receipts', 'ont_action_types', 'ont_analytics', 'ont_builtin_catalog_allowlist', 'ont_builtin_catalog_installs', 'ont_link_types', 'ont_object_policies', 'ont_object_type_key_revisions', 'ont_object_types', 'ont_property_defs', 'org_unit_revisions', 'org_unit_source_bindings', 'org_units', 'organizations', 'person_revisions', 'persons', 'platform_force_removal_effect_bindings', 'platform_force_removal_receipts', 'platform_legacy_catalog_effect_bindings', 'platform_legacy_membership_effect_bindings', 'platform_legacy_topology_effect_bindings', 'platform_legacy_topology_receipts', 'platform_legacy_user_birth_witnesses', 'policy_assignment_revisions', 'policy_capability_clause_fields', 'policy_capability_clauses', 'policy_role_conditions', 'policy_role_permissions', 'policy_role_revisions', 'policy_roles', 'user_role_assignments', 'users')


def _org_closed_digest(raw):
    return hashlib.sha256(raw).hexdigest()


def _org_closed_inventory(root):
    result = {}
    for path in root.rglob('*'):
        name = str(path.relative_to(root))
        if path.is_symlink():
            identity = ('symlink', str(path.readlink()))
        elif path.is_file():
            identity = ('file', _org_closed_digest(path.read_bytes()))
        elif path.is_dir():
            identity = ('directory',)
        else:
            identity = ('special', path.lstat().st_mode)
        result[name] = identity
    return result


def _org_closed_sql_tokens(sql, *, with_spans=False):
    """Small test-only lexer: comments cannot satisfy a SQL contract assertion.

    Preserve quoted values; treat a complete dollar-quoted source as one token.
    This is a representation oracle, never a SQL executor or security parser.
    """
    tokens = []
    def append(token, start, end):
        tokens.append((token, start, end) if with_spans else token)
    position = 0
    while position < len(sql):
        rest = sql[position:]
        if rest[0].isspace():
            position += 1
            continue
        if rest.startswith('--'):
            end = sql.find('\n', position)
            position = len(sql) if end < 0 else end + 1
            continue
        if rest.startswith('/*'):
            position += 2
            depth = 1
            while depth:
                assert position < len(sql), 'unterminated SQL comment'
                if sql.startswith('/*', position):
                    depth += 1
                    position += 2
                elif sql.startswith('*/', position):
                    depth -= 1
                    position += 2
                else:
                    position += 1
            continue
        if rest[0] in ('\'', '"'):
            quote = rest[0]
            end = position + 1
            while True:
                assert end < len(sql), 'unterminated SQL quote'
                if sql[end] != quote:
                    end += 1
                elif end + 1 < len(sql) and sql[end + 1] == quote:
                    end += 2
                else:
                    end += 1
                    break
            append(sql[position:end], position, end)
            position = end
            continue
        dollar = _org_closed_re.match(r'\$(?:[A-Za-z_][A-Za-z_0-9]*)?\$', rest)
        if dollar:
            delimiter = dollar.group()
            end = sql.find(delimiter, position + len(delimiter))
            assert end >= 0, 'unterminated SQL dollar quote'
            end += len(delimiter)
            append(sql[position:end], position, end)
            position = end
            continue
        token = _org_closed_re.match(r'[A-Za-z_][A-Za-z_0-9]*|[0-9]+|::|<>|<=|>=|!=|.', rest)
        assert token is not None
        append(token.group().lower(), position, position + len(token.group()))
        position += len(token.group())
    return tokens


def _org_closed_cte(sql, name, *, repeated_identically=False):
    tokens = _org_closed_sql_tokens(sql)
    matches = []
    for position, token in enumerate(tokens):
        if token != name:
            continue
        opening = position + 1
        if opening < len(tokens) and tokens[opening] == '(':
            depth = 1
            opening += 1
            while depth:
                assert opening < len(tokens)
                depth += (tokens[opening] == '(') - (tokens[opening] == ')')
                opening += 1
        if tokens[opening:opening + 2] != ['as', '(']:
            continue
        start = opening + 2
        end = start
        depth = 1
        while depth:
            assert end < len(tokens), 'unterminated named SQL CTE'
            depth += (tokens[end] == '(') - (tokens[end] == ')')
            end += 1
        matches.append(tokens[start:end - 1])
    assert matches, 'an executable ' + name + ' CTE is required'
    if repeated_identically:
        assert all(match == matches[0] for match in matches), 'repeated ' + name + ' CTE drift'
    else:
        assert len(matches) == 1, 'exactly one executable ' + name + ' CTE is required'
    return matches[0]


def _org_closed_cte_body_span(sql, name):
    """Locate one executable CTE body, never a comment or quoted capture.

    Used only to calibrate the representation oracle on actual exported bytes.
    The original frozen capture copies outside this exact body stay untouched.
    """
    entries = _org_closed_sql_tokens(sql, with_spans=True)
    tokens = [entry[0] for entry in entries]
    matches = []
    for position, token in enumerate(tokens):
        if token != name:
            continue
        opening = position + 1
        if opening < len(tokens) and tokens[opening] == '(':
            depth = 1
            opening += 1
            while depth:
                assert opening < len(tokens), 'unterminated CTE column list'
                depth += (tokens[opening] == '(') - (tokens[opening] == ')')
                opening += 1
        if tokens[opening:opening + 2] != ['as', '(']:
            continue
        end = opening + 2
        depth = 1
        while depth:
            assert end < len(tokens), 'unterminated named SQL CTE'
            depth += (tokens[end] == '(') - (tokens[end] == ')')
            end += 1
        matches.append((entries[opening + 1][2], entries[end - 1][1]))
    assert len(matches) == 1, 'one executable mutation target CTE is required'
    return matches[0]


def _org_closed_replace_cte_body(sql, name, replacement):
    start, end = _org_closed_cte_body_span(sql, name)
    return sql[:start] + replacement + sql[end:]


def _org_closed_values(rows):
    parts = []
    for row in rows:
        parts.append('(' + ','.join(str(value) if isinstance(value, int)
                                  else "'" + value.replace("'", "''") + "'"
                                  for value in row) + ')')
    return _org_closed_sql_tokens('VALUES ' + ','.join(parts))


def _org_closed_code(sql):
    return ' '.join('quoted' if token.startswith(('\'', '"', '$')) else token
                    for token in _org_closed_sql_tokens(sql))


def _org_closed_final_select(sql):
    tokens = _org_closed_sql_tokens(sql)
    depth = 0
    starts = []
    for index, token in enumerate(tokens):
        if token == 'select' and depth == 0:
            starts.append(index)
        depth += (token == '(') - (token == ')')
        assert depth >= 0, 'unbalanced SQL parentheses'
    assert depth == 0 and len(starts) == 1, 'one top-level read-only verdict SELECT'
    return tokens[starts[0]:]


def _org_closed_ledger_check_sql(expected):
    return """IF (WITH expected_migrations(version,checksum) AS (
        """ + ' '.join(_org_closed_values(expected)) + """
        ) SELECT count(*)=231 AND bool_and(e.version IS NOT NULL AND m.version IS NOT NULL
            AND m.success IS TRUE AND (encode(m.checksum,'hex')=e.checksum) IS TRUE) IS TRUE
          FROM expected_migrations e FULL JOIN public._sqlx_migrations m ON m.version=e.version)
          IS NOT TRUE THEN
            RAISE EXCEPTION 'native_org_unit.migration_ledger_mismatch';
        END IF;"""


class NativeOrgUnitClosedPerimeterCustodyGeneration(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='native-org-closed-custody-generator-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        # Copy real existing inputs, not fake SQL profiles. The actual copied
        # CLI runs without imports/site hooks, network or database access.
        inputs = {Path(_ORG_CLOSED_SCRIPT), *(Path(name) for name in _ORG_CLOSED_DEFAULTS),
                  *(Path(name) for pins in _ORG_CLOSED_SPECIAL_MODES.values() for name in pins)}
        for directory in ('ops', 'backend/crates/platform/db/migrations'):
            inputs.update(path.relative_to(ROOT)
                          for path in (ROOT / directory).rglob('*.sql')
                          if str(path.relative_to(ROOT)) not in _ORG_CLOSED_OUTPUTS)
        for name in inputs:
            source = ROOT / name
            self.assertFalse(source.is_symlink(), str(name))
            self.assertTrue(source.is_file(), str(name))
            target = self.root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            _org_closed_shutil.copyfile(source, target)
        (self.root / 'backend/app/src').mkdir(parents=True, exist_ok=True)
        self.assertEqual(_org_closed_digest((self.root / _ORG_CLOSED_SOURCE).read_bytes()),
                         _ORG_CLOSED_SOURCE_SHA256)
        for name, expected in _ORG_CLOSED_DEFAULTS.items():
            self.assertEqual(_org_closed_digest((self.root / name).read_bytes()), expected, name)
        for pins in _ORG_CLOSED_SPECIAL_MODES.values():
            for name, expected in pins.items():
                self.assertEqual(_org_closed_digest((self.root / name).read_bytes()), expected, name)

    def cli(self, *arguments):
        return _org_closed_subprocess.run(
            [_org_closed_sys.executable, '-I', '-B', '-S',
             str(self.root / _ORG_CLOSED_SCRIPT), *arguments],
            cwd=self.root, capture_output=True, text=True, timeout=30)

    def export(self):
        before = _org_closed_inventory(self.root)
        result = self.cli(_ORG_CLOSED_MODE)
        self.assertEqual(result.returncode, 0,
                         'ORG_CLOSED_CUSTODY_MODE_MISSING_OR_REFUSED: ' + result.stderr)
        after = _org_closed_inventory(self.root)
        self.assertEqual(set(after) - set(before), set(_ORG_CLOSED_OUTPUTS),
                         'custody mode must export exactly three real artifacts')
        self.assertEqual({name: after[name] for name in before}, before,
                         'custody export changed an existing input or historical artifact')
        return {name: (self.root / name).read_bytes() for name in _ORG_CLOSED_OUTPUTS}

    def assert_state(self, raw):
        state = raw.decode('utf-8')
        self.assertTrue(state.endswith(';\n'))
        # Frozen serializers retain every byte and their original73/wider76
        # meanings. No new hash may be generated from the mutable checkout.
        for cte, name in (
                ('original73', 'ops/postgres-capture-company-provenance-v1-custody.sql'),
                ('wider76', 'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql')):
            capture = (self.root / name).read_text().removesuffix(';\n')
            self.assertEqual(state.count(capture), 1, 'frozen complete capture changed: ' + name)
            self.assertEqual(_org_closed_cte(state, cte), _org_closed_sql_tokens(capture),
                             'actual executable capture CTE differs: ' + cte)
        pairs = [(variant, predecessor73, predecessor76, closed73, closed76)
                 for variant, predecessor73, predecessor76, closed73, closed76
                 in _ORG_CLOSED_PHASE_PAIRS]
        self.assertEqual(_org_closed_cte(state, 'phase_pairs'), _org_closed_values(pairs))
        self.assertEqual(_org_closed_cte(state, 'matching_phase'), _org_closed_sql_tokens("""
            SELECT p.variant,'closed'::text AS phase
            FROM phase_pairs p CROSS JOIN original73 o CROSS JOIN wider76 w
            WHERE o.snapshot_sha256=p.closed73 AND w.snapshot_sha256=p.closed76
            UNION ALL
            SELECT p.variant,'predecessor'::text AS phase
            FROM phase_pairs p CROSS JOIN original73 o CROSS JOIN wider76 w
            WHERE o.snapshot_sha256=p.predecessor73 AND w.snapshot_sha256=p.predecessor76
        """), 'each phase must match both measurements on the same variant row')
        self.assertEqual(_org_closed_cte(state, 'startup_role'), _org_closed_sql_tokens("""
            SELECT oid,rolname FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup'
        """))
        self.assertEqual(_org_closed_cte(state, 'org_relations'), _org_closed_sql_tokens("""
            SELECT required.name,c.oid,c.relkind,c.relispartition,n.nspname,r.rolname AS owner_name
            FROM required_org_relations required
            LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
            LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
            LEFT JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
        """))
        self.assertEqual(_org_closed_cte(state, 'org_columns'), _org_closed_sql_tokens("""
            SELECT r.name,r.oid,a.attnum,a.attname,a.atttypid,a.atttypmod,a.attnotnull
            FROM org_relations r JOIN pg_catalog.pg_attribute a ON a.attrelid=r.oid
            WHERE a.attnum>0 AND NOT a.attisdropped
        """))
        expected_columns = [
            (relation, number, name)
            for relation, names in (
                ('org_unit_revisions', ('org_id','id','org_unit_id','version','command_id',
                    'actor_id','payload_digest','attributes','receipt','created_at')),
                ('org_unit_source_bindings', ('org_id','source_kind','source_id','org_unit_id',
                    'actor_id','payload_digest','created_at')),
                ('org_units', ('org_id','id','created_at')))
            for number, name in enumerate(names, 1)]
        self.assertEqual(_org_closed_cte(state, 'required_org_columns'),
                         _org_closed_values(expected_columns))
        self.assertEqual(_org_closed_cte(state, 'table_checks'), _org_closed_sql_tokens("""
            SELECT r.name,r.oid,p.privilege,
             pg_catalog.has_table_privilege(s.oid,r.oid,p.privilege) AS allowed
            FROM startup_role s CROSS JOIN org_relations r CROSS JOIN table_privileges p
        """))
        self.assertEqual(_org_closed_cte(state, 'column_checks'), _org_closed_sql_tokens("""
            SELECT c.name,c.oid,c.attnum,c.attname,p.privilege,
             pg_catalog.has_column_privilege(s.oid,c.oid,c.attnum,p.privilege) AS allowed
            FROM startup_role s CROSS JOIN org_columns c CROSS JOIN column_privileges p
        """))
        self.assertEqual(_org_closed_cte(state, 'table_privileges'),
                         _org_closed_values([(v,) for v in
                             ('SELECT', 'INSERT', 'UPDATE', 'DELETE', 'TRUNCATE',
                              'REFERENCES', 'TRIGGER', 'MAINTAIN')]))
        self.assertEqual(_org_closed_cte(state, 'column_privileges'),
                         _org_closed_values([(v,) for v in
                             ('SELECT', 'INSERT', 'UPDATE', 'REFERENCES')]))
        self.assertEqual(_org_closed_cte(state, 'added3_valid'), _org_closed_sql_tokens("""
            SELECT (SELECT count(*) FROM startup_role)=1
             AND (SELECT bool_and(oid IS NOT NULL AND oid>0 AND rolname='console_auth_startup')
                  FROM startup_role) IS TRUE
             AND (SELECT count(*) FROM org_relations)=3
             AND (SELECT count(DISTINCT oid) FROM org_relations)=3
             AND (SELECT bool_and(oid IS NOT NULL AND oid>0 AND relkind='r'
                  AND NOT relispartition AND nspname='public' AND owner_name='console_app')
                  FROM org_relations) IS TRUE
             AND (SELECT count(*) FROM org_columns)=20
             AND (SELECT count(DISTINCT (oid,attnum)) FROM org_columns)=20
             AND (SELECT bool_and(oid IS NOT NULL AND oid>0 AND attnum>0 AND attname IS NOT NULL
                  AND atttypid>0 AND attnotnull IS NOT NULL) FROM org_columns) IS TRUE
             AND (SELECT count(*) FROM org_columns c JOIN required_org_columns e
                  ON e.name=c.name AND e.attnum=c.attnum AND e.attname=c.attname)=20
             AND (SELECT count(*) FROM table_checks)=24
             AND (SELECT count(DISTINCT (oid,privilege)) FROM table_checks)=24
             AND (SELECT count(*) FROM column_checks)=80
             AND (SELECT count(DISTINCT (oid,attnum,privilege)) FROM column_checks)=80
             AND (SELECT bool_and(allowed IS FALSE) FROM table_checks) IS TRUE
             AND (SELECT bool_and(allowed IS FALSE) FROM column_checks) IS TRUE AS valid
        """), 'strict conjunction cannot be broadened, unbound or unused')
        self.assertEqual(_org_closed_cte(state, 'required_org_relations'),
                         _org_closed_values([(v,) for v in
                             ('org_unit_revisions', 'org_unit_source_bindings', 'org_units')]))
        sql = ' '.join(_org_closed_sql_tokens(state))
        for expression in (
                '( select native_directory_startup_rights_valid from original73 ) is true',
                '( select valid from added3_valid ) is true',
                '( select valid from guarded_relations_valid ) is true',
                '( select valid from reserved_relations ) is true',
                '( select valid from reserved_schemas ) is true',
                '( select count ( * ) from matching_phase ) = 1',
                "'native_org_unit.closed_perimeter_compatible'",
                "'native_org_unit.closed_perimeter_required'",
                "'native_org_unit.profile_mismatch'"):
            self.assertIn(expression, sql)
        self.assertEqual(_org_closed_final_select(state), _org_closed_sql_tokens("""
            SELECT CASE
             WHEN (SELECT native_directory_startup_rights_valid FROM original73) IS TRUE
              AND (SELECT valid FROM added3_valid) IS TRUE
              AND (SELECT valid FROM guarded_relations_valid) IS TRUE
              AND (SELECT valid FROM reserved_relations) IS TRUE
              AND (SELECT valid FROM reserved_schemas) IS TRUE
              AND (SELECT valid FROM native_org_routine_namespace) IS TRUE
              AND (SELECT count(*) FROM matching_phase)=1
             THEN CASE (SELECT phase FROM matching_phase)
              WHEN 'closed' THEN 'native_org_unit.closed_perimeter_compatible'
              WHEN 'predecessor' THEN 'native_org_unit.closed_perimeter_required'
              ELSE 'native_org_unit.profile_mismatch' END
             ELSE 'native_org_unit.profile_mismatch' END AS state;
        """), 'a present but unused CTE or diagnostic flag cannot admit custody')
        self.assertNotRegex(_org_closed_code(state), r'\b(insert|update|delete|truncate|create|alter|drop)\b',
                            'read-only classifier must not modify database state')

    def assert_namespace(self, raw):
        state = raw.decode('utf-8')
        # Exact narrow additive queries deliberately have no schema/owner/kind,
        # temporary/system, partition, inheritance or dependency exclusions.
        self.assertEqual(_org_closed_cte(state, 'reserved_relations'), _org_closed_sql_tokens("""
            SELECT count(*)=0 AS valid FROM pg_catalog.pg_class c
            JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
            WHERE c.relname='native_org_unit' OR starts_with(c.relname,'native_org_unit_')
        """))
        self.assertEqual(_org_closed_cte(state, 'reserved_schemas'), _org_closed_sql_tokens("""
            SELECT count(*)=0 AS valid FROM pg_catalog.pg_namespace n
            WHERE n.nspname='native_org_unit' OR starts_with(n.nspname,'native_org_unit_')
        """))
        self.assertEqual(_org_closed_cte(state, 'required_guarded_relations'),
                         _org_closed_values([(v,) for v in
                             ('ont_action_command_receipts', 'org_unit_revisions',
                              'org_unit_source_bindings', 'org_units')]))
        self.assertEqual(_org_closed_cte(state, 'guarded_relations_valid'),
                         _org_closed_sql_tokens("""
            SELECT count(*)=4 AND count(DISTINCT c.oid)=4
             AND count(DISTINCT n.oid)=1 AND count(DISTINCT r.oid)=1
             AND bool_and(c.oid IS NOT NULL AND c.oid>0 AND c.relkind='r'
                  AND NOT c.relispartition AND n.nspname='public' AND r.rolname='console_app')
                  IS TRUE AS valid
            FROM required_guarded_relations required
            LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
            LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
            LEFT JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
        """), 'guarded identities are actual, exact and conjunctively required')
        namespace = ' '.join(_org_closed_cte(state, 'native_org_routine_namespace'))
        self.assertIn("starts_with ( p . proname , 'native_org_unit_' )", namespace)
        self.assertNotIn('like', _org_closed_cte(state, 'reserved_relations'))
        self.assertNotIn('like', _org_closed_cte(state, 'reserved_schemas'))

    def assert_finalizer(self, raw):
        finalizer = raw.decode('utf-8')
        source = (self.root / _ORG_CLOSED_SOURCE).read_text()
        self.assertEqual(finalizer.count(source), 1, 'guard source must be embedded once byte-for-byte')
        envelope = _org_closed_sql_tokens(finalizer)
        self.assertEqual(len(envelope), 3, 'finalizer contains exactly one real statement')
        self.assertEqual(envelope[0], 'do')
        self.assertEqual(envelope[2], ';')
        match = _org_closed_re.match(r'\$(?:[A-Za-z_][A-Za-z_0-9]*)?\$', envelope[1])
        self.assertIsNotNone(match, 'finalizer is one atomic dollar-quoted DO statement')
        delimiter = match.group()
        self.assertTrue(envelope[1].endswith(delimiter))
        body = envelope[1][len(delimiter):-len(delimiter)]
        self.assertEqual(_org_closed_cte(body, 'required_relations', repeated_identically=True),
                         _org_closed_values([(v,) for v in _ORG_CLOSED_RELATIONS]))
        self.assertEqual(len(_ORG_CLOSED_RELATIONS), 76)
        self.assertEqual(list(_ORG_CLOSED_RELATIONS), sorted(set(_ORG_CLOSED_RELATIONS)))
        ledger = (self.root / 'ops/account-custody-migrations.sha384').read_bytes()
        self.assertEqual(_org_closed_digest(ledger),
                         '42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357')
        rows = [line.split('\t') for line in ledger.decode().splitlines()]
        expected = [(int(version), checksum) for version, checksum in rows]
        self.assertEqual([version for version, _ in expected], list(range(1, 232)))
        self.assertEqual(_org_closed_cte(body, 'expected_migrations', repeated_identically=True),
                         _org_closed_values(expected))
        words = _org_closed_sql_tokens(body)
        # The actual initial, closed-replay and installed-success branches
        # each revalidate the complete ledger. Comments and dollar-quoted SQL
        # are inert lexer tokens and cannot stand in for executable checks.
        ledger_check = _org_closed_sql_tokens(_org_closed_ledger_check_sql(expected))
        ledger_prefix = _org_closed_sql_tokens('IF (WITH expected_migrations(version,checksum) AS (')
        ledger_positions = [i for i in range(len(words))
                            if words[i:i + len(ledger_prefix)] == ledger_prefix]
        self.assertEqual(len(ledger_positions), 3,
                         'ORG_CLOSED_LEDGER_RECHECK_COUNT: initial, replay and success each require a check')
        for position in ledger_positions:
            self.assertEqual(words[position:position + len(ledger_check)], ledger_check,
                             'ORG_CLOSED_LEDGER_STRICT_ROWS: every checksum comparison must be IS TRUE')
        replay_prefix = _org_closed_sql_tokens(
            "IF phase='native_org_unit.closed_perimeter_compatible' THEN")
        replay_positions = [i for i in range(len(words))
                            if words[i:i + len(replay_prefix)] == replay_prefix]
        self.assertEqual(len(replay_positions), 1, 'one actual compatible replay branch required')
        replay_check = replay_positions[0] + len(replay_prefix)
        self.assertEqual(words[replay_check:replay_check + len(ledger_check)], ledger_check,
                         'ORG_CLOSED_REPLAY_LEDGER_RECHECK: actual replay must revalidate before RETURN')
        replay_return = replay_check + len(ledger_check)
        self.assertEqual(words[replay_return:replay_return + 5],
                         _org_closed_sql_tokens('RETURN; END IF;'),
                         'replay ledger revalidation must precede its actual RETURN')
        embedded = []
        for position, token in enumerate(words):
            opening = _org_closed_re.match(r'\$(?:[A-Za-z_][A-Za-z_0-9]*)?\$', token)
            if opening and token[len(opening.group()):-len(opening.group())] == source:
                self.assertGreater(position, 0)
                self.assertEqual(words[position - 1], 'execute', 'original source is actually executed')
                embedded.append(token)
        self.assertEqual(len(embedded), 1, 'exact guard input must be the sole executed source copy')
        sql = ' '.join(words)
        for name in ('lock_timeout', 'statement_timeout',
                     'idle_in_transaction_session_timeout', 'transaction_timeout'):
            self.assertIn("current_setting ( '" + name + "'", sql)
            self.assertNotIn("set_config ( '" + name + "'", sql,
                             'caller must set bounds before the DO begins')
        for bound in ('1000', '60000', '30000', '120000'):
            self.assertIn(bound, words, 'reviewed millisecond upper bound omitted')
        for expression in (
                "current_setting ( 'transaction_isolation' )", "'read committed'",
                'session_user', 'current_user', 'rolsuper',
                'pg_catalog . pg_locks', 'pg_backend_pid ( )',
                "'ShareLock'", "'AccessExclusiveLock'", 'granted',
                "'public._sqlx_migrations'", "order by c . relname collate \"C\"",
                "'LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE'",
                'set constraints all immediate'):
            self.assertIn(expression, sql)
        source_index = body.index(source)
        before = body[:source_index]
        after = body[source_index + len(source):]
        self.assertIn('RETURN', before, 'closed replay must return before source execution')
        self.assertIn('_sqlx_migrations', before, 'ledger must be checked before source execution')
        self.assertIn('_sqlx_migrations', after, 'ledger must be rechecked before return')
        self.assertIn('SET CONSTRAINTS ALL IMMEDIATE', after)
        self.assertIn('phase_pairs', before)
        self.assertIn('phase_pairs', after)
        self.assertNotRegex(_org_closed_code(body), r'\b(commit|rollback|insert|update|delete|truncate)\b',
                            'installer cannot commit itself or rewrite business/ledger rows')
        for name in ('ops/postgres-finalize-native-people-directory.sql',
                     'ops/postgres-finalize-native-people-directory-row-lock.sql',
                     'ops/postgres-finalize-company-enrollment.sql'):
            self.assertNotIn((self.root / name).read_text(), finalizer,
                             'closed finalizer must not silently install older phases')

    def test_cli_exports_exact_three_artifacts_preserves_history_and_is_idempotent(self):
        before = _org_closed_inventory(self.root)
        self.assertEqual(self.cli('--check').returncode, 0)
        self.assertEqual(_org_closed_inventory(self.root), before)
        outputs = self.export()
        self.assertEqual(outputs[_ORG_CLOSED_OUTPUTS[0]], outputs[_ORG_CLOSED_OUTPUTS[1]])
        after = _org_closed_inventory(self.root)
        for arguments in ((_ORG_CLOSED_MODE,), (_ORG_CLOSED_MODE, '--check'), (), ('--check',)):
            with self.subTest(arguments=arguments):
                result = self.cli(*arguments)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(_org_closed_inventory(self.root), after)
        self.assertEqual(len(_ORG_CLOSED_DEFAULTS), 33)
        self.assertEqual(_org_closed_digest(_org_closed_json.dumps(
            _ORG_CLOSED_DEFAULTS, sort_keys=True, separators=(',', ':')).encode()),
            '93a82a4c3e295d3218bb80353868b85c591a3aef522560173d99d4fb133e09a6')
        for mode, pins in _ORG_CLOSED_SPECIAL_MODES.items():
            for arguments in ((mode,), (mode, '--check')):
                result = self.cli(*arguments)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(_org_closed_inventory(self.root), after,
                                 'old explicit mode changed history or adopted closed custody')
            for name, expected in pins.items():
                self.assertEqual(_org_closed_digest((self.root / name).read_bytes()), expected, name)

    def test_state_requires_same_variant_pairs_strict_effective_denials_and_frozen_captures(self):
        outputs = self.export()
        self.assert_state(outputs[_ORG_CLOSED_OUTPUTS[0]])

    def test_state_closes_complete_relation_index_schema_and_routine_namespaces(self):
        outputs = self.export()
        self.assert_namespace(outputs[_ORG_CLOSED_OUTPUTS[0]])

    def test_finalizer_requires_bounded_direct_entry_exact_ledger_ordered76_and_source(self):
        outputs = self.export()
        self.assert_finalizer(outputs[_ORG_CLOSED_OUTPUTS[2]])

    def test_check_refuses_each_missing_or_changed_output_without_repair(self):
        originals = self.export()
        for name in _ORG_CLOSED_OUTPUTS:
            target = self.root / name
            for fault in ('missing', 'changed'):
                with self.subTest(output=name, fault=fault):
                    target.unlink()
                    if fault == 'changed':
                        target.write_bytes(originals[name] + b'\n')
                    before = _org_closed_inventory(self.root)
                    result = self.cli(_ORG_CLOSED_MODE, '--check')
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('differs', result.stderr)
                    self.assertEqual(_org_closed_inventory(self.root), before, 'check repaired an output')
                    target.write_bytes(originals[name])

    def test_source_missing_changed_directory_symlink_and_parent_link_refuse_before_output(self):
        self.export()  # Prove the real mode is executable before negative controls.
        for name in _ORG_CLOSED_OUTPUTS:
            (self.root / name).unlink()
        for source_name in (_ORG_CLOSED_SOURCE, 'ops/company-enrollment/provenance-v1.sql',
                            'ops/company-enrollment/provenance-acl-v1.sql'):
            target = self.root / source_name
            original = target.read_bytes()
            for fault in ('missing', 'changed', 'directory', 'symlink', 'dangling_symlink'):
                with self.subTest(source=source_name, fault=fault):
                    target.unlink()
                    linked = self.root / 'unrelated-exact-source.sql'
                    if fault == 'changed':
                        target.write_bytes(original + b'\n')
                    elif fault == 'directory':
                        target.mkdir()
                    elif fault in ('symlink', 'dangling_symlink'):
                        if fault == 'symlink':
                            linked.write_bytes(original)
                        target.symlink_to(linked)
                    before = _org_closed_inventory(self.root)
                    for arguments in ((_ORG_CLOSED_MODE,), (_ORG_CLOSED_MODE, '--check')):
                        result = self.cli(*arguments)
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn('reviewed bytes' if fault == 'changed' else 'regular file', result.stderr)
                        self.assertEqual(_org_closed_inventory(self.root), before,
                                         'input refusal wrote partial custody artifacts')
                    if fault == 'directory':
                        target.rmdir()
                    else:
                        target.unlink(missing_ok=True)
                    target.write_bytes(original)
                    linked.unlink(missing_ok=True)
        parent = self.root / 'ops/native-org-unit'
        retained = self.root / 'retained-native-org-unit'
        parent.rename(retained)
        parent.symlink_to(retained, target_is_directory=True)
        before = _org_closed_inventory(self.root)
        for arguments in ((_ORG_CLOSED_MODE,), (_ORG_CLOSED_MODE, '--check')):
            result = self.cli(*arguments)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('regular file', result.stderr)
            self.assertEqual(_org_closed_inventory(self.root), before)

    def test_all_destinations_are_validated_before_any_artifact_is_written(self):
        originals = self.export()
        for name in _ORG_CLOSED_OUTPUTS:
            (self.root / name).unlink()
        for name in _ORG_CLOSED_OUTPUTS:
            for fault in ('directory', 'symlink', 'dangling_symlink'):
                with self.subTest(output=name, fault=fault):
                    target = self.root / name
                    external = self.root / 'unrelated-exact-output.sql'
                    if fault == 'directory':
                        target.mkdir()
                    else:
                        if fault == 'symlink':
                            external.write_bytes(originals[name])
                        target.symlink_to(external)
                    before = _org_closed_inventory(self.root)
                    for arguments in ((_ORG_CLOSED_MODE,), (_ORG_CLOSED_MODE, '--check')):
                        result = self.cli(*arguments)
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn('regular file', result.stderr)
                        self.assertEqual(_org_closed_inventory(self.root), before,
                                         'invalid later destination allowed partial publication')
                    if fault == 'directory':
                        target.rmdir()
                    else:
                        target.unlink()
                    external.unlink(missing_ok=True)
        parent = self.root / 'backend/app/src'
        retained = self.root / 'retained-app-src'
        parent.rename(retained)
        parent.symlink_to(retained, target_is_directory=True)
        before = _org_closed_inventory(self.root)
        for arguments in ((_ORG_CLOSED_MODE,), (_ORG_CLOSED_MODE, '--check')):
            result = self.cli(*arguments)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('regular file', result.stderr)
            self.assertEqual(_org_closed_inventory(self.root), before)

    def test_ambiguous_or_combined_modes_refuse_without_writes(self):
        self.export()
        before = _org_closed_inventory(self.root)
        for arguments in ((_ORG_CLOSED_MODE, _ORG_CLOSED_MODE),
                          (_ORG_CLOSED_MODE, '--check', '--check'),
                          (_ORG_CLOSED_MODE, '--unknown'),
                          (_ORG_CLOSED_MODE, '--company-provenance-custody'),
                          ('--company-provenance-capture', _ORG_CLOSED_MODE),
                          ('--native-org-unit-closed-perimeter-capture', _ORG_CLOSED_MODE)):
            with self.subTest(arguments=arguments):
                result = self.cli(*arguments)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('usage:', result.stderr)
                self.assertEqual(_org_closed_inventory(self.root), before)

    def test_representation_oracles_reject_corruption_of_actual_exported_sql(self):
        outputs = self.export()
        state = outputs[_ORG_CLOSED_OUTPUTS[0]]
        finalizer = outputs[_ORG_CLOSED_OUTPUTS[2]]
        self.assert_state(state)
        self.assert_namespace(state)
        self.assert_finalizer(finalizer)
        # Mutations are confined to test-owned in-memory exported bytes. They
        # calibrate these prerequisite oracles; no runtime/chaos claim follows.
        state_faults = [
            (_org_closed_re.escape(_ORG_CLOSED_PHASE_PAIRS[0][2]),
             _ORG_CLOSED_PHASE_PAIRS[1][2], self.assert_state),
            (r"starts_with\s*\(\s*c\.relname\s*,\s*'native_org_unit_'\s*\)",
             "c.relname LIKE 'native_org_unit_%'", self.assert_namespace),
            (r"FROM\s+pg_catalog\.pg_namespace\s+n\s+WHERE\s+n\.nspname\s*=\s*'native_org_unit'",
             "FROM pg_catalog.pg_namespace n WHERE n.nspname='public' AND n.nspname='native_org_unit'",
             self.assert_namespace),
        ]
        for pattern, changed, oracle in state_faults:
            with self.subTest(state_fault=pattern):
                corrupt, count = _org_closed_re.subn(pattern, lambda _: changed,
                    state.decode('utf-8'), count=1, flags=_org_closed_re.IGNORECASE)
                self.assertEqual(count, 1, 'absent mutation target cannot pass calibration')
                with self.assertRaises(AssertionError):
                    oracle(corrupt.encode('utf-8'))
        # Target new strict predicates alone; the frozen original73 capture
        # already contains allowed IS FALSE and must remain byte-identical.
        text = state.decode('utf-8')
        captures = [
            (name, (self.root / name).read_text().removesuffix(';\n'))
            for name in ('ops/postgres-capture-company-provenance-v1-custody.sql',
                         'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql')]
        scoped_faults = (
            ('added3_valid', r'\ballowed\s+IS\s+FALSE\b',
             'allowed IS NOT TRUE', self.assert_state),
            ('added3_valid', r'\bFROM\s+table_checks\s*\)\s*=\s*24\b',
             'FROM table_checks)=23', self.assert_state),
            ('added3_valid', r'\bAS\s+valid\b', 'OR TRUE AS valid', self.assert_state),
            ('guarded_relations_valid', r'\bIS\s+TRUE\s+AS\s+valid\b',
             'IS TRUE OR TRUE AS valid', self.assert_namespace),
            ('guarded_relations_valid', r'\bcount\s*\(\s*DISTINCT\s+c\.oid\s*\)\s*=\s*4\b',
             'count(DISTINCT c.oid)=3', self.assert_namespace),
        )
        for name, pattern, changed, oracle in scoped_faults:
            with self.subTest(cte=name, fault=pattern):
                start, end = _org_closed_cte_body_span(text, name)
                body, count = _org_closed_re.subn(pattern, lambda _: changed, text[start:end],
                    count=1, flags=_org_closed_re.IGNORECASE)
                self.assertEqual(count, 1, 'one targeted new-predicate mutation must execute')
                corrupt = _org_closed_replace_cte_body(text, name, body)
                self.assertNotEqual(corrupt, text)
                for capture_name, capture in captures:
                    self.assertEqual(corrupt.count(capture), 1,
                                     'new-predicate calibration altered frozen capture: ' + capture_name)
                with self.assertRaises(AssertionError):
                    oracle(corrupt.encode('utf-8'))
        # Keeping a full byte-identical capture as unused quoted text or a
        # comment cannot satisfy the actual executable original73/wider76 CTE.
        for cte, (_, capture) in zip(('original73', 'wider76'), captures):
            for inert in ('comment', 'quoted'):
                with self.subTest(stub_capture=cte, inert=inert):
                    ignored = ('/*' + capture + '*/') if inert == 'comment' else (
                        ', $org_unused_capture$' + capture + '$org_unused_capture$ AS unused_capture')
                    if inert == 'quoted':
                        self.assertNotIn('$org_unused_capture$', capture)
                    stub = ("SELECT '" + _ORG_CLOSED_PHASE_PAIRS[0][3 if cte == 'original73' else 4]
                            + "'::text AS snapshot_sha256, TRUE AS native_directory_startup_rights_valid "
                            + ignored)
                    corrupt = _org_closed_replace_cte_body(text, cte, stub)
                    self.assertEqual(corrupt.count(capture), 1,
                                     'stub calibration must preserve full frozen capture bytes once')
                    with self.assertRaises(AssertionError):
                        self.assert_state(corrupt.encode('utf-8'))
        source = (self.root / _ORG_CLOSED_SOURCE).read_bytes()
        for original, changed in ((source, source + b'\n'),
                                  (b'SET CONSTRAINTS ALL IMMEDIATE', b'SET CONSTRAINTS ALL DEFERRED')):
            with self.subTest(finalizer_fault=original[:40]):
                self.assertIn(original, finalizer)
                with self.assertRaises(AssertionError):
                    self.assert_finalizer(finalizer.replace(original, changed, 1))

        # Calibrate against real exported bytes while keeping both captures,
        # original guard source and the other complete ledger checks intact.
        finalizer_text = finalizer.decode('utf-8')
        envelope = _org_closed_sql_tokens(finalizer_text, with_spans=True)
        delimiter = _org_closed_re.match(r'\$(?:[A-Za-z_][A-Za-z_0-9]*)?\$', envelope[1][0]).group()
        body_offset = envelope[1][1] + len(delimiter)
        finalizer_body = envelope[1][0][len(delimiter):-len(delimiter)]
        entries = _org_closed_sql_tokens(finalizer_body, with_spans=True)
        words = [entry[0] for entry in entries]
        ledger_rows = [line.split('\t') for line in
                       (self.root / 'ops/account-custody-migrations.sha384').read_text().splitlines()]
        expected = [(int(version), checksum) for version, checksum in ledger_rows]
        ledger_check = _org_closed_sql_tokens(_org_closed_ledger_check_sql(expected))
        ledger_prefix = _org_closed_sql_tokens('IF (WITH expected_migrations(version,checksum) AS (')
        positions = [i for i in range(len(words))
                     if words[i:i + len(ledger_prefix)] == ledger_prefix]
        self.assertEqual(len(positions), 3, 'three actual checks required before calibration')
        spans = [(body_offset + entries[i][1],
                  body_offset + entries[i + len(ledger_check) - 1][2]) for i in positions]
        checks = [finalizer_text[start:end] for start, end in spans]
        for check in checks:
            self.assertEqual(_org_closed_sql_tokens(check), ledger_check)
        replay_prefix = _org_closed_sql_tokens(
            "IF phase='native_org_unit.closed_perimeter_compatible' THEN")
        replay_positions = [i for i in range(len(words))
                            if words[i:i + len(replay_prefix)] == replay_prefix]
        self.assertEqual(len(replay_positions), 1)
        replay_index = replay_positions[0] + len(replay_prefix)
        self.assertEqual(positions[1], replay_index, 'middle actual check belongs to closed replay')
        start, end = spans[1]
        source_text = source.decode('utf-8')
        def retained_inputs(corrupt):
            self.assertEqual(corrupt.count(source_text), finalizer_text.count(source_text))
            for _, capture in captures:
                self.assertEqual(corrupt.count(capture), finalizer_text.count(capture))
        for inert in ('removed', 'comment', 'quoted'):
            with self.subTest(replay_ledger_fault=inert):
                replacement = '' if inert == 'removed' else (
                    '/*' + checks[1] + '*/' if inert == 'comment' else
                    'PERFORM $org_unused_ledger$' + checks[1] + '$org_unused_ledger$;')
                self.assertNotIn('$org_unused_ledger$', checks[1])
                corrupt = finalizer_text[:start] + replacement + finalizer_text[end:]
                self.assertEqual(corrupt[:start], finalizer_text[:start])
                self.assertEqual(corrupt[start + len(replacement):], finalizer_text[end:])
                retained_inputs(corrupt)
                with self.assertRaises(AssertionError):
                    self.assert_finalizer(corrupt.encode('utf-8'))
        for index, ((start, end), check) in enumerate(zip(spans, checks)):
            with self.subTest(nullable_checksum_branch=index):
                changed, count = _org_closed_re.subn(
                    r"\(\s*encode\s*\(\s*m\.checksum\s*,\s*'hex'\s*\)\s*=\s*e\.checksum\s*\)\s+IS\s+TRUE",
                    "encode(m.checksum,'hex')=e.checksum", check, count=1,
                    flags=_org_closed_re.IGNORECASE)
                self.assertEqual(count, 1, 'one actual strict checksum must be removed')
                corrupt = finalizer_text[:start] + changed + finalizer_text[end:]
                self.assertEqual(corrupt[:start], finalizer_text[:start])
                self.assertEqual(corrupt[start + len(changed):], finalizer_text[end:])
                retained_inputs(corrupt)
                with self.assertRaises(AssertionError):
                    self.assert_finalizer(corrupt.encode('utf-8'))
