-- Independent privileged read-only observation in one actual Company.
-- Exact design30 storage-mapping columns; no fabricated product rows.
SELECT jsonb_build_object(
 'publications',(SELECT coalesce(jsonb_agg(to_jsonb(p) ORDER BY p.publication_id),'[]'::jsonb)
   FROM public.payroll_review_publications p WHERE p.org_id=$1),
 'slots',(SELECT coalesce(jsonb_agg(to_jsonb(s) ORDER BY s.subject_key::text,s.lineage_key::text,s.purpose),'[]'::jsonb)
   FROM public.payroll_publication_slots s WHERE s.org_id=$1),
 'controls',(SELECT coalesce(jsonb_agg(to_jsonb(c) ORDER BY c.publication_id),'[]'::jsonb)
   FROM public.payroll_publication_controls c WHERE c.org_id=$1),
 'events',(SELECT coalesce(jsonb_agg(to_jsonb(e) ORDER BY e.event_id),'[]'::jsonb)
   FROM public.payroll_publication_events e WHERE e.org_id=$1),
 'original_receipts',(SELECT coalesce(jsonb_agg(to_jsonb(r) ORDER BY r.command_id),'[]'::jsonb)
   FROM public.ont_action_command_receipts r WHERE r.org_id=$1 AND r.command_id=ANY($2))
)
