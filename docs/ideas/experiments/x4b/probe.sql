-- EXPERIMENT X4b -- FEASIBILITY PROBE SEED. NOT PRODUCTION SCHEMA. NOT A MIGRATION.
--
-- Deliberately NOT under backend/crates/platform/db/migrations/: it must never
-- take a migration slot and it is not intended to land. The ONE table this file
-- creates is prefixed x4b_ so it cannot be confused with the real thing; every
-- other object it touches is a SHIPPED table (run.sh applies the real migration
-- set first), which is the point -- B9's claim is about the real FK, so a
-- replica of ont_links could be wrong about exactly the thing under test.
--
-- Tests docs/ideas/ecosystem-plan-review.md:271-300 (finding B9) against
-- docs/ideas/ecosystem-plan-DRAFT.md §4.1 Tier N `grant` (:563) and §4.3
-- `grant_scope` (:666), which specify a grant as an ont_instances row whose
-- scope is an ont_link to `org_unit | organization | group`.
--
-- Run by run.sh as the topology admin (a superuser: seeding bypasses RLS on
-- purpose). Every ASSERTION runs as console_rt.

BEGIN;

-- ---------------------------------------------------------------------------
-- 0165:429-445 puts DEFERRABLE CONSTRAINT TRIGGERS on the ontology REGISTRY
-- tables demanding one matching protected audit row per mutation, and
-- 0165:354-357 lets only console_rt (with a proven parent mutation) or
-- console_ontology_cmd write that audit row -- so a seed script cannot satisfy
-- it. Two named registry triggers are disabled for the seed and re-enabled at
-- the end of this file.
--
-- This is scoped so it CANNOT rescue the claim under test:
--   * it touches only ont_object_types / ont_link_types -- the TYPE registry.
--     Nothing is disabled on ont_instances, ont_instance_revisions or ont_links.
--   * DISABLE TRIGGER does not disable RLS and does not disable foreign keys.
--     The two things B9 rests on stay fully armed, and run.sh proves it by
--     execution: CONTROL 1 shows ont_instances RLS still hides org A's row from
--     org B, and CASE 3a shows the ont_links FK still rejects a bad endpoint.
-- ---------------------------------------------------------------------------
ALTER TABLE ont_object_types DISABLE TRIGGER trg_ont_object_types_current_audit;
ALTER TABLE ont_link_types   DISABLE TRIGGER trg_ont_link_types_current_audit;

-- ---------------------------------------------------------------------------
-- The conglomerate: group G with two member 법인, A and B. Shipped tables:
-- groups (0060:13), group_memberships (0060:31), organizations.group_id
-- (0060:26-27).
-- ---------------------------------------------------------------------------
INSERT INTO groups (id, slug, name) VALUES
    ('99999999-9999-4999-8999-999999999999', 'x4b-group', 'X4b Holdings');

INSERT INTO organizations (id, slug, name, group_id) VALUES
    ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'x4b-org-a', 'X4b Subsidiary A',
     '99999999-9999-4999-8999-999999999999'),
    ('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', 'x4b-org-b', 'X4b Subsidiary B',
     '99999999-9999-4999-8999-999999999999');

INSERT INTO group_memberships (group_id, org_id) VALUES
    ('99999999-9999-4999-8999-999999999999', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'),
    ('99999999-9999-4999-8999-999999999999', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb');

-- The owner's requirement 3 made concrete: an HR officer whose 소속 is
-- subsidiary A, who must hold authority over the whole group.
INSERT INTO users (id, org_id, display_name, roles, is_active) VALUES
    ('11111111-1111-4111-8111-111111111111', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'X4b HR Officer (소속 = A)', ARRAY['ADMIN'], true);

-- The SHIPPED cross-org authority store, for the substrate comparison at the
-- end of run.sh. Tier O: no org_id, no RLS, no console_rt grant (0060:40-49).
INSERT INTO group_role_grants (group_id, user_id, group_role) VALUES
    ('99999999-9999-4999-8999-999999999999', '11111111-1111-4111-8111-111111111111',
     'GROUP_ADMIN');

-- ---------------------------------------------------------------------------
-- Tier N object types. `grant` is Tier N per §4.1:563. `organization` and
-- `group` types exist because §4.3:666 says the scope arms are LINK TARGETS,
-- and an ont_link target must be an ont_instances row -- so if the plan is to
-- work at all, a `group` instance has to exist somewhere. We give it the most
-- generous possible reading and mint one in EACH org.
-- ---------------------------------------------------------------------------
-- 0165:92-102 added a per-(org, stable_key) revision row that ont_object_types
-- FKs to (fk_ont_object_types_key_revision), so the key must be registered
-- before a type row can exist. Not part of the claim under test; satisfied here
-- so the real schema can be used unmodified.
INSERT INTO ont_object_type_key_revisions (org_id, stable_key, revision) VALUES
    ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'grant', 1),
    ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'organization_scope', 1),
    ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'group_scope', 1),
    ('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', 'group_scope', 1);

INSERT INTO ont_object_types
    (id, org_id, stable_key, title, backing_kind, schema_version, lifecycle_state, created_by)
VALUES
    ('c1000000-0000-4000-8000-000000000001', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'grant', 'Grant', 'instance', 1, 'published', '11111111-1111-4111-8111-111111111111'),
    ('c1000000-0000-4000-8000-000000000002', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'organization_scope', 'Organization (scope)', 'instance', 1, 'published', NULL),
    ('c1000000-0000-4000-8000-000000000003', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'group_scope', 'Group (scope)', 'instance', 1, 'published', NULL),
    -- org B mints its own `group_scope` type and instance: the sibling that
    -- needs to READ the group-scoped grant, modelled as generously as possible.
    ('c2000000-0000-4000-8000-000000000003', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
     'group_scope', 'Group (scope)', 'instance', 1, 'published', NULL);

-- `grant_scope` as §4.3:666 specifies it: a link type on `grant`, OneMany,
-- target = the group scope type. §0.12 requires it also be authored as a
-- property carrying config.link; irrelevant to the FK question under test.
INSERT INTO ont_link_types
    (id, org_id, object_type_id, stable_key, title, to_object_type_id, cardinality)
VALUES
    ('d1000000-0000-4000-8000-000000000001', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'c1000000-0000-4000-8000-000000000001', 'grant_scope', 'Grant scope',
     'c1000000-0000-4000-8000-000000000003', 'one_many');

-- ---------------------------------------------------------------------------
-- Instances. Every row is org-stamped because ont_instances.org_id is
-- NOT NULL (0155:18) -- there is no third option.
-- ---------------------------------------------------------------------------
INSERT INTO ont_instances (id, org_id, object_type_id, title, lifecycle_state) VALUES
    -- CONTROL: an ordinary org A instance whose only job is to be invisible
    -- from org B. If org B CAN see it, the harness is not exercising RLS.
    ('e1000000-0000-4000-8000-0000000000ff', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'c1000000-0000-4000-8000-000000000002', 'x4b control row minted in org A', 'active'),
    -- the scope endpoints
    ('e1000000-0000-4000-8000-000000000001', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'c1000000-0000-4000-8000-000000000002', 'organization scope: A', 'active'),
    ('e1000000-0000-4000-8000-000000000002', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'c1000000-0000-4000-8000-000000000003', 'group scope: G (minted in A)', 'active'),
    ('e2000000-0000-4000-8000-000000000002', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
     'c2000000-0000-4000-8000-000000000003', 'group scope: G (minted in B)', 'active'),
    -- the two grants
    ('e1000000-0000-4000-8000-000000000010', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'c1000000-0000-4000-8000-000000000001', 'grant: company-scoped, org A', 'active'),
    ('e1000000-0000-4000-8000-000000000011', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
     'c1000000-0000-4000-8000-000000000001', 'grant: group-scoped over G', 'active');

-- Revisions carry the authority payload. `grant_subject` is a property (UUID),
-- not a link, per §4.3:665 -- and `party` does not exist yet, so the subject is
-- the HR officer's user id standing in for their party id. The scope descriptor
-- {scope_level, scope_node_id} is the shape B9's "Required" section proposes
-- (AccessScope{level, node_id}, org-hierarchy.md:172, shipped as
-- kernel/core/src/access_scope.rs:28-40) -- recorded here so BOTH the plan's
-- ont_link form (CASE 3) and B9's property form (CASE 2) are measured.
INSERT INTO ont_instance_revisions
    (org_id, instance_id, version, attributes, valid_from, prev_hash, row_hash)
VALUES
    ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'e1000000-0000-4000-8000-000000000010', 1,
     jsonb_build_object(
        'capability', 'purchase.approve',
        'subject_party_id', '11111111-1111-4111-8111-111111111111',
        'scope_level', 'organization',
        'scope_node_id', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'),
     '2026-01-01T00:00:00Z', repeat('0', 64), repeat('a', 64)),
    ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'e1000000-0000-4000-8000-000000000011', 1,
     jsonb_build_object(
        'capability', 'purchase.approve',
        'subject_party_id', '11111111-1111-4111-8111-111111111111',
        'scope_level', 'group',
        'scope_node_id', '99999999-9999-4999-8999-999999999999'),
     '2026-01-01T00:00:00Z', repeat('0', 64), repeat('b', 64));

UPDATE ont_instances i SET current_revision_id = r.id
FROM ont_instance_revisions r WHERE r.instance_id = i.id;

-- The company-scoped grant's scope edge -- the only arm of §4.3:666 the FK can
-- express, and CASE 1's baseline.
INSERT INTO ont_links
    (org_id, link_type_id, from_instance_id, to_instance_id, valid_from)
VALUES
    ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'd1000000-0000-4000-8000-000000000001',
     'e1000000-0000-4000-8000-000000000010', 'e1000000-0000-4000-8000-000000000001',
     '2026-01-01T00:00:00Z');

-- ---------------------------------------------------------------------------
-- KNOWN-BAD CONTROL 2: a byte-identical copy of ont_instances with RLS NEVER
-- enabled, holding the same rows. Its only purpose is to prove that CONTROL 1's
-- empty result is caused by RLS and not by a query that cannot return rows at
-- all -- the failure mode that made six probes defective in one session here.
-- `LIKE ... INCLUDING ALL` copies columns, defaults, CHECKs and indexes but
-- neither the policies nor the FKs.
-- ---------------------------------------------------------------------------
CREATE TABLE x4b_instances_control_norls (LIKE ont_instances INCLUDING ALL);
INSERT INTO x4b_instances_control_norls SELECT * FROM ont_instances;
GRANT SELECT ON x4b_instances_control_norls TO console_rt;

-- Seed done: put the registry audit triggers back before anything is asserted.
ALTER TABLE ont_object_types ENABLE TRIGGER trg_ont_object_types_current_audit;
ALTER TABLE ont_link_types   ENABLE TRIGGER trg_ont_link_types_current_audit;

COMMIT;
