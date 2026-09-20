// Include inside intake_owner. Actual SQLx source; guard bodies remain root-owned.
mod integrity_guards {
    use super::*;
    const GUARDS: &str = include_str!("../../../../ops/postgres-company-enrollment-guards.sql");

    async fn install_guards(pool: &PgPool) {
        let mut connection = pool.acquire().await.unwrap();
        sqlx::raw_sql("BEGIN")
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::raw_sql(GUARDS)
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::raw_sql("COMMIT")
            .execute(&mut *connection)
            .await
            .unwrap();
        profiles(
            &mut connection,
            "account_native.profile_mismatch",
            "account_native.profile_mismatch",
        )
        .await;
    }

    async fn owner_tx(pool: &PgPool) -> sqlx::Transaction<'_, sqlx::Postgres> {
        let mut tx = pool.begin().await.unwrap();
        // Deliberate no-login owner test, never serving-login evidence.
        sqlx::raw_sql("SET LOCAL ROLE console_account_owner; SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='5s'").execute(tx.as_mut()).await.unwrap();
        let actual: String = sqlx::query_scalar("SELECT current_user::text")
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
        assert_eq!(actual, "console_account_owner");
        tx
    }

    async fn catalog(pool: &PgPool) -> bool {
        let triggers:Vec<(String,String,i16,String,bool,bool,String,bool)>=sqlx::query_as("SELECT c.relname::text,t.tgname::text,t.tgtype::smallint,t.tgenabled::text,t.tgdeferrable,t.tginitdeferred,p.proname::text,(p.pronamespace='public'::regnamespace AND t.tgqual IS NULL AND t.tgnargs=0 AND t.tgattr::text='' AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL AND t.tgparentid=0) FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_proc p ON p.oid=t.tgfoid WHERE c.relnamespace='public'::regnamespace AND c.relname IN ('company_enrollment_requests','company_enrollment_request_events','company_enrollment_receipts') AND NOT t.tgisinternal ORDER BY t.tgname")
            .fetch_all(pool).await.unwrap();
        let mut expected = vec![
            (
                "company_enrollment_requests",
                "company_enrollment_request_guard_v1",
                23,
                false,
                "company_enrollment_request_guard_v1",
            ),
            (
                "company_enrollment_requests",
                "company_enrollment_request_preserve_v1",
                42,
                false,
                "company_enrollment_request_preserve_v1",
            ),
            (
                "company_enrollment_request_events",
                "company_enrollment_event_immutable_v1",
                58,
                false,
                "company_enrollment_event_immutable_v1",
            ),
            (
                "company_enrollment_request_events",
                "company_enrollment_event_guard_v1",
                5,
                false,
                "company_enrollment_event_guard_v1",
            ),
            (
                "company_enrollment_receipts",
                "company_enrollment_receipt_intake_guard_v1",
                6,
                false,
                "company_enrollment_receipt_intake_guard_v1",
            ),
            (
                "company_enrollment_requests",
                "company_enrollment_request_closure_v1",
                21,
                true,
                "company_enrollment_intake_closure_v1",
            ),
            (
                "company_enrollment_request_events",
                "company_enrollment_event_closure_v1",
                5,
                true,
                "company_enrollment_intake_closure_v1",
            ),
            (
                "company_enrollment_receipts",
                "company_enrollment_receipt_closure_v1",
                5,
                true,
                "company_enrollment_intake_closure_v1",
            ),
            (
                "company_enrollment_receipts",
                "company_enrollment_receipts_immutable_v1",
                58,
                false,
                "company_enrollment_receipts_immutable_v1",
            ),
        ];
        expected.sort_by_key(|row| row.1);
        let expected: Vec<_> = expected
            .into_iter()
            .map(|(table, name, bits, deferred, function)| {
                (
                    table.to_owned(),
                    name.to_owned(),
                    bits,
                    "A".to_owned(),
                    deferred,
                    deferred,
                    function.to_owned(),
                    true,
                )
            })
            .collect();
        let triggers_match = triggers == expected;
        let functions:Vec<(String,bool,bool)>=sqlx::query_as("SELECT p.proname::text,p.prosecdef,(p.proowner='console_account_owner'::regrole AND l.lanname='plpgsql' AND p.prokind='f' AND p.provolatile='v' AND p.proparallel='u' AND NOT p.proleakproof AND NOT p.proisstrict AND p.pronargs=0 AND p.pronargdefaults=0 AND p.prorettype='trigger'::regtype AND NOT p.proretset AND p.prosupport=0 AND p.proconfig=CASE WHEN p.proname='company_enrollment_receipts_immutable_v1' THEN ARRAY['search_path=pg_catalog, pg_temp'] ELSE ARRAY['search_path=pg_catalog, pg_temp','row_security=on'] END AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))))) FROM pg_proc p JOIN pg_language l ON l.oid=p.prolang WHERE p.pronamespace='public'::regnamespace AND p.proname IN ('company_enrollment_event_guard_v1','company_enrollment_event_immutable_v1','company_enrollment_intake_closure_v1','company_enrollment_receipt_intake_guard_v1','company_enrollment_receipts_immutable_v1','company_enrollment_request_guard_v1','company_enrollment_request_preserve_v1') ORDER BY p.proname, p.oid")
            .fetch_all(pool).await.unwrap();
        let expected: Vec<_> = [
            "company_enrollment_event_guard_v1",
            "company_enrollment_event_immutable_v1",
            "company_enrollment_intake_closure_v1",
            "company_enrollment_receipt_intake_guard_v1",
            "company_enrollment_receipts_immutable_v1",
            "company_enrollment_request_guard_v1",
            "company_enrollment_request_preserve_v1",
        ]
        .into_iter()
        .map(|name| {
            (
                name.to_owned(),
                name == "company_enrollment_intake_closure_v1",
                true,
            )
        })
        .collect();
        triggers_match && functions == expected
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_guards_catalog_and_real_prepare_cancel_remain_valid(pool: PgPool) {
        let (_app, account, _cookies, startup, designation) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        let before = all_rows(&pool).await;
        install_guards(&pool).await;
        assert!(catalog(&pool).await);
        assert!(before == all_rows(&pool).await);
        for (corrupt, restore) in [
            (
                "ALTER TABLE public.company_enrollment_request_events ENABLE TRIGGER company_enrollment_event_immutable_v1",
                "ALTER TABLE public.company_enrollment_request_events ENABLE ALWAYS TRIGGER company_enrollment_event_immutable_v1",
            ),
            (
                "ALTER FUNCTION public.company_enrollment_intake_closure_v1() SECURITY INVOKER",
                "ALTER FUNCTION public.company_enrollment_intake_closure_v1() SECURITY DEFINER",
            ),
            (
                "CREATE FUNCTION public.company_enrollment_intake_closure_v1(integer) RETURNS boolean LANGUAGE sql AS 'SELECT true'",
                "DROP FUNCTION public.company_enrollment_intake_closure_v1(integer)",
            ),
            (
                "CREATE SCHEMA company_guard_corruption; CREATE FUNCTION company_guard_corruption.company_enrollment_event_immutable_v1() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'CORRUPTION_ONLY'; END $$; DROP TRIGGER company_enrollment_event_immutable_v1 ON public.company_enrollment_request_events; CREATE TRIGGER company_enrollment_event_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.company_enrollment_request_events FOR EACH STATEMENT EXECUTE FUNCTION company_guard_corruption.company_enrollment_event_immutable_v1(); ALTER TABLE public.company_enrollment_request_events ENABLE ALWAYS TRIGGER company_enrollment_event_immutable_v1",
                "DROP TRIGGER company_enrollment_event_immutable_v1 ON public.company_enrollment_request_events; CREATE TRIGGER company_enrollment_event_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.company_enrollment_request_events FOR EACH STATEMENT EXECUTE FUNCTION public.company_enrollment_event_immutable_v1(); ALTER TABLE public.company_enrollment_request_events ENABLE ALWAYS TRIGGER company_enrollment_event_immutable_v1; DROP FUNCTION company_guard_corruption.company_enrollment_event_immutable_v1(); DROP SCHEMA company_guard_corruption",
            ),
        ] {
            sqlx::raw_sql(corrupt).execute(&pool).await.unwrap();
            let accepted = catalog(&pool).await;
            sqlx::raw_sql(restore).execute(&pool).await.unwrap();
            assert!(
                !accepted,
                "catalog oracle accepted deliberately weakened guard"
            );
            assert!(catalog(&pool).await);
        }
        let command = Uuid::new_v4();
        let input = bytes(account.account, command, account.account);
        let lower = now(&pool).await;
        assert_eq!(
            prepare(&business, account.account, session, command, &input)
                .await
                .unwrap(),
            ("PENDING".into(), None)
        );
        let upper = now(&pool).await;
        pending(
            &pool,
            account.account,
            session,
            command,
            &input,
            designation.command,
            lower,
            upper,
        )
        .await;
        let prepared = all_rows(&pool).await;
        unrelated_unchanged(&before, &prepared);
        for table in [
            "company_enrollment_requests",
            "company_enrollment_request_events",
        ] {
            assert_eq!(
                added_rows(&before[table], &prepared[table]).unwrap().len(),
                1
            );
        }
        let cancelled = cancel_checked(
            &pool,
            &business,
            account.account,
            session,
            command,
            &prepared,
        )
        .await;
        assert_eq!(
            status(&business, account.account, session, command)
                .await
                .unwrap(),
            Some(("CANCELLED".into(), None))
        );
        assert!(cancelled == all_rows(&pool).await);
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_guards_owner_rewrites_and_partial_transition_fail(pool: PgPool) {
        let (_app, account, _cookies, startup, _) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        install_guards(&pool).await;
        let command = Uuid::new_v4();
        let input = bytes(account.account, command, account.account);
        prepare(&business, account.account, session, command, &input)
            .await
            .unwrap();
        let before = all_rows(&pool).await;
        for statement in [
            "UPDATE public.company_enrollment_requests SET input_bytes=NULL WHERE account_id=$1 AND command_id=$2",
            "UPDATE public.company_enrollment_requests SET state='EXPIRED',input_bytes=NULL,terminal_at=clock_timestamp() WHERE account_id=$1 AND command_id=$2",
            "UPDATE public.company_enrollment_requests SET state=state WHERE account_id=$1 AND command_id=$2",
        ] {
            let mut tx = owner_tx(&pool).await;
            let result = sqlx::query(statement)
                .bind(account.account)
                .bind(command)
                .execute(tx.as_mut())
                .await;
            refused(
                result,
                "P0001",
                Some("company_enrollment.request_transition_invalid"),
            );
            tx.rollback().await.unwrap();
            assert!(before == all_rows(&pool).await);
        }
        let mut tx = owner_tx(&pool).await;
        let result=sqlx::query("UPDATE public.company_enrollment_requests SET state='COMMITTED',terminal_at=clock_timestamp(),committed_receipt_id=$3 WHERE account_id=$1 AND command_id=$2").bind(account.account).bind(command).bind(Uuid::new_v4()).execute(tx.as_mut()).await;
        refused(
            result,
            "P0001",
            Some("company_enrollment.effect_unavailable"),
        );
        tx.rollback().await.unwrap();
        assert!(before == all_rows(&pool).await);
        let mut tx = owner_tx(&pool).await;
        let changed=sqlx::query("UPDATE public.company_enrollment_requests SET state='CANCELLED',terminal_at=clock_timestamp() WHERE account_id=$1 AND command_id=$2").bind(account.account).bind(command).execute(tx.as_mut()).await.unwrap();
        assert_eq!(changed.rows_affected(), 1);
        refused(
            sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
                .execute(tx.as_mut())
                .await,
            "P0001",
            Some("company_enrollment.intake_closure_invalid"),
        );
        tx.rollback().await.unwrap();
        assert!(before == all_rows(&pool).await);
        // Temporarily broaden only the no-login owner's privileges inside each
        // rollback, to prove defense beyond the retained narrow ACLs.
        for (statement, expected) in [
            (
                "UPDATE public.company_enrollment_requests SET account_id='00000000-0000-0000-0000-000000000000'",
                "company_enrollment.request_immutable",
            ),
            (
                "UPDATE public.company_enrollment_requests SET command_id='00000000-0000-0000-0000-000000000000'",
                "company_enrollment.request_immutable",
            ),
            (
                "UPDATE public.company_enrollment_requests SET codec_version=2",
                "company_enrollment.request_immutable",
            ),
            (
                "UPDATE public.company_enrollment_requests SET input_digest=decode(repeat('00',32),'hex')",
                "company_enrollment.request_immutable",
            ),
            (
                "UPDATE public.company_enrollment_requests SET designation_receipt_id='00000000-0000-0000-0000-000000000000'",
                "company_enrollment.request_immutable",
            ),
            (
                "UPDATE public.company_enrollment_requests SET created_at=created_at+interval '1 microsecond'",
                "company_enrollment.request_immutable",
            ),
            (
                "UPDATE public.company_enrollment_requests SET expires_at=expires_at+interval '1 microsecond'",
                "company_enrollment.request_immutable",
            ),
            (
                "DELETE FROM public.company_enrollment_requests WHERE false",
                "company_enrollment.request_immutable",
            ),
            (
                "TRUNCATE public.company_enrollment_requests CASCADE",
                "company_enrollment.request_immutable",
            ),
            (
                "UPDATE public.company_enrollment_request_events SET reason_code=reason_code WHERE false",
                "company_enrollment.event_immutable",
            ),
            (
                "DELETE FROM public.company_enrollment_request_events WHERE false",
                "company_enrollment.event_immutable",
            ),
            (
                "TRUNCATE public.company_enrollment_request_events",
                "company_enrollment.event_immutable",
            ),
            (
                "INSERT INTO public.company_enrollment_receipts DEFAULT VALUES",
                "company_enrollment.effect_unavailable",
            ),
        ] {
            let acl_before:Vec<(String,Option<String>)>=sqlx::query_as("SELECT relname::text,relacl::text FROM pg_class WHERE oid IN ('public.company_enrollment_requests'::regclass,'public.company_enrollment_request_events'::regclass,'public.company_enrollment_receipts'::regclass) ORDER BY relname").fetch_all(&pool).await.unwrap();
            let mut tx = pool.begin().await.unwrap();
            sqlx::raw_sql("GRANT UPDATE,DELETE,TRUNCATE ON public.company_enrollment_requests,public.company_enrollment_request_events TO console_account_owner; GRANT TRUNCATE ON public.company_enrollment_receipts TO console_account_owner; SET LOCAL ROLE console_account_owner").execute(tx.as_mut()).await.unwrap();
            refused(
                sqlx::raw_sql(statement).execute(tx.as_mut()).await,
                "P0001",
                Some(expected),
            );
            tx.rollback().await.unwrap();
            assert!(before == all_rows(&pool).await);
            let acl_after:Vec<(String,Option<String>)>=sqlx::query_as("SELECT relname::text,relacl::text FROM pg_class WHERE oid IN ('public.company_enrollment_requests'::regclass,'public.company_enrollment_request_events'::regclass,'public.company_enrollment_receipts'::regclass) ORDER BY relname").fetch_all(&pool).await.unwrap();
            assert_eq!(acl_before, acl_after);
        }
        let cancelled =
            cancel_checked(&pool, &business, account.account, session, command, &before).await;
        for statement in [
            "UPDATE public.company_enrollment_requests SET state='PENDING',terminal_at=NULL WHERE account_id=$1 AND command_id=$2",
            "UPDATE public.company_enrollment_requests SET input_bytes=NULL WHERE account_id=$1 AND command_id=$2",
        ] {
            let mut tx = owner_tx(&pool).await;
            refused(
                sqlx::query(statement)
                    .bind(account.account)
                    .bind(command)
                    .execute(tx.as_mut())
                    .await,
                "P0001",
                Some("company_enrollment.request_transition_invalid"),
            );
            tx.rollback().await.unwrap();
            assert!(cancelled == all_rows(&pool).await);
        }
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_guards_missing_event_fails_deferred_forcing_and_commit(pool: PgPool) {
        let (_app, account, _cookies, startup, _) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        install_guards(&pool).await;
        for terminal in [false, true] {
            for force in [false, true] {
                let command = Uuid::new_v4();
                let input = bytes(account.account, command, account.account);
                if terminal {
                    prepare(&business, account.account, session, command, &input)
                        .await
                        .unwrap();
                }
                let before = all_rows(&pool).await;
                sqlx::raw_sql("CREATE SEQUENCE public.company_guard_fault_seen; CREATE FUNCTION public.company_guard_suppress_event() RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$ BEGIN IF NEW.event_revision=TG_ARGV[0]::bigint THEN PERFORM nextval('public.company_guard_fault_seen'::regclass); RETURN NULL; END IF; RETURN NEW; END $$").execute(&pool).await.unwrap();
                let trigger = if terminal {
                    "CREATE TRIGGER zzz_company_guard_fault BEFORE INSERT ON public.company_enrollment_request_events FOR EACH ROW EXECUTE FUNCTION public.company_guard_suppress_event('2')"
                } else {
                    "CREATE TRIGGER zzz_company_guard_fault BEFORE INSERT ON public.company_enrollment_request_events FOR EACH ROW EXECUTE FUNCTION public.company_guard_suppress_event('1')"
                };
                sqlx::raw_sql(trigger).execute(&pool).await.unwrap();
                let mut tx = business.begin().await.unwrap();
                if terminal {
                    let rows:Vec<(String,Option<Uuid>)>=sqlx::query_as("SELECT state,receipt_id FROM public.company_enrollment_cancel_v1($1,$2,$3)").bind(account.account).bind(session).bind(command).fetch_all(tx.as_mut()).await.unwrap();
                    assert_eq!(rows, vec![("CANCELLED".into(), None)]);
                } else {
                    assert_eq!(
                        prepare_in(&mut tx, account.account, session, command, &input)
                            .await
                            .unwrap(),
                        ("PENDING".into(), None)
                    );
                }
                if force {
                    refused(
                        sqlx::raw_sql("SET CONSTRAINTS ALL IMMEDIATE")
                            .execute(tx.as_mut())
                            .await,
                        "P0001",
                        Some("company_enrollment.intake_closure_invalid"),
                    );
                    tx.rollback().await.unwrap();
                } else {
                    refused(
                        tx.commit().await,
                        "P0001",
                        Some("company_enrollment.intake_closure_invalid"),
                    );
                }
                let fired: bool = sqlx::query_scalar(
                    "SELECT is_called AND last_value=1 FROM public.company_guard_fault_seen",
                )
                .fetch_one(&pool)
                .await
                .unwrap();
                assert!(fired, "actual matching event INSERT must have been omitted");
                sqlx::raw_sql("DROP TRIGGER zzz_company_guard_fault ON public.company_enrollment_request_events; DROP FUNCTION public.company_guard_suppress_event(); DROP SEQUENCE public.company_guard_fault_seen").execute(&pool).await.unwrap();
                assert!(before == all_rows(&pool).await);
            }
        }
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_guards_final_inserted_event_cannot_change_time(pool: PgPool) {
        let (_app, account, _cookies, startup, _) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        install_guards(&pool).await;
        let before = all_rows(&pool).await;
        sqlx::raw_sql("CREATE SEQUENCE public.company_guard_fault_seen; CREATE FUNCTION public.company_guard_change_event() RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$ BEGIN PERFORM nextval('public.company_guard_fault_seen'::regclass); NEW.occurred_at:=NEW.occurred_at+interval '1 microsecond'; RETURN NEW; END $$; CREATE TRIGGER zzz_company_guard_fault BEFORE INSERT ON public.company_enrollment_request_events FOR EACH ROW EXECUTE FUNCTION public.company_guard_change_event()").execute(&pool).await.unwrap();
        let command = Uuid::new_v4();
        refused(
            prepare(
                &business,
                account.account,
                session,
                command,
                &bytes(account.account, command, account.account),
            )
            .await,
            "P0001",
            Some("company_enrollment.event_invalid"),
        );
        let fired: bool = sqlx::query_scalar(
            "SELECT is_called AND last_value=1 FROM public.company_guard_fault_seen",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(fired);
        sqlx::raw_sql("DROP TRIGGER zzz_company_guard_fault ON public.company_enrollment_request_events; DROP FUNCTION public.company_guard_change_event(); DROP SEQUENCE public.company_guard_fault_seen").execute(&pool).await.unwrap();
        assert!(before == all_rows(&pool).await);
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_guards_nonempty_private_install_is_not_retroactive_acceptance(
        pool: PgPool,
    ) {
        let (_app, account, _cookies, startup, _) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        let command = Uuid::new_v4();
        prepare(
            &business,
            account.account,
            session,
            command,
            &bytes(account.account, command, account.account),
        )
        .await
        .unwrap();
        let before = all_rows(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        refused(
            sqlx::raw_sql(GUARDS).execute(tx.as_mut()).await,
            "P0001",
            Some("company_enrollment.guard_install_requires_empty"),
        );
        tx.rollback().await.unwrap();
        assert!(before == all_rows(&pool).await);
        let installed:i64=sqlx::query_scalar("SELECT count(*) FROM pg_proc WHERE pronamespace='public'::regnamespace AND proname='company_enrollment_intake_closure_v1'").fetch_one(&pool).await.unwrap();
        assert_eq!(installed, 0);
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_guards_install_rechecks_empty_after_real_prepare_wait(pool: PgPool) {
        let (_app, account, _cookies, startup, designation) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        let before = all_rows(&pool).await;
        let command = Uuid::new_v4();
        let input = bytes(account.account, command, account.account);
        let lower = now(&pool).await;
        let mut first = business.begin().await.unwrap();
        let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(first.as_mut())
            .await
            .unwrap();
        assert_eq!(
            prepare_in(&mut first, account.account, session, command, &input)
                .await
                .unwrap(),
            ("PENDING".into(), None)
        );
        let mut installer = pool.begin().await.unwrap();
        sqlx::raw_sql("SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='5s'")
            .execute(installer.as_mut())
            .await
            .unwrap();
        let waiter: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(installer.as_mut())
            .await
            .unwrap();
        let observe = async {
            let seen=tokio::time::timeout(std::time::Duration::from_millis(500),async {
                loop {let reached:bool=sqlx::query_scalar("SELECT $2=ANY(pg_blocking_pids($1)) AND EXISTS(SELECT 1 FROM pg_locks WHERE pid=$1 AND relation='public.company_enrollment_requests'::regclass AND mode='AccessExclusiveLock' AND NOT granted)").bind(waiter).bind(holder).fetch_one(&pool).await.unwrap();if reached {break;}tokio::time::sleep(std::time::Duration::from_millis(5)).await;}
            }).await.is_ok();
            first.commit().await.unwrap();
            seen
        };
        let attempt = async {
            let result = sqlx::raw_sql(GUARDS).execute(installer.as_mut()).await;
            installer.rollback().await.unwrap();
            result
        };
        let (result, reached) = tokio::time::timeout(std::time::Duration::from_secs(7), async {
            tokio::join!(attempt, observe)
        })
        .await
        .unwrap();
        assert!(
            reached,
            "actual installer AccessExclusive wait on real in-flight request required"
        );
        refused(
            result,
            "P0001",
            Some("company_enrollment.guard_install_requires_empty"),
        );
        let upper = now(&pool).await;
        pending(
            &pool,
            account.account,
            session,
            command,
            &input,
            designation.command,
            lower,
            upper,
        )
        .await;
        let after = all_rows(&pool).await;
        unrelated_unchanged(&before, &after);
        for table in [
            "company_enrollment_requests",
            "company_enrollment_request_events",
        ] {
            assert_eq!(added_rows(&before[table], &after[table]).unwrap().len(), 1);
        }
        let installed:i64=sqlx::query_scalar("SELECT count(*) FROM pg_proc WHERE pronamespace='public'::regnamespace AND proname='company_enrollment_intake_closure_v1'").fetch_one(&pool).await.unwrap();
        assert_eq!(installed, 0);
        business.close().await;
        startup.close().await;
    }
}
