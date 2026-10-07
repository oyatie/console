//! Native Company identity owner acceptance, using the actual enclosing fixture.
use super::*;

#[path = "native_identity_birth_tests.rs"]
mod birth;
#[path = "native_identity_current_tests.rs"]
mod current;
#[path = "native_identity_effect_tests.rs"]
mod effect;
#[path = "native_identity_metadata_tests.rs"]
mod metadata;
#[path = "native_identity_migration_tests.rs"]
mod migration;

#[path = "native_birth_oracles.rs"]
mod birth_oracles;
use birth_oracles::{
    assert_native_catalog_content, identity_digests_match, identity_graph_matches, identity_rows,
};
use effect::assert_only_pending_company_intake;
use metadata::{identity_catalog_contract, identity_catalog_matches, identity_catalog_observation};

#[path = "native_topology_metadata_tests.rs"]
mod topology_metadata;
#[path = "native_topology_migration_tests.rs"]
mod topology_migration;
pub(super) use topology_metadata::assert_native_topology_metadata;
use topology_migration::assert_topology_backfill;

#[path = "native_topology_preextension.rs"]
mod topology_preextension;
use topology_preextension::prepare_company_preextension_database;

#[path = "native_catalog_migration_tests.rs"]
mod catalog_migration;
use catalog_migration::{
    assert_catalog_migration, catalog_allowlist_preserved, legacy_catalog_replay_after_extension,
    legacy_catalog_shape, legacy_catalog_shape_preserved, populate_legacy_catalog,
};

#[path = "native_catalog_audit_tests.rs"]
mod catalog_audit;
#[path = "native_catalog_birth_tests.rs"]
mod catalog_birth;
use catalog_birth::{assert_native_catalog_attribution, native_catalog_attribution_matches};

use catalog_migration::legacy_catalog_attachment_after_extension;
