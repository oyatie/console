ALTER TABLE public.accounts OWNER TO console_account_owner;
ALTER TABLE public.account_security OWNER TO console_account_owner;
ALTER TABLE public.account_security_events OWNER TO console_account_owner;
SELECT 1/0;
ALTER TABLE public.account_terms_acceptances OWNER TO console_account_owner;
ALTER TABLE public.account_terms_head OWNER TO console_terms_owner;
ALTER TABLE public.account_terms_release_receipts OWNER TO console_terms_owner;
