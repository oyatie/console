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
NEW="ops/postgres-company-enrollment-schema.sql"
EXPECTED={'backend/app/src/account_custody_state.sql': '5fb0a2608c5904fe0c1d58723290b9603afb9af08eef19c410d023a484230865', 'ops/postgres-finalize-account-custody.sql': 'fb532ef3b82f6d03b58d6e164a26567683039444098cf1a0a98d33e27e5dd4f8', 'ops/account-custody-migrations.sha384': '736d28bc7b3b5cf15ca4c341fd3622dc76521db9dbf974b75cef7e73eb68fbf5', 'ops/postgres-verify-account-native.sql': 'c4e30ee9560b4d443acad1813a63667cbab9a32aadfbd893542f4b054bf6b308', 'backend/app/src/account_credential_custody_state.sql': '0bc6fe6579414ebed546d076c0ec1b7f1cc5ba30f4fe5e135159f4e75c0416a2', 'ops/postgres-finalize-account-credentials.sql': '2f960163c9bd8832cdd3a058a6ae69d7503cf1009c476ef292af9443036624d9'}

class CompanyInputGeneratorTests(unittest.TestCase):
    def test_one_new_output_preserves_all_existing_generated_bytes(self):
        outputs=GENERATOR.generated_files()
        self.assertEqual(set(outputs),set(EXPECTED)|{PARSER,NEW})
        for path,digest in EXPECTED.items():
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
        self.assertEqual(outputs[NEW],schema)
        self.assertEqual((ROOT/NEW).read_bytes(),schema.encode())
        self.assertIn("UNINSTALLED",schema)
        for relation in ("company_enrollment_requests", "company_enrollment_receipts", "company_enrollment_request_events"):
            declaration="CREATE TABLE public."+relation
            self.assertIn(declaration,schema)
            for path in set(EXPECTED)|{PARSER}:
                self.assertNotIn(declaration,outputs[path],path)

if __name__=="__main__":
    unittest.main()
