"""Private Company outputs must preserve existing bytes without installing custody."""
import hashlib
import importlib.util
import json
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location("company_custody_generator",ROOT/"ops/generate-account-custody.py")
GENERATOR=importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)
PARSER="ops/postgres-company-enrollment-input.sql"
PARSER_SHA256="cbc685bff861fec809b930e57673cce5426a771a236323471955510de0631290"
SCHEMA="ops/postgres-company-enrollment-schema.sql"
SCHEMA_SHA256="7e1dbee080cb3f09d0133ac9a30857ee5101a076e1cec171ec06849116e672bb"
INTAKE="ops/postgres-company-enrollment-intake.sql"
INTAKE_SHA256="2d8e5bc991e35ca1c8195d5ee9f0b5c2f367a96e4d77cf613945f9d533cc7532"
NEW="ops/postgres-company-enrollment-guards.sql"
EXPECTED={'backend/app/src/account_custody_state.sql': '5fb0a2608c5904fe0c1d58723290b9603afb9af08eef19c410d023a484230865', 'ops/postgres-finalize-account-custody.sql': 'fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8', 'ops/account-custody-migrations.sha384': '736d28bc7b3b5cf15ca4c341fd3622dc76521db9dbf974b75cef7e73eb68fbf5', 'ops/postgres-verify-account-native.sql': 'c4e30ee9560b4d443acad1813a63667cbab9a32aadfbd893542f4b054bf6b308', 'backend/app/src/account_credential_custody_state.sql': '0bc6fe6579414ebed546d076c0ec1b7f1cc5ba30f4fe5e135159f4e75c0416a2', 'ops/postgres-finalize-account-credentials.sql': '2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9'}

COMPANY_PROFILE_OUTPUTS={'ops/postgres-company-enrollment-owner.sql': '2a7fffb57eedff3c3ba5981015e22f53ea80972e3222c38ff1c6ace1f0858d2d', 'ops/postgres-capture-company-enrollment-custody.sql': '6ef7b2754176952da89a0b61a7ba999d64b4c12e70fc949add18c63171b0abb5', 'backend/app/src/company_enrollment_custody_state.sql': 'fb2030a4d2891b206c41454ce8258d65920034e8ef817c5f7a812569aae614bd', 'ops/postgres-finalize-company-enrollment.sql': 'bc35b52d5692e474a3c890dde73112f56e7b85a241ab390075d58b5d5b43a92f'}


# Existing native policy outputs were added before this test's roster caught up.
# Pins are exact15441 historical bytes, independently preserved by capture work.
POLICY_PROFILE_OUTPUTS={
    'ops/postgres-native-company-policy-owner.sql': '833300caeac6874f35ebb08017bfbb7891a5fe765a4c761b139466ae221573d5',
    'ops/postgres-capture-native-company-policy-custody.sql': 'd5d2c45c691383ddde9ac213d9fe2674db0750f3ceabdc733eaaf2a570d937e3',
    'ops/postgres-native-company-policy-custody-state.sql': '072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479',
    'backend/app/src/native_company_policy_custody_state.sql': '072794defc065f8730eafeab66a111bdd2439c49b8c1d7c3052ca83a7edf1479',
    'ops/postgres-finalize-native-company-policy.sql': '4e7fc41b1d2ed6c2155d44d43347c18815ed9e70611eb9285f21e1c590bf996a',
}
# Declared capture additions; not custody/profile admission. A corrected capture
# requires independent review before updating its exact artifact pin.
POLICY_CAPTURE_OUTPUTS={
    'ops/postgres-native-company-policy-v2-owner.sql': 'f2050f21ef8151339289b2f013abdd543e90f8803fb6f6d4b9f004abf4409602',
    'ops/postgres-capture-native-company-policy-v2-custody.sql': '7534375fbae287ccf5e5015815e788ef0d7a379eed623af9a72f5db0d89614b4',
}

# Separately reviewed exact dual-codec finalizer and reader-first classifier.
POLICY_FINALIZER_OUTPUTS={
    'ops/postgres-finalize-native-company-policy-v2.sql': 'ec945607e209b93843116ae2b2a20772797dce38ff7884fb96081f09651f7d8e',
    'ops/postgres-native-company-policy-v2-custody-state.sql': 'e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc',
    'backend/app/src/native_company_policy_v2_custody_state.sql': 'e507d75f446ad8d3e0befe9321d94731a1a2cc9b3e0e9c0306c1459a098a53cc',
}

# Independently reviewed Directory capture, classifier, and guarded finalizer.
DIRECTORY_OUTPUTS={
    'ops/postgres-native-people-directory-owner.sql': '3dd0524bc751eba2aa806ee0dc670dc3cecbf8838c20b83896773a0c5b6016a0',
    'ops/postgres-capture-native-people-directory-custody.sql': 'bc8a1f87f57cd676ca1a3deae12263b1cca4a290c71a12e9a1749d189e090ca7',
    'ops/postgres-capture-native-people-directory-staged-custody.sql': '890ebc034fe9836f45e26c13d05352085db753f17e4a72b1ec05b6ecbe0a0e36',
    'ops/postgres-native-people-directory-custody-state.sql': '20c96bc2a9264d5ed4b86cb948cbe0574450243509263469c45af955d5aff3dd',
    'backend/app/src/native_people_directory_custody_state.sql': '20c96bc2a9264d5ed4b86cb948cbe0574450243509263469c45af955d5aff3dd',
    'ops/postgres-finalize-native-people-directory.sql': 'f4f99cf873c2ab970789e44ccf9737f2dd38f6dc6b05f1849fbd4461bf6a2357',
}

class CompanyInputGeneratorTests(unittest.TestCase):
    def test_one_new_output_preserves_all_existing_generated_bytes(self):
        outputs=GENERATOR.generated_files()
        self.assertEqual(set(outputs),set(EXPECTED)|{PARSER,SCHEMA,INTAKE,NEW}|set(COMPANY_PROFILE_OUTPUTS)|set(POLICY_PROFILE_OUTPUTS)|set(POLICY_CAPTURE_OUTPUTS)|set(POLICY_FINALIZER_OUTPUTS)|set(DIRECTORY_OUTPUTS))
        for path,digest in EXPECTED.items():
            if path == "ops/account-custody-migrations.sha384":
                # Reviewed9b9baed6a appended0229; retain exact historical228pin.
                suffix = b'229\t08ece6d6edf558d06eff11297da18184e9a762ff4f5455fab135c84a1b34dbef449b6c957f3d7fce197af113f821a659\n'
                # Prove the exact reviewed230 append before projecting historical229.
                checksum230 = '69b9f0de4175868ce73f01ce52ac47ee5607ae6d88c91ab9bc519112ea1cb9682945c2cc0b87a91a78edb32a2b3fade1'
                self.assertEqual(hashlib.sha384((ROOT/'backend/crates/platform/db/migrations/0230_native_people_directory_storage.sql').read_bytes()).hexdigest(),checksum230)
                suffix230 = ('230\t'+checksum230+'\n').encode()
                complete_ledger = outputs[path].encode()
                self.assertTrue(complete_ledger.endswith(suffix230))
                self.assertEqual(len(complete_ledger.splitlines()),230)
                ledger = complete_ledger[:-len(suffix230)]
                self.assertTrue(ledger.endswith(suffix))
                self.assertEqual(len(ledger.splitlines()),229)
                self.assertEqual(hashlib.sha256(ledger[:-len(suffix)]).hexdigest(),digest,path)
            else:
                self.assertEqual(hashlib.sha256(outputs[path].encode()).hexdigest(),digest,path)
            self.assertEqual(outputs[path].encode(),(ROOT/path).read_bytes(),path)
        for path,digest in {**COMPANY_PROFILE_OUTPUTS,**POLICY_PROFILE_OUTPUTS,**POLICY_CAPTURE_OUTPUTS,**POLICY_FINALIZER_OUTPUTS,**DIRECTORY_OUTPUTS}.items():
            self.assertEqual(hashlib.sha256(outputs[path].encode()).hexdigest(),digest,path)
            self.assertEqual(outputs[path].encode(),(ROOT/path).read_bytes(),path)
        emitted=GENERATOR.company_enrollment_input_sql()
        self.assertEqual(outputs[PARSER],emitted)
        self.assertEqual((ROOT/PARSER).read_bytes(),emitted.encode())
        self.assertEqual(hashlib.sha256(emitted.encode()).hexdigest(),PARSER_SHA256)
        self.assertIn("CREATE FUNCTION public.company_enrollment_decode_input_v1",emitted)
        self.assertIn("UNINSTALLED",emitted)
        for path in EXPECTED:
            self.assertNotIn("CREATE FUNCTION public.company_enrollment_decode_input_v1",outputs[path],path)
        schema=GENERATOR.company_enrollment_schema_sql()
        self.assertEqual(outputs[SCHEMA],schema)
        self.assertEqual((ROOT/SCHEMA).read_bytes(),schema.encode())
        self.assertEqual(hashlib.sha256(schema.encode()).hexdigest(),SCHEMA_SHA256)
        self.assertIn("UNINSTALLED",schema)
        for relation in ("company_enrollment_requests", "company_enrollment_receipts", "company_enrollment_request_events"):
            declaration="CREATE TABLE public."+relation
            self.assertIn(declaration,schema)
            for path in set(EXPECTED)|{PARSER}:
                self.assertNotIn(declaration,outputs[path],path)

        intake=GENERATOR.company_enrollment_intake_sql()
        self.assertEqual(outputs[INTAKE],intake)
        self.assertEqual((ROOT/INTAKE).read_bytes(),intake.encode())
        self.assertEqual(hashlib.sha256(intake.encode()).hexdigest(),INTAKE_SHA256)
        self.assertIn("UNINSTALLED",intake)
        for name in ("company_enrollment_session_material_v1", "company_enrollment_prepare_v1", "company_enrollment_status_v1", "company_enrollment_cancel_v1"):
            declaration="CREATE FUNCTION public."+name
            self.assertIn(declaration,intake)
            for path in set(EXPECTED)|{PARSER,SCHEMA}:
                self.assertNotIn(declaration,outputs[path],path)

        guards=GENERATOR.company_enrollment_guards_sql()
        self.assertEqual(outputs[NEW],guards)
        self.assertEqual((ROOT/NEW).read_bytes(),guards.encode())
        self.assertIn("UNINSTALLED",guards)
        for name in ("company_enrollment_request_guard_v1", "company_enrollment_request_preserve_v1", "company_enrollment_event_immutable_v1", "company_enrollment_event_guard_v1", "company_enrollment_receipt_intake_guard_v1", "company_enrollment_intake_closure_v1"):
            declaration="CREATE FUNCTION public."+name
            self.assertIn(declaration,guards)
            for path in set(EXPECTED)|{PARSER,SCHEMA,INTAKE}:
                self.assertNotIn(declaration,outputs[path],path)

if __name__=="__main__":
    unittest.main()
