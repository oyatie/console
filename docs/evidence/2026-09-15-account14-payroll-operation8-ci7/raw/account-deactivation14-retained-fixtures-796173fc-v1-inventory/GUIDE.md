# Retained fixture prerequisite amendment

Private author /root/inventory_repair; root remains sole integration/Cargo/DB writer.
Exact base796173fc484279dbb9cca103caa57e026719ffaf; two current preimages match.
Authorization: root standing authorization for independently reviewed retained
fixture prerequisites, relayed by /root/capture_review. Independent reviewer:
/root/capture_review. Product candidate V1 remains unchanged.

The mandatory guard is installed by the actual custody finalizer, not numbered
migrations alone. Both retained suites previously ran migrations directly as the
SQLx administrator and then SET ROLE console_rt, leaving no valid guard profile.
Simply calling finalize afterward would also have the wrong table ownership.

Each of the existing three cases per file now starts from an empty SQLx database
and calls the established prepare_account_test_database helper before its
unchanged runtime pool setup. That helper checks marked disposable admin identity,
hands the empty database to the actual console_app migration owner, runs every
numbered migration, runs the real production finalizer, and tests the real Auth
projection. The SQLx migrations=false annotation selects that actual migration
path; it does not skip schema installation or weaken an assertion.

Only one import, three annotations and three prerequisite calls change per file.
Removing those exact changes recovers every original byte. All six names and all
existing assertions, fixtures, runtime SET ROLE operations and product calls stay
unchanged. Source-only parser/format and four private patch apply/check/inverse
commands pass. Compiler discovery0, Rust/Cargo/DB execution0. Root must run both
unfiltered existing targets after independently reviewed product integration.

Pre-mortem: missing guard setup masks product behavior with42883, or administrator
migration ownership makes finalizer refuse. Blast radius: two retained test
fixtures only. Detection: exact inverse proof plus unchanged complete target runs.
Rollback: discard this private patch; no integration changes. Stop on source drift,
assertion changes, missing real helper prerequisites, or unreviewed failure.
Lenses: Essentialism, Chesterton's Fence, Red Team, Operability, Blast radius,
Zero trust. HOLD: independent fixture approval, root runtime/exact-head proof,
production authority. No new product claims or native enrollment/Auth7 scope.
