-- Exact0166 owner body plus native-target refusals before both replay shortcuts.
-- Preserve console_leave_definer ownership, ACL, search_path and original byte semantics.
CREATE OR REPLACE FUNCTION leave_api.apply_employee_import_batch(
    p_org_id UUID, p_run_id UUID, p_source_ref TEXT, p_rows JSONB,
    p_actor UUID, p_apply_audit JSONB, p_trace_id TEXT, p_span_id TEXT
) RETURNS TABLE(report JSONB, replayed BOOLEAN)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog SET row_security = on AS $$
DECLARE
    v_run public.data_import_runs%ROWTYPE;
    v_row JSONB;
    v_employee public.employees%ROWTYPE;
    v_employee_id UUID;
    v_employee_updated_at TIMESTAMPTZ;
    v_balance_result RECORD;
    v_source_key TEXT;
    v_company TEXT;
    v_name TEXT;
    v_idempotency_key TEXT;
    v_identity_strategy TEXT;
    v_identity_confidence TEXT;
    v_identity_review_required BOOLEAN;
    v_outcome TEXT;
    v_outcomes JSONB := '[]'::JSONB;
    v_report JSONB;
    v_input_rows INTEGER;
BEGIN
    PERFORM leave_api.assert_context(p_org_id,p_actor,p_trace_id,p_span_id);
    PERFORM leave_api.assert_employee_importer(p_org_id,p_actor);
    IF char_length(pg_catalog.btrim(p_source_ref)) NOT BETWEEN 1 AND 256
       OR jsonb_typeof(p_rows) IS DISTINCT FROM 'array' THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='employee_import_batch.invalid_payload';
    END IF;
    v_input_rows := jsonb_array_length(p_rows);

    -- APPLIED-run replay below skips per-row processing; current native targets
    -- still refuse rather than returning a success-shaped import summary.
    IF EXISTS (
        SELECT 1 FROM jsonb_array_elements(p_rows) item
        JOIN public.employees e ON e.org_id=p_org_id
          AND e.source_key=pg_catalog.btrim(item->>'source_key')
        WHERE e.source_kind='NATIVE_DIRECTORY'
    ) THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='employee_import_batch.native_directory_target';
    END IF;

    IF p_run_id IS NOT NULL THEN
        SELECT * INTO v_run
          FROM public.data_import_runs r
         WHERE r.org_id=p_org_id AND r.id=p_run_id
         FOR UPDATE;
        IF NOT FOUND OR v_run.entity_type <> 'employee_hr' THEN
            RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='employee_import_batch.run_not_found';
        END IF;
        IF v_run.status = 'APPLIED' THEN
            SELECT v_run.apply_summary || pg_catalog.jsonb_build_object(
                'inserted',0,
                'updated',0,
                'skipped',coalesce((v_run.apply_summary->>'input_rows')::INTEGER,0),
                'companies',coalesce((
                    SELECT jsonb_agg(item || pg_catalog.jsonb_build_object(
                        'inserted',0,
                        'updated',0,
                        'skipped',coalesce((item->>'input_rows')::INTEGER,0)
                    ) ORDER BY item->>'company')
                    FROM jsonb_array_elements(
                        coalesce(v_run.apply_summary->'companies','[]'::JSONB)
                    ) item
                ),'[]'::JSONB)
            ) INTO v_report;
            RETURN QUERY SELECT v_report, true;
            RETURN;
        END IF;
        IF v_run.status <> 'DRY_RUN' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='employee_import_batch.run_not_dry_run';
        END IF;
        IF p_source_ref <> 'run:' || p_run_id::TEXT
           OR v_input_rows <> v_run.candidate_rows
           OR (SELECT count(DISTINCT item->>'source_key')
                 FROM jsonb_array_elements(p_rows) item) <> v_run.candidate_rows
           OR EXISTS (
                SELECT 1
                  FROM jsonb_array_elements(p_rows) item
                 WHERE NOT EXISTS (
                    SELECT 1 FROM public.data_import_rows ir
                     WHERE ir.org_id=p_org_id AND ir.run_id=p_run_id
                       AND ir.row_status='CANDIDATE'
                       AND ir.source_key=item->>'source_key'
                 )
           )
           OR EXISTS (
                SELECT 1
                  FROM jsonb_array_elements(p_rows) item
                  JOIN public.data_import_rows ir
                    ON ir.org_id=p_org_id
                   AND ir.run_id=p_run_id
                   AND ir.row_status='CANDIDATE'
                   AND ir.source_key=item->>'source_key'
                 CROSS JOIN LATERAL (
                    SELECT CASE
                        WHEN ir.canonical_row#>>'{source_metadata,identity_resolution,strategy}'
                             IN ('employee_number','legal_identifier_hash',
                                 'birth_hire_fingerprint','source_row_fingerprint')
                        THEN ir.canonical_row#>>'{source_metadata,identity_resolution,strategy}'
                        ELSE 'source_row_fingerprint'
                    END AS strategy
                 ) identity
                 WHERE item IS DISTINCT FROM (
                    ir.canonical_row || pg_catalog.jsonb_build_object(
                        'raw_row',ir.raw_row,
                        'identity',pg_catalog.jsonb_build_object(
                            'strategy',identity.strategy,
                            'confidence',CASE identity.strategy
                                WHEN 'employee_number' THEN 'high'
                                WHEN 'legal_identifier_hash' THEN 'high'
                                WHEN 'birth_hire_fingerprint' THEN 'medium'
                                ELSE 'low'
                            END,
                            'review_required',NOT (
                                coalesce((ir.canonical_row#>>
                                    '{source_metadata,identity_resolution,manual_review_required}')
                                    ::BOOLEAN,true) = false
                                AND identity.strategy IN (
                                    'employee_number','legal_identifier_hash'
                                )
                            )
                        )
                    )
                 )
           ) THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='employee_import_batch.run_payload_mismatch';
        END IF;
    END IF;

    FOR v_row IN SELECT value FROM jsonb_array_elements(p_rows)
    LOOP
        v_company := pg_catalog.btrim(v_row->>'company');
        v_name := pg_catalog.btrim(v_row->>'name');
        v_source_key := pg_catalog.btrim(v_row->>'source_key');
        IF coalesce(v_company,'') = '' OR coalesce(v_name,'') = ''
           OR coalesce(v_source_key,'') = ''
           OR coalesce(pg_catalog.btrim(v_row->>'source_filename'),'') = ''
           OR coalesce(pg_catalog.btrim(v_row->>'source_sheet'),'') = ''
           OR coalesce((v_row->>'source_row')::INTEGER,0) <= 0
           OR jsonb_typeof(v_row->'raw_row') IS DISTINCT FROM 'object'
           OR jsonb_typeof(v_row->'source_metadata') IS DISTINCT FROM 'object'
           OR jsonb_typeof(v_row->'canonical') IS DISTINCT FROM 'object' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='employee_import_batch.invalid_row';
        END IF;

        v_idempotency_key := 'employee-import:' || pg_catalog.encode(
            public.digest(p_source_ref || ':' || v_source_key,'sha256'),'hex'
        );
        SELECT * INTO v_employee
          FROM public.employees e
         WHERE e.org_id=p_org_id AND e.source_key=v_source_key
         FOR UPDATE;

        -- Native directory has no import or leave provenance. Refuse BEFORE the
        -- existing receipt replay branch, including a no-employee-DML replay.
        IF FOUND AND v_employee.source_kind='NATIVE_DIRECTORY' THEN
            RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='employee_import_batch.native_directory_target';
        END IF;

        -- A prior receipt means this exact source row already committed. Call
        -- the protected command to verify its payload binding, but deliberately
        -- do not rewrite ordinary roster fields or updated_at on replay.
        IF FOUND AND EXISTS (
            SELECT 1 FROM public.leave_balance_import_receipts r
             WHERE r.org_id=p_org_id AND r.idempotency_key=v_idempotency_key
        ) THEN
            SELECT * INTO v_balance_result
              FROM leave_api.import_employee_leave_balance(
                p_org_id,v_employee.id,v_employee.updated_at,
                v_row#>>'{canonical,leave_accrued}',
                v_row#>>'{canonical,leave_used}',
                v_row#>>'{canonical,leave_remaining}',
                'employee_import',p_source_ref,v_idempotency_key,
                p_actor,p_trace_id,p_span_id
              );
            v_outcome := 'skipped';
        ELSE
            v_identity_strategy := coalesce(
                v_row#>>'{identity,strategy}','source_row_fingerprint'
            );
            v_identity_confidence := coalesce(v_row#>>'{identity,confidence}','low');
            v_identity_review_required := coalesce(
                (v_row#>>'{identity,review_required}')::BOOLEAN,true
            );
            INSERT INTO public.employees (
                org_id,company,name,source_filename,source_sheet,source_row,
                source_key,raw_row,source_metadata,employee_number,org_unit,job,
                position,worksite_name,worksite_address,hire_date,exit_date,
                employment_status,identity_resolution_strategy,
                identity_resolution_confidence,identity_review_required,
                identity_name_only_merge
            ) VALUES (
                p_org_id,v_company,v_name,v_row->>'source_filename',
                v_row->>'source_sheet',(v_row->>'source_row')::INTEGER,
                v_source_key,v_row->'raw_row',v_row->'source_metadata',
                v_row#>>'{canonical,employee_number}',
                v_row#>>'{canonical,org_unit}',v_row#>>'{canonical,job}',
                v_row#>>'{canonical,position}',v_row#>>'{canonical,worksite_name}',
                v_row#>>'{canonical,worksite_address}',v_row#>>'{canonical,hire_date}',
                v_row#>>'{canonical,exit_date}',
                coalesce(v_row#>>'{canonical,employment_status}','ACTIVE'),
                v_identity_strategy,v_identity_confidence,
                v_identity_review_required,false
            )
            ON CONFLICT (org_id,source_key) DO UPDATE SET
                company=EXCLUDED.company,name=EXCLUDED.name,
                source_filename=EXCLUDED.source_filename,
                source_sheet=EXCLUDED.source_sheet,source_row=EXCLUDED.source_row,
                raw_row=EXCLUDED.raw_row,source_metadata=EXCLUDED.source_metadata,
                employee_number=EXCLUDED.employee_number,org_unit=EXCLUDED.org_unit,
                job=EXCLUDED.job,position=EXCLUDED.position,
                worksite_name=EXCLUDED.worksite_name,
                worksite_address=EXCLUDED.worksite_address,
                hire_date=EXCLUDED.hire_date,exit_date=EXCLUDED.exit_date,
                employment_status=EXCLUDED.employment_status,
                identity_resolution_strategy=EXCLUDED.identity_resolution_strategy,
                identity_resolution_confidence=EXCLUDED.identity_resolution_confidence,
                identity_review_required=EXCLUDED.identity_review_required,
                identity_name_only_merge=false,updated_at=pg_catalog.clock_timestamp()
            RETURNING id,updated_at,
                CASE WHEN xmax=0 THEN 'inserted' ELSE 'updated' END
              INTO v_employee_id,v_employee_updated_at,v_outcome;

            SELECT * INTO v_balance_result
              FROM leave_api.import_employee_leave_balance(
                p_org_id,v_employee_id,v_employee_updated_at,
                v_row#>>'{canonical,leave_accrued}',
                v_row#>>'{canonical,leave_used}',
                v_row#>>'{canonical,leave_remaining}',
                'employee_import',p_source_ref,v_idempotency_key,
                p_actor,p_trace_id,p_span_id
              );
        END IF;
        v_outcomes := v_outcomes || pg_catalog.jsonb_build_array(
            pg_catalog.jsonb_build_object('company',v_company,'outcome',v_outcome)
        );
    END LOOP;

    SELECT pg_catalog.jsonb_build_object(
        'input_rows',v_input_rows,
        'inserted',(SELECT count(*) FROM jsonb_array_elements(v_outcomes) item
                    WHERE item->>'outcome'='inserted'),
        'updated',(SELECT count(*) FROM jsonb_array_elements(v_outcomes) item
                   WHERE item->>'outcome'='updated'),
        'skipped',(SELECT count(*) FROM jsonb_array_elements(v_outcomes) item
                   WHERE item->>'outcome'='skipped'),
        'companies',coalesce((
            SELECT jsonb_agg(pg_catalog.jsonb_build_object(
                'company',company,'input_rows',input_rows,
                'inserted',inserted,'updated',updated,'skipped',skipped
            ) ORDER BY company)
            FROM (
                SELECT item->>'company' AS company,count(*) AS input_rows,
                    count(*) FILTER (WHERE item->>'outcome'='inserted') AS inserted,
                    count(*) FILTER (WHERE item->>'outcome'='updated') AS updated,
                    count(*) FILTER (WHERE item->>'outcome'='skipped') AS skipped
                FROM jsonb_array_elements(v_outcomes) item
                GROUP BY item->>'company'
            ) companies
        ),'[]'::JSONB)
    ) INTO v_report;

    IF p_run_id IS NOT NULL THEN
        UPDATE public.data_import_runs r
           SET status='APPLIED',apply_summary=v_report,applied_by=p_actor,
               applied_at=pg_catalog.clock_timestamp(),updated_at=pg_catalog.clock_timestamp()
         WHERE r.org_id=p_org_id AND r.id=p_run_id;
    END IF;
    -- A successful first application always has intrinsic command-owned
    -- evidence, including legacy direct imports and roster-only/null-balance
    -- rows. Pure receipt/APPLIED replay is side-effect-free and reuses the
    -- original evidence rather than appending a misleading duplicate.
    IF p_run_id IS NOT NULL OR EXISTS (
        SELECT 1 FROM jsonb_array_elements(v_outcomes) item
        WHERE item->>'outcome' <> 'skipped'
    ) THEN
        INSERT INTO public.audit_events
            (actor,action,target_type,target_id,before_snap,after_snap,
             trace_id,span_id,occurred_at,org_id)
        VALUES (
            p_actor,'data_import.apply',
            CASE WHEN p_run_id IS NULL THEN 'employee_import_batch' ELSE 'data_import_run' END,
            coalesce(p_run_id::TEXT,pg_catalog.btrim(p_source_ref)),NULL,
            coalesce(p_apply_audit,'{}'::JSONB) || pg_catalog.jsonb_build_object(
                'run_id',p_run_id,'entity_type','employee_hr',
                'source_ref',pg_catalog.btrim(p_source_ref),'report',v_report
            ),p_trace_id,p_span_id,pg_catalog.statement_timestamp(),p_org_id
        );
    END IF;
    RETURN QUERY SELECT v_report,false;
END;
$$;
