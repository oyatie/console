// Test-only immutable literal fixtures for the reviewed Organization V12 codec3 appendix.
// Design SHA256 c26751c6a49b759de50ee9e9dd42c7255b2403d1c874673f7b51b91d23dd3086.
// They establish no catalog installation, owner authority or serving acceptance.

pub(super) const POSITIVE_JSON: &str = r###"[
  {
    "name": "install",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "bytes": 123,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Install",
      "action_byte": null,
      "recipient_account_id": null,
      "assignment_expectation": null,
      "expires_at_microseconds": null
    }
  },
  {
    "name": "Read_grant_new",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "bytes": 149,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Grant",
      "action_byte": 1,
      "recipient_account_id": "44444444-4444-4444-8444-444444444444",
      "assignment_expectation": null,
      "expires_at_microseconds": 1790607600000000
    }
  },
  {
    "name": "Read_grant_replace",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "bytes": 181,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Grant",
      "action_byte": 1,
      "recipient_account_id": "44444444-4444-4444-8444-444444444444",
      "assignment_expectation": {
        "role_revision": 1,
        "assignment_id": "55555555-5555-4555-8555-555555555555",
        "assignment_revision": 9
      },
      "expires_at_microseconds": 1790607600000000
    }
  },
  {
    "name": "Read_revoke",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "bytes": 156,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Revoke",
      "action_byte": 1,
      "recipient_account_id": null,
      "assignment_expectation": {
        "role_revision": 1,
        "assignment_id": "55555555-5555-4555-8555-555555555555",
        "assignment_revision": 9
      },
      "expires_at_microseconds": null
    }
  },
  {
    "name": "CreateSite_grant_new",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "bytes": 149,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Grant",
      "action_byte": 2,
      "recipient_account_id": "44444444-4444-4444-8444-444444444444",
      "assignment_expectation": null,
      "expires_at_microseconds": 1790607600000000
    }
  },
  {
    "name": "CreateSite_grant_replace",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "bytes": 181,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Grant",
      "action_byte": 2,
      "recipient_account_id": "44444444-4444-4444-8444-444444444444",
      "assignment_expectation": {
        "role_revision": 1,
        "assignment_id": "55555555-5555-4555-8555-555555555555",
        "assignment_revision": 9
      },
      "expires_at_microseconds": 1790607600000000
    }
  },
  {
    "name": "CreateSite_revoke",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "bytes": 156,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Revoke",
      "action_byte": 2,
      "recipient_account_id": null,
      "assignment_expectation": {
        "role_revision": 1,
        "assignment_id": "55555555-5555-4555-8555-555555555555",
        "assignment_revision": 9
      },
      "expires_at_microseconds": null
    }
  },
  {
    "name": "CorrectSiteName_grant_new",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "bytes": 149,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Grant",
      "action_byte": 3,
      "recipient_account_id": "44444444-4444-4444-8444-444444444444",
      "assignment_expectation": null,
      "expires_at_microseconds": 1790607600000000
    }
  },
  {
    "name": "CorrectSiteName_grant_replace",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "bytes": 181,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Grant",
      "action_byte": 3,
      "recipient_account_id": "44444444-4444-4444-8444-444444444444",
      "assignment_expectation": {
        "role_revision": 1,
        "assignment_id": "55555555-5555-4555-8555-555555555555",
        "assignment_revision": 9
      },
      "expires_at_microseconds": 1790607600000000
    }
  },
  {
    "name": "CorrectSiteName_revoke",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "bytes": 156,
    "expected": {
      "actor_account_id": "11111111-1111-4111-8111-111111111111",
      "company_id": "33333333-3333-4333-8333-333333333333",
      "command_id": "22222222-2222-4222-8222-222222222222",
      "expected_company_epoch": 7,
      "catalog_version": "native-org-unit-work-v1",
      "manifest_digest": "e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
      "operation": "Revoke",
      "action_byte": 3,
      "recipient_account_id": null,
      "assignment_expectation": {
        "role_revision": 1,
        "assignment_id": "55555555-5555-4555-8555-555555555555",
        "assignment_revision": 9
      },
      "expires_at_microseconds": null
    }
  }
]
"###;

pub(super) const REFUSAL_JSON: &str = r###"[
  {
    "name": "install/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580100",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb405801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415901",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415800",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415804",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444400",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444000006",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c0000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb40580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41590201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580001444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580401444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff01444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580200444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580204444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802ff444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/nil-recipient",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201000000000000000000000000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-witness-tag/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-witness-tag/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440200065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-witness-tag/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444ff00065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-expiry/-9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444008000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-expiry/-62135596860000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444400ff234000fd40b900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-expiry/1790607599999999",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1bff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-expiry/1790607600000001",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c01",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-expiry/253402268400000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444000384440541429c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/invalid-expiry/9223372036854775807",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444007fffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/149",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/150",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/151",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/152",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/153",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/154",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/155",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/156",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/157",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/158",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555555555455585",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/159",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/160",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/161",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555555555455585555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/162",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/163",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/164",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555555555455585555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/165",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/166",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555555555555500",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/167",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555555555455585555555555555550000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/168",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/169",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/170",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/171",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/172",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/173",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/174",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/175",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000090006",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/176",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/177",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/178",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/179",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/truncated/180",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c0000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb405802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415902014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415800014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415804014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802004444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802044444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802ff4444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/nil-recipient",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802010000000000000000000000000000000001000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-witness-tag/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444400000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-witness-tag/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444402000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-witness-tag/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444ff000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-expiry/-9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000098000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-expiry/-62135596860000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020144444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000000009ff234000fd40b900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-expiry/1790607599999999",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1bff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-expiry/1790607600000001",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c01",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-expiry/253402268400000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000090384440541429c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-expiry/9223372036854775807",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580201444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000097fffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-role-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000055555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-role-revision/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000255555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-role-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401ffffffffffffffff55555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/nil-assignment",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000100000000000000000000000000000000000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-assignment-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-assignment-revision/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555800000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/invalid-assignment-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555ffffffffffffffff00065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000015555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000015555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555555545",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000015555555555554555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555555545558555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000015555555555554555855555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555555545558555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000015555555555554555855555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555555545558555555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/149",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000015555555555554555855555555555555500",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/150",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/151",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555555545558555555555555555000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/152",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000015555555555554555855555555555555500000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/153",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/154",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555555545558555555555555555000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/truncated/155",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030100000000000000015555555555554555855555555555555500000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555555545558555555555555555000000000000000900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb405803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415903010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415800010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415804010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803000000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803040000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803ff0000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-role-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000000555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-role-revision/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000002555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-role-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301ffffffffffffffff555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/nil-assignment",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001000000000000000000000000000000000000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-assignment-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-assignment-revision/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555558000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/invalid-assignment-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301000000000000000155555555555545558555555555555555ffffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444400",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444000006",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c0000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb40580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41590202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580002444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580402444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff02444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580200444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580204444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802ff444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/nil-recipient",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202000000000000000000000000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-witness-tag/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-witness-tag/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440200065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-witness-tag/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444ff00065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-expiry/-9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444008000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-expiry/-62135596860000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444400ff234000fd40b900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-expiry/1790607599999999",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1bff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-expiry/1790607600000001",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c01",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-expiry/253402268400000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444000384440541429c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/invalid-expiry/9223372036854775807",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444007fffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/149",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/150",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/151",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/152",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/153",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/154",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/155",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/156",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/157",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/158",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555555555455585",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/159",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/160",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/161",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555555555455585555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/162",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/163",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/164",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555555555455585555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/165",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/166",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555555555555500",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/167",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555555555455585555555555555550000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/168",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/169",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/170",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/171",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/172",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/173",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/174",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/175",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000090006",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/176",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/177",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/178",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/179",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/truncated/180",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c0000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb405802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415902024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415800024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415804024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802004444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802044444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802ff4444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/nil-recipient",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802020000000000000000000000000000000001000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-witness-tag/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444400000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-witness-tag/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444402000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-witness-tag/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444ff000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-expiry/-9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000098000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-expiry/-62135596860000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020244444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000000009ff234000fd40b900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-expiry/1790607599999999",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1bff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-expiry/1790607600000001",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c01",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-expiry/253402268400000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000090384440541429c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-expiry/9223372036854775807",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580202444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000097fffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-role-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000055555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-role-revision/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000255555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-role-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401ffffffffffffffff55555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/nil-assignment",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000100000000000000000000000000000000000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-assignment-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-assignment-revision/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555800000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/invalid-assignment-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555ffffffffffffffff00065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000015555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000015555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555555545",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000015555555555554555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555555545558555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000015555555555554555855555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555555545558555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000015555555555554555855555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555555545558555555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/149",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000015555555555554555855555555555555500",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/150",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/151",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555555545558555555555555555000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/152",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000015555555555554555855555555555555500000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/153",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/154",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555555545558555555555555555000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/truncated/155",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030200000000000000015555555555554555855555555555555500000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555555545558555555555555555000000000000000900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb405803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415903020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415800020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415804020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803000000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803040000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803ff0000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-role-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000000555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-role-revision/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000002555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-role-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302ffffffffffffffff555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/nil-assignment",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001000000000000000000000000000000000000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-assignment-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-assignment-revision/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555558000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/invalid-assignment-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302000000000000000155555555555545558555555555555555ffffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444400",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444000006",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c0000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb40580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41590203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580003444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580403444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff03444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580200444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580204444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802ff444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/nil-recipient",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203000000000000000000000000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-witness-tag/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-witness-tag/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440200065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-witness-tag/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444ff00065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-expiry/-9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444008000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-expiry/-62135596860000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444400ff234000fd40b900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-expiry/1790607599999999",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1bff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-expiry/1790607600000001",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c01",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-expiry/253402268400000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444000384440541429c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/invalid-expiry/9223372036854775807",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444007fffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/149",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/150",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/151",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/152",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/153",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/154",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/155",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/156",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/157",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/158",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555555555455585",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/159",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/160",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/161",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555555555455585555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/162",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/163",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/164",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555555555455585555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/165",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/166",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555555555555500",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/167",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555555555455585555555555555550000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/168",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/169",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/170",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/171",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/172",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/173",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/174",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/175",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000090006",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/176",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/177",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/178",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/179",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/truncated/180",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c0000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb405802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415902034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415800034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415804034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802004444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802044444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802ff4444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/nil-recipient",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802030000000000000000000000000000000001000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-witness-tag/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444400000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-witness-tag/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444402000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-witness-tag/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444ff000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-expiry/-9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000098000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-expiry/-62135596860000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158020344444444444444448444444444444444010000000000000001555555555555455585555555555555550000000000000009ff234000fd40b900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-expiry/1790607599999999",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1bff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-expiry/1790607600000001",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c01",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-expiry/253402268400000000",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000090384440541429c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-expiry/9223372036854775807",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580203444444444444444484444444444444440100000000000000015555555555554555855555555555555500000000000000097fffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-role-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000055555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-role-revision/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000255555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-role-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401ffffffffffffffff55555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/nil-assignment",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000100000000000000000000000000000000000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-assignment-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-assignment-revision/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555800000000000000000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/invalid-assignment-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555ffffffffffffffff00065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/0",
    "codec": 3,
    "hex": "",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/1",
    "codec": 3,
    "hex": "63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/2",
    "codec": 3,
    "hex": "636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/3",
    "codec": 3,
    "hex": "636f6e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/4",
    "codec": 3,
    "hex": "636f6e73",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/5",
    "codec": 3,
    "hex": "636f6e736f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/6",
    "codec": 3,
    "hex": "636f6e736f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/7",
    "codec": 3,
    "hex": "636f6e736f6c65",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/8",
    "codec": 3,
    "hex": "636f6e736f6c652e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/9",
    "codec": 3,
    "hex": "636f6e736f6c652e63",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d7061",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e79",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f7267",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d75",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e6974",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/35",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/36",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/37",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/38",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/39",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/40",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/41",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/42",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/43",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/44",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/45",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/46",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/47",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/48",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/49",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/51",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/52",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/53",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/54",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/55",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/56",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/57",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/58",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/59",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/60",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/61",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/62",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/63",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/64",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/65",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/67",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/68",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/69",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/70",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/71",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/72",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/73",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/74",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/75",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/76",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/77",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/78",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/79",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/80",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/81",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/82",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/83",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/84",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/85",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/86",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/87",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/88",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/89",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c69637900000311111111111141118111111111111111333333333333433383333333333333332222222222224222822222222222222200000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/90",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/91",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/92",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/93",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a6",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/94",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/95",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/96",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f1",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/97",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/98",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d70",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/99",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705d",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/100",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/101",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/102",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c61",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/103",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/104",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf9",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/105",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/106",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf0",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/107",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/108",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/109",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/110",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c329",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/111",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c32932",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/112",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/113",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c3",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/114",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c304",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/115",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/116",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d2",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/117",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20a",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/118",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/119",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/120",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/121",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/122",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/123",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/124",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/125",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/126",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/127",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/128",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/129",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/130",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/131",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/132",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/133",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/134",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000015555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/135",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/136",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/137",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000015555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/138",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/139",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555555545",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/140",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000015555555555554555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/141",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/142",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555555545558555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/143",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000015555555555554555855555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/144",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/145",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555555545558555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/146",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000015555555555554555855555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/147",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/148",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555555545558555555555555555",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/149",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000015555555555554555855555555555555500",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/150",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/151",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555555545558555555555555555000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/152",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000015555555555554555855555555555555500000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/153",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/154",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555555545558555555555555555000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/truncated/155",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158030300000000000000015555555555554555855555555555555500000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/suffix/00",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555555545558555555555555555000000000000000900",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/suffix/010203",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009010203",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/0",
    "codec": 3,
    "hex": "626f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/1",
    "codec": 3,
    "hex": "636e6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/2",
    "codec": 3,
    "hex": "636f6f736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/3",
    "codec": 3,
    "hex": "636f6e726f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/4",
    "codec": 3,
    "hex": "636f6e736e6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/5",
    "codec": 3,
    "hex": "636f6e736f6d652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c642e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652f636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e626f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636e6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6c70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d71616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70606e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616f792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e782e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792f6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6e72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f73672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72662d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672c756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d746e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756f69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e68742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69752d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742c706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d716f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706e6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6d6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6863790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6962790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963780000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790100031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/32",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790001031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/prefix-bit/33",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000021111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e156a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e057a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a78f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68e52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f53f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/5",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f03d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/6",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13c705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/7",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d715dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/8",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705cc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/9",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc28c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/10",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38d618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/11",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c608bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/12",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618af97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/13",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf87bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/14",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97af09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/15",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf19b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/16",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09a37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/17",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b36c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/18",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c2293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/19",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3283255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/20",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293355c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/21",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293254c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/22",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c20442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/23",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30542d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/24",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30443d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/25",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d30aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/26",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20bee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/27",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aef51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/28",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee50fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/29",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fa415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/30",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb405803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/manifest-bit/31",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415903030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/unknown-operation/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415800030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/unknown-operation/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415804030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/unknown-operation/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb4158ff030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/nil-id/34",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000030000000000000000000000000000000033333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/nil-id/50",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111100000000000000000000000000000000222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/nil-id/66",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333000000000000000000000000000000000000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/platform-company",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111110000000000000000000000000000face222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-company-epoch/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-company-epoch/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222228000000000000000e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-company-epoch/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c696379000003111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222ffffffffffffffffe056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/unknown-action/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803000000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/unknown-action/4",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803040000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/unknown-action/255",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803ff0000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-role-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000000555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-role-revision/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000002555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-role-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303ffffffffffffffff555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/nil-assignment",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001000000000000000000000000000000000000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-assignment-revision/0",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-assignment-revision/9223372036854775808",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555558000000000000000",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/invalid-assignment-revision/18446744073709551615",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303000000000000000155555555555545558555555555555555ffffffffffffffff",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/wrong-codec/-1",
    "codec": -1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/wrong-codec/0",
    "codec": 0,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/wrong-codec/1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/wrong-codec/2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/wrong-codec/4",
    "codec": 4,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/wrong-codec/32767",
    "codec": 32767,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec1_literal_0/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd01",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec1_literal_1/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd02444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec1_literal_2/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec1_literal_3/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec2_literal_0/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e01",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec2_literal_1/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e0201444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec2_literal_2/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e02014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec2_literal_3/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e03010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec2_literal_4/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e0202444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec2_literal_5/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e02024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "codec2_literal_6/cross-family",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e03020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/valid-operation-wrong-shape/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "install/valid-operation-wrong-shape/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580101444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_new/valid-operation-wrong-shape/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580301444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_grant_replace/valid-operation-wrong-shape/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "Read_revoke/valid-operation-wrong-shape/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802010000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580102444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_new/valid-operation-wrong-shape/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580302444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_grant_replace/valid-operation-wrong-shape/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CreateSite_revoke/valid-operation-wrong-shape/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802020000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580103444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_new/valid-operation-wrong-shape/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb41580303444444444444444484444444444444440000065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_grant_replace/valid-operation-wrong-shape/3",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415803034444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/valid-operation-wrong-shape/1",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415801030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  },
  {
    "name": "CorrectSiteName_revoke/valid-operation-wrong-shape/2",
    "codec": 3,
    "hex": "636f6e736f6c652e636f6d70616e792e6f72672d756e69742d706f6c6963790000031111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007e056a68f52f13d705dc38c618bf97bf09b37c3293255c30442d20aee51fb415802030000000000000001555555555555455585555555555555550000000000000009",
    "expected_error": "invalid Company business policy input"
  }
]
"###;

pub(super) const PREDECESSOR_JSON: &str = r###"[
  {
    "source": "backend/crates/identity/application/src/company_policy/business/tests.rs",
    "name": "codec1_literal_0",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd01"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/business/tests.rs",
    "name": "codec1_literal_1",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd02444444444444444484444444444444440000065c8c51ee1c00"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/business/tests.rs",
    "name": "codec1_literal_2",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/business/tests.rs",
    "name": "codec1_literal_3",
    "codec": 1,
    "hex": "636f6e736f6c652e636f6d70616e792e627573696e6573732d706f6c696379000001111111111111411181111111111111113333333333334333833333333333333322222222222242228222222222222222000000000000000707781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd030000000000000001555555555555455585555555555555550000000000000009"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/people_policy_tests.rs",
    "name": "codec2_literal_0",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e01"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/people_policy_tests.rs",
    "name": "codec2_literal_1",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e0201444444444444444484444444444444440000065c8c51ee1c00"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/people_policy_tests.rs",
    "name": "codec2_literal_2",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e02014444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/people_policy_tests.rs",
    "name": "codec2_literal_3",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e03010000000000000001555555555555455585555555555555550000000000000009"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/people_policy_tests.rs",
    "name": "codec2_literal_4",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e0202444444444444444484444444444444440000065c8c51ee1c00"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/people_policy_tests.rs",
    "name": "codec2_literal_5",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e02024444444444444444844444444444444401000000000000000155555555555545558555555555555555000000000000000900065c8c51ee1c00"
  },
  {
    "source": "backend/crates/identity/application/src/company_policy/people_policy_tests.rs",
    "name": "codec2_literal_6",
    "codec": 2,
    "hex": "636f6e736f6c652e636f6d70616e792e70656f706c652d706f6c6963790000011111111111114111811111111111111133333333333343338333333333333333222222222222422282222222222222220000000000000007591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e03020000000000000001555555555555455585555555555555550000000000000009"
  }
]
"###;
