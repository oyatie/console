-- Corrective custody source: apply only after exact Directory v1 verification.
-- PostgreSQL RI checks lock the accepted input FOR KEY SHARE as its owner.
-- One column UPDATE privilege permits that lock; the existing ALWAYS statement
-- trigger still rejects every UPDATE, DELETE and TRUNCATE, including zero rows.
-- No runtime grant, table-wide UPDATE, historical rewrite or new writer.
GRANT UPDATE(command_id) ON public.native_people_inputs_v1 TO console_account_owner;
