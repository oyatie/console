// Private SQL-role boundary. No signed-token, HTTP or serving-profile acceptance.
mod intake_owner {
    use super::*;
    use console_identity_application::CompanyEnrollmentV1;
    use console_platform_test_support::{TestDatabaseLogin, login_test_pool};
    use sha2::{Digest, Sha256};

    const PARSER: &str = include_str!("../../../../ops/postgres-company-enrollment-input.sql");
    const SCHEMA: &str = include_str!("../../../../ops/postgres-company-enrollment-schema.sql");
    const INTAKE: &str = include_str!("../../../../ops/postgres-company-enrollment-intake.sql");
    const ROOT_STATE: &str = include_str!("../../src/account_custody_state.sql");
    const CREDENTIAL_STATE: &str = include_str!("../../src/account_credential_custody_state.sql");

    async fn family(pool: &PgPool, account: Uuid) -> Uuid {
        let rows: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM public.auth_refresh_token_families WHERE user_id=$1 AND protocol='ACCOUNT_V1' AND org_id IS NULL AND revoked_at IS NULL")
            .bind(account).fetch_all(pool).await.unwrap();
        assert_eq!(rows.len(), 1, "genuine registered family required");
        rows[0]
    }

    fn bytes(account: Uuid, command: Uuid, recipient: Uuid) -> Vec<u8> {
        CompanyEnrollmentV1::from_json_slice(
            &serde_json::to_vec(&enrollment(command, recipient)).unwrap(),
        )
        .unwrap()
        .encode(account)
        .unwrap()
    }

    async fn profiles(connection: &mut sqlx::PgConnection, root: &str, credential: &str) {
        sqlx::raw_sql("SET search_path=pg_catalog,pg_temp; SET statement_timeout='10s'; SET lock_timeout='1s'")
            .execute(&mut *connection).await.unwrap();
        for (source, expected) in [(ROOT_STATE, root), (CREDENTIAL_STATE, credential)] {
            let actual: String = sqlx::query_scalar(source)
                .fetch_one(&mut *connection)
                .await
                .unwrap();
            assert_eq!(actual, expected, "unchanged custody classifier");
        }
    }

    async fn install(pool: &PgPool) -> PgPool {
        let mut admin = pool.acquire().await.unwrap();
        let genuine: bool = sqlx::query_scalar("SELECT session_user=current_user AND current_user='console_buck_admin' AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' AND (SELECT rolsuper FROM pg_roles WHERE rolname=current_user)")
            .fetch_one(&mut *admin).await.unwrap();
        assert!(genuine);
        profiles(
            &mut admin,
            "account_custody.native_finalized",
            "account_credentials.native_finalized",
        )
        .await;
        sqlx::raw_sql("BEGIN").execute(&mut *admin).await.unwrap();
        for source in [PARSER, SCHEMA, INTAKE] {
            sqlx::raw_sql(source).execute(&mut *admin).await.unwrap();
        }
        sqlx::raw_sql("COMMIT").execute(&mut *admin).await.unwrap();
        profiles(
            &mut admin,
            "account_native.profile_mismatch",
            "account_native.profile_mismatch",
        )
        .await;
        let business = login_test_pool(pool, TestDatabaseLogin::Business).await;
        let identity: (String, String, bool, bool) = sqlx::query_as("SELECT session_user::text,current_user::text,(SELECT rolsuper FROM pg_roles WHERE rolname=current_user),(SELECT rolbypassrls FROM pg_roles WHERE rolname=current_user)")
            .fetch_one(&business).await.unwrap();
        assert_eq!(
            identity,
            ("console_rt".into(), "console_rt".into(), false, false)
        );
        business
    }

    async fn material(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        account: Uuid,
        session: Uuid,
        command: Uuid,
        input: Option<&[u8]>,
    ) -> Result<(Option<Uuid>, Option<Vec<u8>>), sqlx::Error> {
        let rows =
            sqlx::query("SELECT * FROM public.company_enrollment_session_material_v1($1,$2,$3,$4)")
                .bind(account)
                .bind(session)
                .bind(command)
                .bind(input)
                .fetch_all(tx.as_mut())
                .await?;
        assert_eq!(rows.len(), 1, "exactly one material row");
        let row = &rows[0];
        let state: String = row.try_get("security_state")?;
        let generation: i64 = row.try_get("security_generation")?;
        let family_generation: Option<i64> = row.try_get("account_security_generation")?;
        let owner: Uuid = row.try_get("user_id")?;
        let protocol: String = row.try_get("protocol")?;
        let assurance: Option<String> = row.try_get("assurance")?;
        let org: Option<Uuid> = row.try_get("org_id")?;
        let revoked: Option<time::OffsetDateTime> = row.try_get("revoked_at")?;
        let auth_time: Option<time::OffsetDateTime> = row.try_get("auth_time")?;
        let created: time::OffsetDateTime = row.try_get("created_at")?;
        let revision: i64 = row.try_get("revision")?;
        let context_generation: i64 = row.try_get("context_generation")?;
        assert!(
            state == "ACTIVE"
                && generation > 0
                && family_generation == Some(generation)
                && owner == account
                && protocol == "ACCOUNT_V1"
                && assurance.as_deref() == Some("PASSKEY_PRIMARY")
                && org.is_none()
                && revoked.is_none()
                && auth_time.is_some_and(|at| at <= created)
                && revision > 0
                && context_generation > 0,
            "actual native material, never a fabricated Rust session"
        );
        let recipient: Option<Uuid> = row.try_get("planned_recipient")?;
        let digest: Option<Vec<u8>> = row.try_get("planned_input_digest")?;
        assert!(digest.as_ref().is_none_or(|bytes| bytes.len() == 32));
        Ok((recipient, digest))
    }

    async fn plan(
        pool: &PgPool,
        account: Uuid,
        session: Uuid,
        command: Uuid,
        input: Option<&[u8]>,
    ) -> (Option<Uuid>, Option<Vec<u8>>) {
        let mut tx = pool.begin().await.unwrap();
        let value = material(&mut tx, account, session, command, input)
            .await
            .unwrap();
        tx.rollback().await.unwrap();
        value
    }

    async fn now(pool: &PgPool) -> time::OffsetDateTime {
        sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn prepare_in(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        account: Uuid,
        session: Uuid,
        command: Uuid,
        input: &[u8],
    ) -> Result<(String, Option<Uuid>), sqlx::Error> {
        sqlx::raw_sql("SET LOCAL statement_timeout='5s'; SET LOCAL lock_timeout='1s'")
            .execute(tx.as_mut())
            .await?;
        material(tx, account, session, command, Some(input)).await?;
        let rows: Vec<(String, Option<Uuid>)> = sqlx::query_as(
            "SELECT state,receipt_id FROM public.company_enrollment_prepare_v1($1,$2,$3,$4)",
        )
        .bind(account)
        .bind(session)
        .bind(command)
        .bind(input)
        .fetch_all(tx.as_mut())
        .await?;
        assert_eq!(rows.len(), 1, "exactly one prepare row");
        Ok(rows.into_iter().next().unwrap())
    }

    async fn prepare(
        pool: &PgPool,
        account: Uuid,
        session: Uuid,
        command: Uuid,
        input: &[u8],
    ) -> Result<(String, Option<Uuid>), sqlx::Error> {
        let mut tx = pool.begin().await?;
        match prepare_in(&mut tx, account, session, command, input).await {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    async fn status(
        pool: &PgPool,
        account: Uuid,
        session: Uuid,
        command: Uuid,
    ) -> Result<Option<(String, Option<Vec<u8>>)>, sqlx::Error> {
        let mut tx = pool.begin().await?;
        material(&mut tx, account, session, command, None).await?;
        let rows = sqlx::query("SELECT * FROM public.company_enrollment_status_v1($1,$2,$3)")
            .bind(account)
            .bind(session)
            .bind(command)
            .fetch_all(tx.as_mut())
            .await?;
        assert!(rows.len() <= 1, "zero or one status row");
        let result = rows.into_iter().next().map(|row| {
            assert_eq!(row.get::<i16, _>("codec_version"), 1);
            for column in [
                "receipt_id",
                "org_id",
                "group_id",
                "administrative_account_id",
            ] {
                assert!(
                    row.get::<Option<Uuid>, _>(column).is_none(),
                    "pending-only tests cannot fabricate committed results"
                );
            }
            (row.get("state"), row.get("input_bytes"))
        });
        tx.commit().await?;
        Ok(result)
    }

    async fn cancel(
        pool: &PgPool,
        account: Uuid,
        session: Uuid,
        command: Uuid,
    ) -> Result<String, sqlx::Error> {
        let mut tx = pool.begin().await?;
        material(&mut tx, account, session, command, None).await?;
        let rows = sqlx::query("SELECT * FROM public.company_enrollment_cancel_v1($1,$2,$3)")
            .bind(account)
            .bind(session)
            .bind(command)
            .fetch_all(tx.as_mut())
            .await?;
        assert_eq!(
            rows.len(),
            1,
            "exactly one existing-command cancellation row"
        );
        let row = &rows[0];
        for column in [
            "receipt_id",
            "org_id",
            "group_id",
            "administrative_account_id",
        ] {
            assert!(row.get::<Option<Uuid>, _>(column).is_none());
        }
        let state = row.get("state");
        tx.commit().await?;
        Ok(state)
    }

    fn unrelated_unchanged(before: &BTreeMap<String, String>, after: &BTreeMap<String, String>) {
        let mut before = before.clone();
        let mut after = after.clone();
        for table in [
            "company_enrollment_requests",
            "company_enrollment_request_events",
        ] {
            assert!(before.remove(table).is_some());
            assert!(after.remove(table).is_some());
        }
        assert!(
            before == after,
            "all non-intake histories must remain byte-identical"
        );
    }

    // Preserve every other raw row byte-for-byte; only target state/time may change.
    fn cancel_successor(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
        account: Uuid,
        command: Uuid,
        session: Uuid,
    ) -> bool {
        use serde_json::value::RawValue;
        let mut rest_before = before.clone();
        let mut rest_after = after.clone();
        let (Some(before_requests), Some(after_requests), Some(before_events), Some(after_events)) = (
            rest_before.remove("company_enrollment_requests"),
            rest_after.remove("company_enrollment_requests"),
            rest_before.remove("company_enrollment_request_events"),
            rest_after.remove("company_enrollment_request_events"),
        ) else {
            return false;
        };
        if rest_before != rest_after {
            return false;
        }
        let (Ok(before_rows), Ok(after_rows)) = (
            serde_json::from_str::<Vec<&RawValue>>(&before_requests),
            serde_json::from_str::<Vec<&RawValue>>(&after_requests),
        ) else {
            return false;
        };
        let split = |rows: Vec<&RawValue>| -> Option<(Vec<String>, Value)> {
            let mut other = Vec::new();
            let mut target = None;
            for row in rows {
                let value: Value = serde_json::from_str(row.get()).ok()?;
                if value["account_id"] == json!(account) && value["command_id"] == json!(command) {
                    if target.replace(value).is_some() {
                        return None;
                    }
                } else {
                    other.push(row.get().to_owned());
                }
            }
            Some((other, target?))
        };
        let (Some((before_other, mut expected)), Some((after_other, actual))) =
            (split(before_rows), split(after_rows))
        else {
            return false;
        };
        if before_other != after_other
            || expected["state"] != "PENDING"
            || !expected["terminal_at"].is_null()
        {
            return false;
        }
        let Some(terminal) = actual
            .get("terminal_at")
            .filter(|value| value.is_string())
            .cloned()
        else {
            return false;
        };
        expected["state"] = json!("CANCELLED");
        expected["terminal_at"] = terminal.clone();
        if expected != actual {
            return false;
        }
        let Some(events) = added_rows(&before_events, &after_events) else {
            return false;
        };
        events
            == vec![
                json!({"account_id":account,"command_id":command,"event_revision":2,"from_state":"PENDING","to_state":"CANCELLED","occurred_at":terminal,"actor_account_id":account,"session_id":session,"reason_code":"CANCELLED"}),
            ]
    }

    async fn cancel_checked(
        pool: &PgPool,
        business: &PgPool,
        account: Uuid,
        session: Uuid,
        command: Uuid,
        before: &BTreeMap<String, String>,
    ) -> BTreeMap<String, String> {
        let lower = now(pool).await;
        assert_eq!(
            cancel(business, account, session, command).await.unwrap(),
            "CANCELLED"
        );
        let upper = now(pool).await;
        let after = all_rows(pool).await;
        assert!(
            cancel_successor(before, &after, account, command, session),
            "exact target-only request transition and append-only event required"
        );
        let valid:bool=sqlx::query_scalar("SELECT terminal_at >= $3 AND terminal_at <= $4 AND terminal_at >= created_at FROM public.company_enrollment_requests WHERE account_id=$1 AND command_id=$2")
            .bind(account).bind(command).bind(lower).bind(upper).fetch_one(pool).await.unwrap();
        assert!(valid, "actual DB clock bounds");
        after
    }

    #[test]
    fn company_intake_cancel_oracle_rejects_missing_and_rewritten_history() {
        let account = Uuid::from_u128(1);
        let command = Uuid::from_u128(2);
        let session = Uuid::from_u128(3);
        let original = json!({"account_id":account,"command_id":command,"codec_version":1,"created_at":"created","expires_at":"expires","designation_receipt_id":"origin","input_bytes":"bytes","input_digest":"digest","state":"PENDING","terminal_at":null,"committed_receipt_id":null});
        let foreign = json!({"account_id":Uuid::from_u128(4),"command_id":Uuid::from_u128(5),"state":"PENDING"});
        let initial = json!({"account_id":account,"command_id":command,"event_revision":1,"from_state":null,"to_state":"PENDING","occurred_at":"created","actor_account_id":account,"session_id":session,"reason_code":"PREPARED"});
        let mut cancelled = original.clone();
        cancelled["state"] = json!("CANCELLED");
        cancelled["terminal_at"] = json!("terminal");
        let terminal = json!({"account_id":account,"command_id":command,"event_revision":2,"from_state":"PENDING","to_state":"CANCELLED","occurred_at":"terminal","actor_account_id":account,"session_id":session,"reason_code":"CANCELLED"});
        let before = BTreeMap::from([
            (
                "company_enrollment_requests".into(),
                json!([original, foreign]).to_string(),
            ),
            (
                "company_enrollment_request_events".into(),
                json!([initial]).to_string(),
            ),
            ("unrelated".into(), "[{\"exact\":9007199254740993}]".into()),
        ]);
        let after = BTreeMap::from([
            (
                "company_enrollment_requests".into(),
                json!([cancelled, foreign]).to_string(),
            ),
            (
                "company_enrollment_request_events".into(),
                json!([initial, terminal]).to_string(),
            ),
            ("unrelated".into(), "[{\"exact\":9007199254740993}]".into()),
        ]);
        assert!(cancel_successor(&before, &after, account, command, session));
        let mut failures = 0;
        for field in [
            "codec_version",
            "created_at",
            "expires_at",
            "designation_receipt_id",
            "input_bytes",
            "input_digest",
        ] {
            let mut corrupt = after.clone();
            let mut row = cancelled.clone();
            row[field] = json!("corrupt");
            corrupt.insert(
                "company_enrollment_requests".into(),
                json!([row, foreign]).to_string(),
            );
            assert!(!cancel_successor(
                &before, &corrupt, account, command, session
            ));
            failures += 1;
        }
        for rows in [
            json!([cancelled]),
            json!([cancelled, foreign, foreign]),
            json!([cancelled,{"account_id":"rewritten-other"}]),
        ] {
            let mut corrupt = after.clone();
            corrupt.insert("company_enrollment_requests".into(), rows.to_string());
            assert!(!cancel_successor(
                &before, &corrupt, account, command, session
            ));
            failures += 1;
        }
        let mut bad_time = terminal.clone();
        bad_time["occurred_at"] = json!("other-time");
        for events in [
            json!([terminal]),
            json!([initial, bad_time]),
            json!([initial, terminal, terminal]),
        ] {
            let mut corrupt = after.clone();
            corrupt.insert(
                "company_enrollment_request_events".into(),
                events.to_string(),
            );
            assert!(!cancel_successor(
                &before, &corrupt, account, command, session
            ));
            failures += 1;
        }
        let mut corrupt = after.clone();
        corrupt.insert("unrelated".into(), "[{\"exact\":9007199254740992}]".into());
        assert!(!cancel_successor(
            &before, &corrupt, account, command, session
        ));
        failures += 1;
        assert_eq!(failures, 13);
    }

    async fn pending(
        pool: &PgPool,
        account: Uuid,
        session: Uuid,
        command: Uuid,
        input: &[u8],
        designation_command: Uuid,
        lower: time::OffsetDateTime,
        upper: time::OffsetDateTime,
    ) {
        let row = sqlx::query("SELECT r.*, (SELECT count(*) FROM public.company_enrollment_request_events e WHERE (e.account_id,e.command_id)=(r.account_id,r.command_id)) AS events, EXISTS(SELECT 1 FROM public.company_enrollment_request_events e WHERE (e.account_id,e.command_id)=(r.account_id,r.command_id) AND e.event_revision=1 AND e.from_state IS NULL AND e.to_state='PENDING' AND e.reason_code='PREPARED' AND e.occurred_at=r.created_at AND e.actor_account_id=r.account_id AND e.session_id=$3) AS exact_event, EXISTS(SELECT 1 FROM public.deployment_operator_receipts d WHERE d.receipt_id=r.designation_receipt_id AND d.account_id=r.account_id AND d.command_id=$4 AND d.kind='DESIGNATE') AS exact_origin FROM public.company_enrollment_requests r WHERE account_id=$1 AND command_id=$2")
            .bind(account).bind(command).bind(session).bind(designation_command).fetch_one(pool).await.unwrap();
        assert_eq!(row.get::<i16, _>("codec_version"), 1);
        assert!(row.get::<Vec<u8>, _>("input_bytes") == input);
        assert!(row.get::<Vec<u8>, _>("input_digest") == Sha256::digest(input).as_slice());
        assert_eq!(row.get::<String, _>("state"), "PENDING");
        assert!(row.get::<Option<Uuid>, _>("committed_receipt_id").is_none());
        assert!(
            row.get::<Option<time::OffsetDateTime>, _>("terminal_at")
                .is_none()
        );
        let created: time::OffsetDateTime = row.get("created_at");
        let expires: time::OffsetDateTime = row.get("expires_at");
        assert!(
            created >= lower
                && created <= upper
                && expires > created
                && expires <= created + time::Duration::hours(168)
        );
        assert!(row.get::<bool, _>("exact_event") && row.get::<bool, _>("exact_origin"));
        assert_eq!(row.get::<i64, _>("events"), 1);
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_sql_pending_history_and_event_fault_are_atomic(pool: PgPool) {
        let (_app, account, _cookies, startup, designation) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        let command = Uuid::new_v4();
        let input = bytes(account.account, command, account.account);
        assert_eq!(
            plan(&business, account.account, session, command, Some(&input)).await,
            (Some(account.account), None)
        );
        let before = all_rows(&pool).await;
        sqlx::raw_sql("CREATE SEQUENCE public.company_intake_event_fault_seen; CREATE FUNCTION public.company_intake_event_fault() RETURNS trigger LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,pg_temp AS $$ BEGIN IF NOT EXISTS(SELECT 1 FROM public.company_enrollment_requests WHERE account_id=NEW.account_id AND command_id=NEW.command_id AND state='PENDING') THEN RAISE EXCEPTION 'COMPANY_INTAKE_REQUEST_NOT_REACHED'; END IF; PERFORM nextval('public.company_intake_event_fault_seen'::regclass); RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='COMPANY_INTAKE_EVENT_FAULT'; END $$; CREATE TRIGGER company_intake_event_fault BEFORE INSERT ON public.company_enrollment_request_events FOR EACH ROW EXECUTE FUNCTION public.company_intake_event_fault()")
            .execute(&pool).await.unwrap();
        let failed = prepare(&business, account.account, session, command, &input).await;
        let fired: bool = sqlx::query_scalar(
            "SELECT is_called AND last_value=1 FROM public.company_intake_event_fault_seen",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::raw_sql("DROP TRIGGER company_intake_event_fault ON public.company_enrollment_request_events; DROP FUNCTION public.company_intake_event_fault(); DROP SEQUENCE public.company_intake_event_fault_seen").execute(&pool).await.unwrap();
        assert!(
            fired,
            "must reach actual event owner, not unrelated missing schema"
        );
        refused(failed, "P0001", Some("COMPANY_INTAKE_EVENT_FAULT"));
        assert!(before == all_rows(&pool).await);
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
        let digest = Sha256::digest(&input).to_vec();
        assert_eq!(
            plan(&business, account.account, session, command, Some(&input)).await,
            (Some(account.account), Some(digest.clone()))
        );
        assert_eq!(
            plan(&business, account.account, session, command, None).await,
            (None, Some(digest))
        );
        let after = all_rows(&pool).await;
        unrelated_unchanged(&before, &after);
        assert_eq!(
            added_rows(
                &before["company_enrollment_requests"],
                &after["company_enrollment_requests"]
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            added_rows(
                &before["company_enrollment_request_events"],
                &after["company_enrollment_request_events"]
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            status(&business, account.account, session, command)
                .await
                .unwrap(),
            Some(("PENDING".into(), Some(input.clone())))
        );
        assert_eq!(
            prepare(&business, account.account, session, command, &input)
                .await
                .unwrap(),
            ("PENDING".into(), None)
        );
        assert!(
            after == all_rows(&pool).await,
            "reopen/exact replay must be effect-free"
        );
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_sql_conflict_namespace_and_foreign_recovery_preserve_history(
        pool: PgPool,
    ) {
        let (app, account, _cookies, startup, _) = designated(&pool).await;
        let (foreign, _) = enrolled(&app).await;
        let session = family(&pool, account.account).await;
        let foreign_session = family(&pool, foreign.account).await;
        let business = install(&pool).await;
        let command = Uuid::new_v4();
        let input = bytes(account.account, command, account.account);
        prepare(&business, account.account, session, command, &input)
            .await
            .unwrap();
        let before = all_rows(&pool).await;
        let changed = bytes(account.account, command, foreign.account);
        assert_eq!(
            plan(&business, account.account, session, command, Some(&changed)).await,
            (Some(account.account), Some(Sha256::digest(&input).to_vec()))
        );
        assert_eq!(
            plan(&business, account.account, session, command, None).await,
            (None, Some(Sha256::digest(&input).to_vec()))
        );
        assert_eq!(
            plan(&business, account.account, session, Uuid::new_v4(), None).await,
            (None, None)
        );
        refused(
            prepare(&business, account.account, session, command, &changed).await,
            "P0001",
            Some("company_enrollment.conflict"),
        );
        refused(
            prepare(
                &business,
                account.account,
                session,
                command,
                &bytes(foreign.account, command, account.account),
            )
            .await,
            "22023",
            Some("company_enrollment.invalid_input"),
        );
        refused(
            prepare(&business, account.account, session, Uuid::new_v4(), &input).await,
            "22023",
            Some("company_enrollment.invalid_input"),
        );
        refused(
            status(&business, account.account, foreign_session, command).await,
            "P0001",
            Some("account.authentication_invalid"),
        );
        assert!(
            status(&business, foreign.account, foreign_session, command)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            status(&business, account.account, session, Uuid::new_v4())
                .await
                .unwrap()
                .is_none()
        );
        assert!(before == all_rows(&pool).await);
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_sql_cancel_survives_designation_revoke_and_never_reopens(pool: PgPool) {
        let (_app, account, _cookies, startup, designation) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        let command = Uuid::new_v4();
        let input = bytes(account.account, command, account.account);
        prepare(&business, account.account, session, command, &input)
            .await
            .unwrap();
        revoke(
            &startup,
            &designation,
            Uuid::new_v4(),
            1,
            "intake recovery test",
        )
        .await
        .unwrap();
        let before = all_rows(&pool).await;
        assert_eq!(
            prepare(&business, account.account, session, command, &input)
                .await
                .unwrap(),
            ("PENDING".into(), None)
        );
        let new = Uuid::new_v4();
        refused(
            prepare(
                &business,
                account.account,
                session,
                new,
                &bytes(account.account, new, account.account),
            )
            .await,
            "P0001",
            Some("company_enrollment.forbidden"),
        );
        assert!(before == all_rows(&pool).await);
        let after =
            cancel_checked(&pool, &business, account.account, session, command, &before).await;
        assert_eq!(
            status(&business, account.account, session, command)
                .await
                .unwrap(),
            Some(("CANCELLED".into(), None))
        );
        assert_eq!(
            cancel(&business, account.account, session, command)
                .await
                .unwrap(),
            "CANCELLED"
        );
        assert_eq!(
            prepare(&business, account.account, session, command, &input)
                .await
                .unwrap(),
            ("CANCELLED".into(), None)
        );
        assert!(after == all_rows(&pool).await);
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_sql_capacity_is_atomic_and_cancel_releases_one_slot(pool: PgPool) {
        let (_app, account, _cookies, startup, designation) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        let before = all_rows(&pool).await;
        let lower = now(&pool).await;
        let mut commands = Vec::new();
        for _ in 0..15 {
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
            commands.push(command);
        }
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let ai = bytes(account.account, a, account.account);
        let bi = bytes(account.account, b, account.account);
        let (left, right) = tokio::join!(
            prepare(&business, account.account, session, a, &ai),
            prepare(&business, account.account, session, b, &bi)
        );
        let (winner, loser) = match (left, right) {
            (Ok(value), Err(error)) => {
                assert_eq!(value, ("PENDING".into(), None));
                refused::<()>(Err(error), "P0001", Some("company_enrollment.capacity"));
                (a, b)
            }
            (Err(error), Ok(value)) => {
                assert_eq!(value, ("PENDING".into(), None));
                refused::<()>(Err(error), "P0001", Some("company_enrollment.capacity"));
                (b, a)
            }
            _ => panic!("exactly one final capacity slot must be admitted"),
        };
        let upper = now(&pool).await;
        commands.push(winner);
        for command in &commands {
            pending(
                &pool,
                account.account,
                session,
                *command,
                &bytes(account.account, *command, account.account),
                designation.command,
                lower,
                upper,
            )
            .await;
        }
        let full = all_rows(&pool).await;
        unrelated_unchanged(&before, &full);
        assert_eq!(
            added_rows(
                &before["company_enrollment_requests"],
                &full["company_enrollment_requests"]
            )
            .unwrap()
            .len(),
            16
        );
        assert_eq!(
            added_rows(
                &before["company_enrollment_request_events"],
                &full["company_enrollment_request_events"]
            )
            .unwrap()
            .len(),
            16
        );
        prepare(
            &business,
            account.account,
            session,
            winner,
            &bytes(account.account, winner, account.account),
        )
        .await
        .unwrap();
        refused(
            prepare(
                &business,
                account.account,
                session,
                loser,
                &bytes(account.account, loser, account.account),
            )
            .await,
            "P0001",
            Some("company_enrollment.capacity"),
        );
        assert!(full == all_rows(&pool).await);
        let cancelled = cancel_checked(
            &pool,
            &business,
            account.account,
            session,
            commands[0],
            &full,
        )
        .await;
        let replacement = bytes(account.account, loser, account.account);
        let replacement_lower = now(&pool).await;
        assert_eq!(
            prepare(&business, account.account, session, loser, &replacement)
                .await
                .unwrap(),
            ("PENDING".into(), None)
        );
        let replacement_upper = now(&pool).await;
        pending(
            &pool,
            account.account,
            session,
            loser,
            &replacement,
            designation.command,
            replacement_lower,
            replacement_upper,
        )
        .await;
        let replaced = all_rows(&pool).await;
        unrelated_unchanged(&cancelled, &replaced);
        assert_eq!(
            added_rows(
                &cancelled["company_enrollment_requests"],
                &replaced["company_enrollment_requests"]
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            added_rows(
                &cancelled["company_enrollment_request_events"],
                &replaced["company_enrollment_request_events"]
            )
            .unwrap()
            .len(),
            1
        );
        for command in &commands[1..] {
            pending(
                &pool,
                account.account,
                session,
                *command,
                &bytes(account.account, *command, account.account),
                designation.command,
                lower,
                upper,
            )
            .await;
        }

        let counts:(i64,i64,i64)=sqlx::query_as("SELECT (SELECT count(*) FROM public.company_enrollment_requests WHERE account_id=$1 AND state='PENDING'),(SELECT count(*) FROM public.company_enrollment_requests WHERE account_id=$1 AND state='CANCELLED'),(SELECT count(*) FROM public.company_enrollment_request_events WHERE account_id=$1)")
            .bind(account.account).fetch_one(&pool).await.unwrap();
        assert_eq!(counts, (16, 1, 18));
        unrelated_unchanged(&before, &all_rows(&pool).await);
        business.close().await;
        startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_intake_sql_same_command_waits_then_replays_one_durable_history(pool: PgPool) {
        let (_app, account, _cookies, startup, designation) = designated(&pool).await;
        let session = family(&pool, account.account).await;
        let business = install(&pool).await;
        let before = all_rows(&pool).await;
        let command = Uuid::new_v4();
        let input = bytes(account.account, command, account.account);
        let lower = now(&pool).await;
        let mut first = business.begin().await.unwrap();
        let blocker: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(first.as_mut())
            .await
            .unwrap();
        prepare_in(&mut first, account.account, session, command, &input)
            .await
            .unwrap();
        let mut connection = business.acquire().await.unwrap();
        let waiter: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        let copied = input.clone();
        let actor = account.account;
        let task = tokio::spawn(async move {
            let mut tx = connection.begin().await.unwrap();
            let value = prepare_in(&mut tx, actor, session, command, &copied).await;
            match value {
                Ok(value) => {
                    tx.commit().await.unwrap();
                    Ok(value)
                }
                Err(error) => {
                    tx.rollback().await.unwrap();
                    Err(error)
                }
            }
        });
        let mut observed = false;
        for _ in 0..100 {
            let blockers: Vec<i32> = sqlx::query_scalar("SELECT pg_blocking_pids($1)")
                .bind(waiter)
                .fetch_one(&pool)
                .await
                .unwrap();
            if blockers.contains(&blocker) {
                observed = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        first.commit().await.unwrap();
        let replay = tokio::time::timeout(std::time::Duration::from_secs(5), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(observed, "actual owner blocking witness required");
        assert_eq!(replay, ("PENDING".into(), None));
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
        assert_eq!(
            added_rows(
                &before["company_enrollment_requests"],
                &after["company_enrollment_requests"]
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            added_rows(
                &before["company_enrollment_request_events"],
                &after["company_enrollment_request_events"]
            )
            .unwrap()
            .len(),
            1
        );
        business.close().await;
        startup.close().await;
    }
}
