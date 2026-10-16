-- PostgreSQL requires UPDATE on at least one column for SELECT ... FOR SHARE.
-- Payroll review retains the actual binding rows against DELETE/reinsert until
-- commit. Migration 0213's immutable BEFORE UPDATE trigger still rejects every
-- affected-row UPDATE; neither its definition nor the forced RLS policy changes.
-- Existing readers remain compatible. Revoke this column privilege only after
-- disabling readers that require these locks; fence approvals before rollback
-- to readers without natural-person independence checks.
GRANT UPDATE (employee_id) ON public.employee_person_bindings TO console_rt;
