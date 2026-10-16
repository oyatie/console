def company_eligibility_snapshot_query():
    # Extend the unchanged Business-session serializer; historical bytes and
    # fingerprints stay valid only for the historical contract they describe.
    query = business_session_snapshot_query()
    replacements = [
        ("OR (n.nspname='public' AND p.proname IN (", "OR (n.nspname='public' AND p.proname IN ('account_company_setup_eligibility_v1',"),
        ("   ('public.account_context_presence_v1(uuid)'),", "   ('public.account_company_setup_eligibility_v1(uuid)'),\n   ('public.account_context_presence_v1(uuid)'),"),
        ("count(*)=59 AND bool_and(present", "count(*)=60 AND bool_and(present"),
    ]
    for before, after in replacements:
        if query.count(before) != 1:
            raise ValueError('Company eligibility snapshot anchor missing or duplicate')
        query = query.replace(before, after)
    return query
