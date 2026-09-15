WITH fixture_subjects AS (
    SELECT * FROM public.users WHERE id = ANY($1::uuid[])
), fixture_orgs AS (
    SELECT * FROM public.organizations WHERE id IN (SELECT org_id FROM fixture_subjects)
), fixture_ceremonies AS (
    SELECT * FROM public.auth_webauthn_ceremonies WHERE user_id = ANY($1::uuid[])
)
SELECT jsonb_build_object(
    'database', current_database(),
    'server_version_num', current_setting('server_version_num'),
    'migration_version', (SELECT max(version) FROM public._sqlx_migrations WHERE success),
    'migration_checksums', (SELECT jsonb_agg(jsonb_build_array(version, encode(checksum,'hex')) ORDER BY version)
        FROM public._sqlx_migrations WHERE version <= 225),
    'tables', jsonb_build_object(
        'groups', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.groups t WHERE id IN (SELECT group_id FROM fixture_orgs)),
        'organizations', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_orgs t),
        'group_memberships', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.group_id,t.org_id),'[]'::jsonb)
            FROM public.group_memberships t WHERE org_id IN (SELECT id FROM fixture_orgs)),
        'users', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_subjects t),
        'auth_webauthn_credentials', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_webauthn_credentials t WHERE user_id = ANY($1::uuid[])),
        'auth_webauthn_ceremonies', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_ceremonies t),
        'auth_webauthn_ceremony_bindings', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.ceremony_id),'[]'::jsonb)
            FROM public.auth_webauthn_ceremony_bindings t WHERE ceremony_id IN (SELECT id FROM fixture_ceremonies)),
        'auth_refresh_token_families', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_refresh_token_families t WHERE user_id = ANY($1::uuid[])),
        'auth_refresh_tokens', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_refresh_tokens t WHERE user_id = ANY($1::uuid[])),
        'audit_events', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.audit_events t WHERE id = ANY($2::uuid[]))
    ),
    'unused_fixture_edges', jsonb_build_object(
        'user_branches', (SELECT count(*) FROM public.user_branches WHERE user_id=ANY($1::uuid[])),
        'group_role_grants', (SELECT count(*) FROM public.group_role_grants WHERE user_id=ANY($1::uuid[]))
    )
)
