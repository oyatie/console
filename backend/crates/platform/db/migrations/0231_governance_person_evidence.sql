-- Immutable request-opening and decision-time Person evidence. Historical rows
-- remain NULL (unattested); current mutable User links cannot backfill history.
-- No Person FK is added: retained audit identities must not add an erasure
-- barrier. Existing Company RLS and append-only triggers remain unchanged.
ALTER TABLE public.gov_approval_requests ADD COLUMN requester_person_id UUID;
ALTER TABLE public.gov_approvals ADD COLUMN approver_person_id UUID;

COMMENT ON COLUMN public.gov_approval_requests.requester_person_id IS
    'pd:personal — Canonical requester Person pinned when the request opened; NULL historical rows are unattested';
COMMENT ON COLUMN public.gov_approvals.approver_person_id IS
    'pd:personal — Canonical approver Person pinned when the decision committed; NULL historical rows are unattested';
