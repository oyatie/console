-- EXPERIMENT X4 -- FEASIBILITY PROBE. NOT PRODUCTION SCHEMA. NOT A MIGRATION.
--
-- Deliberately NOT under backend/crates/platform/db/migrations/: it must never
-- take a migration slot, and it is not intended to land. Every object is
-- prefixed x4probe_ so it cannot be confused with the real thing.
--
-- Tests docs/ideas/ecosystem-plan-DRAFT.md §4.2: tenant visibility mediated by
-- an EDGE rather than by scoping the party row, using only app.current_org.
--
-- Run by run.sh as the topology admin. The assertions run as console_rt.

BEGIN;

-- ---------------------------------------------------------------------------
-- Tier O -- platform-level identity.
-- Shape is fixed by plan §4.1:510: "(id, party_kind, status, created_at) --
-- and nothing else", and §4.1:512 "The row holds no personal data". No org_id,
-- no RLS org filter. Verified against §4.1 before writing.
-- ---------------------------------------------------------------------------
CREATE TABLE x4probe_party (
    id         UUID        PRIMARY KEY,
    party_kind TEXT        NOT NULL CHECK (party_kind IN ('NATURAL','LEGAL')),
    status     TEXT        NOT NULL CHECK (status IN ('ACTIVE','TERMINATED')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- Tier T -- the keystone edge. Column set and UNIQUE key from plan §4.1:527
-- and §4.1:547-550 (relationship_kind, valid_from/valid_to, created_by,
-- mandatory reason -- the shape clearance_assignments proves at 0147:14-32).
-- ---------------------------------------------------------------------------
CREATE TABLE x4probe_party_org_visibility (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id            UUID        NOT NULL,
    party_id          UUID        NOT NULL REFERENCES x4probe_party(id) ON DELETE RESTRICT,
    relationship_kind TEXT        NOT NULL,
    valid_from        TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to          TIMESTAMPTZ NULL,
    created_by        UUID        NULL,
    reason            TEXT        NOT NULL CHECK (char_length(reason) BETWEEN 1 AND 512),
    UNIQUE (org_id, party_id, relationship_kind, valid_from)
);

-- KNOWN-BAD CONTROL 1: byte-identical edge table, RLS never enabled.
-- Armed as org A this MUST return org B's edge. If it does not, the harness is
-- not exercising RLS at all and every result below is void.
CREATE TABLE x4probe_edge_control_norls (
    LIKE x4probe_party_org_visibility INCLUDING ALL
);

-- KNOWN-BAD CONTROL 3: correct RLS, but the UNIQUE key OMITS org_id.
-- A unique index is enforced physically, below RLS. This exists to measure --
-- not merely assert -- that org_id leading the key at §4.1:527 is load-bearing
-- for confidentiality. Armed as A, inserting a row colliding with B's invisible
-- row MUST raise 23505 and thereby leak that B holds an edge.
CREATE TABLE x4probe_edge_control_uniqleak (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id            UUID        NOT NULL,
    party_id          UUID        NOT NULL,
    relationship_kind TEXT        NOT NULL,
    valid_from        TIMESTAMPTZ NOT NULL DEFAULT now(),
    reason            TEXT        NOT NULL,
    UNIQUE (party_id, relationship_kind, valid_from)   -- org_id deliberately absent
);
ALTER TABLE x4probe_edge_control_uniqleak ENABLE ROW LEVEL SECURITY;
ALTER TABLE x4probe_edge_control_uniqleak FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON x4probe_edge_control_uniqleak
    USING (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid)
    WITH CHECK (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid);

-- ---------------------------------------------------------------------------
-- RLS armed with the standard org_isolation policy, copied verbatim from the
-- neighbouring table clearance_assignments at 0147:68-75 -- same ENABLE, same
-- FORCE, same USING, same WITH CHECK. Testing the real pattern, not an
-- invented one.
-- ---------------------------------------------------------------------------
ALTER TABLE x4probe_party_org_visibility ENABLE ROW LEVEL SECURITY;
ALTER TABLE x4probe_party_org_visibility FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON x4probe_party_org_visibility
    USING (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid)
    WITH CHECK (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid);

-- Grants mirror 0147:85-89: explicit, per-table, to console_rt.
GRANT SELECT, INSERT, UPDATE, DELETE ON x4probe_party_org_visibility  TO console_rt;
GRANT SELECT, INSERT, UPDATE, DELETE ON x4probe_edge_control_norls     TO console_rt;
GRANT SELECT, INSERT, UPDATE, DELETE ON x4probe_edge_control_uniqleak  TO console_rt;

-- VARIANT A (what the brief specifies): console_rt reads party directly and
-- reaches it only through the edge join. No definer anywhere.
GRANT SELECT ON x4probe_party TO console_rt;

-- ---------------------------------------------------------------------------
-- Seed: one human at two orgs, plus one party only org B can see.
-- The second party exists to test whether A can observe parties it holds no
-- edge to -- a cardinality question Variant A has to answer honestly.
-- ---------------------------------------------------------------------------
INSERT INTO x4probe_party (id, party_kind, status) VALUES
    ('11111111-1111-4111-8111-111111111111', 'NATURAL', 'ACTIVE'),
    ('22222222-2222-4222-8222-222222222222', 'NATURAL', 'ACTIVE');

INSERT INTO x4probe_party_org_visibility (org_id, party_id, relationship_kind, valid_from, reason) VALUES
    ('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', '11111111-1111-4111-8111-111111111111',
     'EMPLOYMENT', '2026-01-01T00:00:00Z', 'x4 probe: org A employs the party'),
    ('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', '11111111-1111-4111-8111-111111111111',
     'EMPLOYMENT', '2026-02-01T00:00:00Z', 'x4 probe: org B also employs the same party'),
    ('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', '22222222-2222-4222-8222-222222222222',
     'EMPLOYMENT', '2026-03-01T00:00:00Z', 'x4 probe: party only org B can see');

INSERT INTO x4probe_edge_control_norls
    SELECT * FROM x4probe_party_org_visibility;

INSERT INTO x4probe_edge_control_uniqleak (id, org_id, party_id, relationship_kind, valid_from, reason)
    SELECT id, org_id, party_id, relationship_kind, valid_from, reason
    FROM x4probe_party_org_visibility;

-- ---------------------------------------------------------------------------
-- VARIANT B resolver -- the plan's own §4.1:506 "definer-mediated" reading,
-- where console_rt holds NO grant on party (§4.2:630-631). Structure copied
-- from group_role_grants_for_user at 0060:99-126: SECURITY DEFINER, pinned
-- search_path, row_security off then on, EXCEPTION handler restoring it,
-- REVOKE FROM PUBLIC, GRANT EXECUTE to console_rt.
--
-- The one deliberate deviation §4.2:644-649 demands: it filters on
-- current_setting('app.current_org'), NOT on a caller-supplied parameter.
-- ---------------------------------------------------------------------------
-- rls-arming: ok x4 probe resolver, filters on app.current_org
CREATE OR REPLACE FUNCTION x4probe_resolve_correct()
RETURNS TABLE (party_id UUID, party_kind TEXT, relationship_kind TEXT)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
DECLARE
    caller_org UUID := NULLIF(current_setting('app.current_org', true), '')::uuid;
BEGIN
    IF caller_org IS NULL THEN
        RAISE EXCEPTION 'x4probe: app.current_org is not armed';
    END IF;
    SET LOCAL row_security = off;
    RETURN QUERY
        SELECT p.id, p.party_kind, v.relationship_kind
        FROM x4probe_party p
            JOIN x4probe_party_org_visibility v ON v.party_id = p.id
        WHERE v.org_id = caller_org;
    SET LOCAL row_security = on;
EXCEPTION WHEN OTHERS THEN
    SET LOCAL row_security = on;
    RAISE;
END;
$$;
REVOKE ALL ON FUNCTION x4probe_resolve_correct() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION x4probe_resolve_correct() TO console_rt;

-- KNOWN-BAD CONTROL 2: the 0060:99 shape verbatim -- trusts a caller-supplied
-- org id and never reads app.current_org. This is the failure mode §4.2:644-649
-- names as "the likely failure". Armed as A, passing B's org MUST leak B's edge.
CREATE OR REPLACE FUNCTION x4probe_resolve_leaky(p_org UUID)
RETURNS TABLE (party_id UUID, party_kind TEXT, relationship_kind TEXT)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
BEGIN
    SET LOCAL row_security = off;
    RETURN QUERY
        SELECT p.id, p.party_kind, v.relationship_kind
        FROM x4probe_party p
            JOIN x4probe_party_org_visibility v ON v.party_id = p.id
        WHERE v.org_id = p_org;
    SET LOCAL row_security = on;
EXCEPTION WHEN OTHERS THEN
    SET LOCAL row_security = on;
    RAISE;
END;
$$;
REVOKE ALL ON FUNCTION x4probe_resolve_leaky(UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION x4probe_resolve_leaky(UUID) TO console_rt;

COMMIT;
