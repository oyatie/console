# Qualified table identity gate regression

Owner /root/finish_migration, test-only base 9c846602511043014a039b77335e762e37032bea. Root owns source repair and admission. Cartesian doubt, Red Team, blast radius, and operability lenses.

Observed source cause: tokenize_sql splits at dot and quote; normalize_identifier receives already split names, making rsplit('.') ineffective. Sanitizer additionally removes double quotes and folds quoted case. A dot-only concatenation repair could conflate private.accounts with the allowlisted public.accounts, or quoted "Accounts" with accounts. Avoid adding public to an allowlist.

Affected consumers in backend/ci/gates/tenant-isolation/src/lib.rs:
- discover_created_tables / table_name_after_create_table: qualified CREATE, optional IF NOT EXISTS, classification identity.
- discover_org_columns: CREATE column, ALTER ADD COLUMN, ALTER SET NOT NULL; alter_table_target must use same identity.
- org_column_definition_has_not_null: token sequence matching; punctuation/name handling must not break scalar keyword recognition.
- discover_literal_rls: ALTER ENABLE/FORCE plus policy_target_table; same identity across differently spelled references.
- check_owner_only_table_grants: presently any owner-only basename anywhere in GRANT token stream counts, including a column name or other-schema relation. Qualification handling must interpret relation targets, preserving rejection for defaultpublic owner-only relations and both console_rt/PUBLIC recipients.
- discover_dynamic_rls / extract_array_string_literals: separate raw-text source of fact keys. Existing unqualified format('%I') rollout names must continue to match defaultpublic literal tables. Do not reinterpret one %I literal containing a dot as schema qualification.
- evaluate_facts: global, owner-only, nullable allowlists and aggregate identities. Only actual defaultpublic identities receive existing exceptions; nonpublic relations remain separate.

Regression entry is actual public check_migrations_root on isolated real migration files. Positive control, qualified/unqualified mixed spelling, quoted lowercase and whitespace, nullable exception, unknown classification, cross-schema isolation, same-schema sibling evidence isolation, quoted case/dot distinction, grants and existing dynamic rollout are covered. These are static scanner contracts, not database RLS execution or complete SQL semantic proof. Existing scanner's broader dynamic-loop/GUC heuristics are not expanded by this repair.

Stop: tests need source changes before approval, allowlist/schema/grant/dependency changes, or evidence that source repair must expand beyond shared qualification identity. Root must admit clean-tree RED and rerun same probe plus existing lib tests and real repository gate.
