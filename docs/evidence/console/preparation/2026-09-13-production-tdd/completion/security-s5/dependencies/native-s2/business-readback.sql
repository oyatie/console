-- Privileged READ ONLY test readback after actual owner invocations. Never fixture INSERT.
-- Tables/columns are exact native18 physical-contract.sql.txt. Native migration is a prerequisite.
SELECT jsonb_build_object(
  'run',(SELECT to_jsonb(r) FROM public.payroll_draft_runs r WHERE r.org_id=$1 AND r.id=$2),
  'lines',(SELECT coalesce(jsonb_agg(to_jsonb(l) ORDER BY l.id),'[]'::jsonb) FROM public.payroll_draft_lines l WHERE l.org_id=$1 AND l.run_id=$2),
  'inputs',(SELECT coalesce(jsonb_agg(to_jsonb(i) ORDER BY i.input_revision),'[]'::jsonb) FROM public.payroll_input_revisions i WHERE i.org_id=$1 AND i.run_id=$2),
  'units',(SELECT coalesce(jsonb_agg(to_jsonb(u) ORDER BY u.input_revision,u.employee_id),'[]'::jsonb) FROM public.payroll_input_units u WHERE u.org_id=$1 AND u.run_id=$2),
  'sources',(SELECT coalesce(jsonb_agg(to_jsonb(s) ORDER BY s.input_revision,s.employee_id,s.source_kind,s.selection,s.ordinal),'[]'::jsonb) FROM public.payroll_input_source_refs s WHERE s.org_id=$1 AND s.run_id=$2),
  'batches',(SELECT coalesce(jsonb_agg(to_jsonb(b) ORDER BY b.calculation_revision),'[]'::jsonb) FROM public.payroll_calculation_batches b WHERE b.org_id=$1 AND b.run_id=$2),
  'outcomes',(SELECT coalesce(jsonb_agg(to_jsonb(o) ORDER BY o.calculation_revision,o.employee_id),'[]'::jsonb) FROM public.payroll_calculation_units o WHERE o.org_id=$1 AND o.run_id=$2),
  'money',(SELECT coalesce(jsonb_agg(to_jsonb(m) ORDER BY m.id),'[]'::jsonb) FROM public.payroll_line_calculations m WHERE m.org_id=$1 AND m.run_id=$2)
)
