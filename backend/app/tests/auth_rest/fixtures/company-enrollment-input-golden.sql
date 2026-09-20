BEGIN READ ONLY;
SET LOCAL statement_timeout='10s';
SET LOCAL lock_timeout='1s';
SET LOCAL search_path=pg_catalog;
-- Pure parser tests use synthetic bytes only, no fixture writes or product effects.
-- Run as root's reviewed administrative test connection, never claim login ACL
-- execution from this parser run. No routine installation occurs here.
DO $tests$
DECLARE
    fixture jsonb;
    actual jsonb;
    payload bytea;
    damaged bytea;
    byte_count integer;
    n integer;
    valid_count integer:=0;
    rejected_count integer:=0;
    error_message text;
BEGIN
    IF to_regprocedure('public.company_enrollment_decode_input_v1(bytea)') IS NULL THEN
        RAISE EXCEPTION 'company.parser_prerequisite_missing';
    END IF;
    FOR fixture IN SELECT value FROM jsonb_array_elements($vectors$[{"name": "ascii", "authenticated_account": "11111111-1111-4111-8111-111111111111", "input": {"command_id": "22222222-2222-4222-8222-222222222222", "group_id": null, "slug": "han", "name": "Han Company", "administrative_account_id": "33333333-3333-4333-8333-333333333333"}, "hex": "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001111111111111411181111111111111112222222222224222822222222222222200333333333333433383333333333333330000000368616e0000000b48616e20436f6d70616e79", "bytes": 100, "sha256": "50787a22cdda9189a3d680cf28b29590d27cdb5e68fa79706aace1d6ba1bb4dc"}, {"name": "korean", "authenticated_account": "11111111-1111-4111-8111-111111111111", "input": {"command_id": "22222222-2222-4222-8222-222222222222", "group_id": null, "slug": "han-seoul", "name": "한 회사", "administrative_account_id": "33333333-3333-4333-8333-333333333333"}, "hex": "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001111111111111411181111111111111112222222222224222822222222222222200333333333333433383333333333333330000000968616e2d73656f756c0000000aed959c20ed9a8cec82ac", "bytes": 105, "sha256": "ebfd1b79dd312d0ab60f77ec9e3d1653f05052deeabda79bcedb1d27e201c219"}, {"name": "escaped", "authenticated_account": "11111111-1111-4111-8111-111111111111", "input": {"command_id": "22222222-2222-4222-8222-222222222222", "group_id": null, "slug": "han-2", "name": "회사 \"한\" \\ 본사", "administrative_account_id": "33333333-3333-4333-8333-333333333333"}, "hex": "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001111111111111411181111111111111112222222222224222822222222222222200333333333333433383333333333333330000000568616e2d3200000015ed9a8cec82ac2022ed959c22205c20ebb3b8ec82ac", "bytes": 112, "sha256": "4387c7e0270437e74f501483016a191b42f258b4cdf90ce6db5f767c1dc6fef0"}, {"name": "preserved_spaces", "authenticated_account": "11111111-1111-4111-8111-111111111111", "input": {"command_id": "22222222-2222-4222-8222-222222222222", "group_id": null, "slug": "han", "name": "  한 회사  ", "administrative_account_id": "33333333-3333-4333-8333-333333333333"}, "hex": "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001111111111111411181111111111111112222222222224222822222222222222200333333333333433383333333333333330000000368616e0000000e2020ed959c20ed9a8cec82ac2020", "bytes": 103, "sha256": "5cf5b7d086ccb2001aecc7f9cf0a961c84e6082f8572d5993cba8413140aaf37"}, {"name": "nonnull_group", "authenticated_account": "11111111-1111-4111-8111-111111111111", "input": {"command_id": "22222222-2222-4222-8222-222222222222", "group_id": "44444444-4444-4444-8444-444444444444", "slug": "han", "name": "Han Company", "administrative_account_id": "33333333-3333-4333-8333-333333333333"}, "hex": "636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e7400000111111111111141118111111111111111222222222222422282222222222222220144444444444444448444444444444444333333333333433383333333333333330000000368616e0000000b48616e20436f6d70616e79", "bytes": 116, "sha256": "4947be5d786487e4560cfa0d6f2075b16290c4bacd067791b6b11470927876cb"}]$vectors$::jsonb) LOOP
        payload:=decode(fixture->>'hex','hex');
        SELECT jsonb_build_object('codec_version',d.codec_version,'account_id',d.account_id,
            'command_id',d.command_id,'group_id',d.group_id,
            'administrative_account_id',d.administrative_account_id,
            'slug',d.company_slug,'name',d.company_name,'digest',encode(d.input_digest,'hex'))
            INTO STRICT actual FROM public.company_enrollment_decode_input_v1(payload) d;
        IF actual IS DISTINCT FROM ((fixture->'input') || jsonb_build_object(
            'codec_version',1,'account_id',fixture->>'authenticated_account','digest',fixture->>'sha256')) THEN
            RAISE EXCEPTION 'company.parser_golden_mismatch: %',fixture->>'name';
        END IF;
        valid_count:=valid_count+1;
        byte_count:=octet_length(payload);
        FOR n IN 0..byte_count-1 LOOP
            BEGIN
                PERFORM * FROM public.company_enrollment_decode_input_v1(substring(payload FROM 1 FOR n));
                RAISE EXCEPTION 'company.parser_accepted_truncation: %/%',fixture->>'name',n;
            EXCEPTION WHEN SQLSTATE '22023' THEN
                GET STACKED DIAGNOSTICS error_message=MESSAGE_TEXT;
                IF error_message<>'company_enrollment.invalid_input' THEN RAISE; END IF;
                rejected_count:=rejected_count+1;
            END;
        END LOOP;
        FOREACH damaged IN ARRAY ARRAY[payload||decode('00','hex'),payload||decode('000102','hex'),payload||decode('20','hex')] LOOP
            BEGIN
                PERFORM * FROM public.company_enrollment_decode_input_v1(damaged);
                RAISE EXCEPTION 'company.parser_accepted_suffix';
            EXCEPTION WHEN SQLSTATE '22023' THEN
                GET STACKED DIAGNOSTICS error_message=MESSAGE_TEXT;
                IF error_message<>'company_enrollment.invalid_input' THEN RAISE; END IF;
                rejected_count:=rejected_count+1;
            END;
        END LOOP;
    END LOOP;
    IF valid_count<>5 THEN RAISE EXCEPTION 'company.parser_wrong_vector_count'; END IF;
    BEGIN
        PERFORM * FROM public.company_enrollment_decode_input_v1(NULL::bytea);
        RAISE EXCEPTION 'company.parser_accepted_null_or_returned_no_row';
    EXCEPTION WHEN SQLSTATE '22023' THEN
        GET STACKED DIAGNOSTICS error_message=MESSAGE_TEXT;
        IF error_message<>'company_enrollment.invalid_input' THEN RAISE; END IF;
        rejected_count:=rejected_count+1;
    END;
    IF rejected_count<>552 THEN RAISE EXCEPTION 'company.parser_wrong_rejected_case_count'; END IF;
    RAISE NOTICE 'company.parser valid_vectors=% rejected_prefix_suffix_null_cases=%',valid_count,rejected_count;
END
$tests$;
ROLLBACK;
