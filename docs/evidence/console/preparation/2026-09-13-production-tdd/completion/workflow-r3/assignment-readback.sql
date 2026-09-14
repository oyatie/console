SELECT jsonb_build_object(
  'controls', COALESCE((SELECT jsonb_agg(to_jsonb(c) ORDER BY task_id) FROM workflow_work_controls c WHERE org_id=$1), '[]'::jsonb),
  'assignments', COALESCE((SELECT jsonb_agg(to_jsonb(a) ORDER BY task_id,epoch) FROM workflow_assignment_revisions a WHERE org_id=$1), '[]'::jsonb),
  'charges', COALESCE((SELECT jsonb_agg(to_jsonb(c) ORDER BY task_id) FROM group_work_charges c WHERE group_id=$2 AND target_org_id=$1), '[]'::jsonb),
  'charge_events', COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY event_id) FROM group_work_charge_events e WHERE group_id=$2), '[]'::jsonb)
)
