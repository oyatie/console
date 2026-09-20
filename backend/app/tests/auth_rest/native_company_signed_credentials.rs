// Include INSIDE reviewed intake_owner. Auth seam proposal; runtime tests0.
mod signed_credentials {
    use super::*;
    use console_platform_auth::account::{AccountEnrollmentCredentials, AccountOperationError};
    use console_platform_auth::{JwtSettings, JwtVerifier};

    struct Signed {
        account: Uuid,
        session: Uuid,
        access: String,
        csrf: String,
        other_family_csrf: String,
        foreign_csrf: String,
        key: SigningKey,
        verifier: JwtVerifier,
        ttl: time::Duration,
        business: PgPool,
        startup: PgPool,
        designation: Uuid,
    }

    async fn fixture(pool: &PgPool, revoked: bool) -> Signed {
        let (app, key) = signed_fixture(pool).await;
        let (mut account, cookies) = enrolled(&app).await;
        let csrf = proof(&app, &cookies).await;
        let other = fresh_login(&app, &mut account).await;
        let other_family_csrf = proof(&app, &other).await;
        let (_, foreign) = enrolled(&app).await;
        let foreign_csrf = proof(&app, &foreign).await;
        let startup = startup(pool).await;
        let designation = designation(pool, account.account).await;
        let receipt = designate(&startup, &designation).await.unwrap();
        assert!(!receipt.0.is_nil() && receipt.1 == 1 && !receipt.2);
        let config = account_browser_config(pool, app._artifacts.root.clone(), &key);
        let jwt = config.jwt.unwrap();
        let ttl = config.auth_rest.unwrap().refresh_family_absolute_ttl;
        let verifier = JwtVerifier::from_es256_public_pem(
            JwtSettings {
                issuer: jwt.issuer,
                audience: jwt.audience,
                access_token_ttl: time::Duration::minutes(15),
            },
            jwt.public_key_pem.as_bytes(),
        )
        .unwrap();
        let claims = signed_claims(&cookies.0[ACCESS], &key).unwrap();
        let session = Uuid::parse_str(claims["sid"].as_str().unwrap()).unwrap();
        assert_eq!(claims["sub"], json!(account.account));
        if revoked {
            let response = request(
                &app,
                "POST",
                "/api/v2/auth/logout",
                &cookies,
                Some(json!({})),
                &[("X-Console-CSRF", &csrf)],
            )
            .await;
            assert_eq!(response.status, StatusCode::OK);
            let actual:bool=sqlx::query_scalar("SELECT revoked_at IS NOT NULL FROM public.auth_refresh_token_families WHERE id=$1 AND user_id=$2")
                .bind(session).bind(account.account).fetch_one(pool).await.unwrap();
            assert!(actual, "real logout owner must revoke selected family");
        }
        let business = install(pool).await;
        Signed {
            account: account.account,
            session,
            access: cookies.0[ACCESS].clone(),
            csrf,
            other_family_csrf,
            foreign_csrf,
            key,
            verifier,
            ttl,
            business,
            startup,
            designation: designation.command,
        }
    }

    async fn prepare_signed(
        f: &Signed,
        command: Uuid,
        access: &str,
        csrf: &str,
    ) -> Result<(), AccountOperationError> {
        let mut tx = f.business.begin().await?;
        let result = async {
            let credentials = AccountEnrollmentCredentials::for_mutation(access, csrf)?;
            let account = credentials
                .account_id_in_tx(&mut tx, &f.verifier, f.ttl)
                .await?;
            assert_eq!(account, f.account);
            let input = bytes(account, command, account);
            let mut guard = credentials
                .lock_submit_in_tx(&mut tx, &f.verifier, f.ttl, command, &input)
                .await?;
            let rows: Vec<(String, Option<Uuid>)> = sqlx::query_as(
                "SELECT state,receipt_id FROM public.company_enrollment_prepare_v1($1,$2,$3,$4)",
            )
            .bind(guard.account_id())
            .bind(guard.session_id())
            .bind(command)
            .bind(&input)
            .fetch_all(guard.connection())
            .await?;
            assert_eq!(rows, vec![("PENDING".into(), None)]);
            guard.finish().await?;
            Ok(())
        }
        .await;
        match result {
            Ok(()) => {
                tx.commit().await?;
                Ok(())
            }
            Err(error) => {
                tx.rollback().await?;
                Err(error)
            }
        }
    }

    #[sqlx::test(migrations = false)]
    async fn company_signed_current_credentials_prepare_reopen_and_cancel_exact_history(
        pool: PgPool,
    ) {
        let f = fixture(&pool, false).await;
        let command = Uuid::new_v4();
        let input = bytes(f.account, command, f.account);
        let before = all_rows(&pool).await;
        let lower = now(&pool).await;
        prepare_signed(&f, command, &f.access, &f.csrf)
            .await
            .unwrap();
        let upper = now(&pool).await;
        pending(
            &pool,
            f.account,
            f.session,
            command,
            &input,
            f.designation,
            (lower, upper),
        )
        .await;
        let prepared = all_rows(&pool).await;
        unrelated_unchanged(&before, &prepared);
        for relation in [
            "company_enrollment_requests",
            "company_enrollment_request_events",
        ] {
            assert_eq!(
                added_rows(&before[relation], &prepared[relation])
                    .unwrap()
                    .len(),
                1
            );
        }
        let mut tx = f.business.begin().await.unwrap();
        let read = AccountEnrollmentCredentials::for_read(&f.access).unwrap();
        assert!(matches!(
            read.lock_submit_in_tx(&mut tx, &f.verifier, f.ttl, command, &input)
                .await,
            Err(AccountOperationError::CsrfInvalid)
        ));
        assert!(matches!(
            read.lock_cancel_in_tx(&mut tx, &f.verifier, f.ttl, command)
                .await,
            Err(AccountOperationError::CsrfInvalid)
        ));
        let mut guard = read
            .lock_status_in_tx(&mut tx, &f.verifier, f.ttl, command)
            .await
            .unwrap();
        assert!(guard.planned_recipient().is_none());
        assert_eq!(
            guard.planned_input_digest(),
            Some(Sha256::digest(&input).as_slice())
        );
        let rows = sqlx::query("SELECT * FROM public.company_enrollment_status_v1($1,$2,$3)")
            .bind(f.account)
            .bind(f.session)
            .bind(command)
            .fetch_all(guard.connection())
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<String, _>("state"), "PENDING");
        assert_eq!(rows[0].get::<i16, _>("codec_version"), 1);
        assert!(rows[0].get::<Vec<u8>, _>("input_bytes") == input);
        for name in [
            "receipt_id",
            "org_id",
            "group_id",
            "administrative_account_id",
        ] {
            assert!(rows[0].get::<Option<Uuid>, _>(name).is_none());
        }
        guard.finish().await.unwrap();
        tx.commit().await.unwrap();
        assert!(prepared == all_rows(&pool).await);
        let mut tx = f.business.begin().await.unwrap();
        let credentials = AccountEnrollmentCredentials::for_mutation(&f.access, &f.csrf).unwrap();
        let lower = now(&pool).await;
        let mut guard = credentials
            .lock_cancel_in_tx(&mut tx, &f.verifier, f.ttl, command)
            .await
            .unwrap();
        let rows = sqlx::query("SELECT * FROM public.company_enrollment_cancel_v1($1,$2,$3)")
            .bind(f.account)
            .bind(f.session)
            .bind(command)
            .fetch_all(guard.connection())
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<String, _>("state"), "CANCELLED");
        for name in [
            "receipt_id",
            "org_id",
            "group_id",
            "administrative_account_id",
        ] {
            assert!(rows[0].get::<Option<Uuid>, _>(name).is_none());
        }
        guard.finish().await.unwrap();
        tx.commit().await.unwrap();
        let upper = now(&pool).await;
        assert!(cancel_successor(
            &prepared,
            &all_rows(&pool).await,
            f.account,
            command,
            f.session
        ));
        let bounded:bool=sqlx::query_scalar("SELECT terminal_at >= $3 AND terminal_at <= $4 FROM public.company_enrollment_requests WHERE account_id=$1 AND command_id=$2").bind(f.account).bind(command).bind(lower).bind(upper).fetch_one(&pool).await.unwrap();
        assert!(bounded);
        f.business.close().await;
        f.startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_signed_invalid_access_and_bound_proofs_preserve_every_row(pool: PgPool) {
        let f = fixture(&pool, false).await;
        let before = all_rows(&pool).await;
        let access_claims = signed_claims(&f.access, &f.key).unwrap();
        let proof_claims = signed_claims(&f.csrf, &f.key).unwrap();
        let bad_key_access = sign_proof_claims(&access_claims, &SigningKey::random(&mut OsRng));
        assert!(
            f.verifier
                .verify_account_access_token(&bad_key_access, now(&pool).await)
                .is_err()
        );
        let mut cases = vec![
            ("".to_owned(), f.csrf.clone(), false),
            (f.csrf.clone(), f.csrf.clone(), false),
            (bad_key_access.clone(), f.csrf.clone(), false),
            (f.access.clone(), "".to_owned(), true),
            (f.access.clone(), f.access.clone(), true),
            (f.access.clone(), f.other_family_csrf.clone(), true),
            (f.access.clone(), f.foreign_csrf.clone(), true),
        ];
        // Actual signature damage, not only a different otherwise-trusted key.
        let mut damaged = f.access.clone();
        let signature = damaged.rfind('.').unwrap() + 1;
        let replacement = if &damaged[signature..signature + 1] == "A" {
            "B"
        } else {
            "A"
        };
        damaged.replace_range(signature..signature + 1, replacement);
        assert!(
            f.verifier
                .verify_account_access_token(&damaged, now(&pool).await)
                .is_err()
        );
        cases.push((damaged, f.csrf.clone(), false));
        let expiry = access_claims["iat"].as_i64().unwrap() + 1;
        sqlx::query("SELECT pg_sleep(GREATEST(0.0,$1::double precision-extract(epoch FROM clock_timestamp()))+0.025)").bind(expiry as f64).execute(&pool).await.unwrap();
        let at = now(&pool).await.unix_timestamp();
        let mut expired = access_claims.clone();
        expired["exp"] = json!(expiry);
        let expired = sign_proof_claims(&expired, &f.key);
        assert!(matches!(
            f.verifier
                .verify_account_access_token(&expired, now(&pool).await)
                .unwrap(),
            console_platform_auth::AccountAccessVerification::Expired(_)
        ));
        cases.push((expired, f.csrf.clone(), false));
        let mut wrong = access_claims.clone();
        wrong["auth_time"] = json!(0);
        cases.push((sign_proof_claims(&wrong, &f.key), f.csrf.clone(), false));
        let mut wrong = proof_claims.clone();
        wrong["security_generation"] = json!((claim_generation(&proof_claims) + 1).to_string());
        cases.push((f.access.clone(), sign_proof_claims(&wrong, &f.key), true));
        let mut expired = proof_claims;
        expired["iat"] = json!(at - 10);
        expired["nbf"] = json!(at - 10);
        expired["exp"] = json!(at);
        cases.push((f.access.clone(), sign_proof_claims(&expired, &f.key), true));
        assert_eq!(cases.len(), 12);
        for (access, csrf, proof_error) in cases {
            let result = prepare_signed(&f, Uuid::new_v4(), &access, &csrf).await;
            assert!(if proof_error {
                matches!(result, Err(AccountOperationError::CsrfInvalid))
            } else {
                matches!(result, Err(AccountOperationError::AuthenticationInvalid))
            });
            assert!(
                before == all_rows(&pool).await,
                "signed refusal changed complete history"
            );
        }
        f.business.close().await;
        f.startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_signed_real_logout_revocation_denies_retained_access_and_proof(pool: PgPool) {
        let f = fixture(&pool, true).await;
        let before = all_rows(&pool).await;
        assert!(matches!(
            prepare_signed(&f, Uuid::new_v4(), &f.access, &f.csrf).await,
            Err(AccountOperationError::AuthenticationInvalid)
        ));
        assert!(before == all_rows(&pool).await);
        f.business.close().await;
        f.startup.close().await;
    }

    #[sqlx::test(migrations = false)]
    async fn company_signed_expiry_after_material_wait_and_after_owner_rolls_back(pool: PgPool) {
        let f = fixture(&pool, false).await;
        for after_owner in [false, true] {
            for proof_expires in [false, true] {
                let before = all_rows(&pool).await;
                let command = Uuid::new_v4();
                let input = bytes(f.account, command, f.account);
                let expires = if after_owner {
                    now(&pool).await.unix_timestamp() + 3
                } else {
                    // Real integer-second JWT deadline, chosen with ~650-750ms left.
                    // Release after expiry stays inside the SQL owner's fixed1s wait.
                    tokio::time::timeout(std::time::Duration::from_secs(2), async {
                        loop {
                            let at = now(&pool).await;
                            if (250_000_000..=350_000_000).contains(&at.nanosecond()) {
                                break at.unix_timestamp() + 1;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                        }
                    })
                    .await
                    .unwrap()
                };
                let mut access = f.access.clone();
                let mut csrf = f.csrf.clone();
                let token = if proof_expires {
                    &mut csrf
                } else {
                    &mut access
                };
                let mut claims = signed_claims(token, &f.key).unwrap();
                claims["exp"] = json!(expires);
                *token = sign_proof_claims(&claims, &f.key);
                let mut tx = f.business.begin().await.unwrap();
                sqlx::raw_sql("SET LOCAL lock_timeout='1s'; SET LOCAL statement_timeout='5s'")
                    .execute(tx.as_mut())
                    .await
                    .unwrap();
                let credentials =
                    AccountEnrollmentCredentials::for_mutation(&access, &csrf).unwrap();
                assert_eq!(
                    credentials
                        .account_id_in_tx(&mut tx, &f.verifier, f.ttl)
                        .await
                        .unwrap(),
                    f.account
                );
                if after_owner {
                    let mut guard = credentials
                        .lock_submit_in_tx(&mut tx, &f.verifier, f.ttl, command, &input)
                        .await
                        .unwrap();
                    let rows:Vec<(String,Option<Uuid>)>=sqlx::query_as("SELECT state,receipt_id FROM public.company_enrollment_prepare_v1($1,$2,$3,$4)").bind(f.account).bind(f.session).bind(command).bind(&input).fetch_all(guard.connection()).await.unwrap();
                    assert_eq!(rows, vec![("PENDING".into(), None)]);
                    let reached:Vec<(String,Option<Vec<u8>>)>=sqlx::query_as("SELECT state,input_bytes FROM public.company_enrollment_status_v1($1,$2,$3)").bind(f.account).bind(f.session).bind(command).fetch_all(guard.connection()).await.unwrap();
                    assert_eq!(reached, vec![("PENDING".into(), Some(input.clone()))]);
                    sqlx::query("SELECT pg_sleep(GREATEST(0.0,$1::double precision-extract(epoch FROM clock_timestamp()))+0.025)").bind(expires as f64).execute(guard.connection()).await.unwrap();
                    let result = guard.finish().await;
                    assert!(if proof_expires {
                        matches!(result, Err(AccountOperationError::CsrfInvalid))
                    } else {
                        matches!(result, Err(AccountOperationError::AuthenticationInvalid))
                    });
                } else {
                    let mut blocker = pool.begin().await.unwrap();
                    let holder: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                        .fetch_one(blocker.as_mut())
                        .await
                        .unwrap();
                    sqlx::query("SELECT * FROM public.account_security_lock_exclusive_v1($1)")
                        .bind(f.account)
                        .fetch_one(blocker.as_mut())
                        .await
                        .unwrap();
                    let waiter: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
                        .fetch_one(tx.as_mut())
                        .await
                        .unwrap();
                    let observe = async {
                        let reached=tokio::time::timeout(std::time::Duration::from_millis(500),async {
                        loop {let waiting:bool=sqlx::query_scalar("SELECT $2=ANY(pg_blocking_pids($1)) AND extract(epoch FROM clock_timestamp()) < $3::bigint").bind(waiter).bind(holder).bind(expires).fetch_one(&pool).await.unwrap();if waiting {break;}tokio::time::sleep(std::time::Duration::from_millis(5)).await;}
                    }).await.is_ok();
                        if reached {
                            sqlx::query("SELECT pg_sleep(GREATEST(0.0,$1::double precision-extract(epoch FROM clock_timestamp()))+0.025)").bind(expires as f64).execute(&pool).await.unwrap();
                        }
                        blocker.rollback().await.unwrap();
                        reached
                    };
                    let acquire = async {
                        let result = credentials
                            .lock_submit_in_tx(&mut tx, &f.verifier, f.ttl, command, &input)
                            .await;
                        match result {
                            Ok(guard) => {
                                drop(guard);
                                false
                            }
                            Err(error) => {
                                if proof_expires {
                                    matches!(error, AccountOperationError::CsrfInvalid)
                                } else {
                                    matches!(error, AccountOperationError::AuthenticationInvalid)
                                }
                            }
                        }
                    };
                    let (refused, reached) =
                        tokio::time::timeout(std::time::Duration::from_secs(12), async {
                            tokio::join!(acquire, observe)
                        })
                        .await
                        .unwrap();
                    assert!(
                        reached && refused,
                        "actual material lockwait before expiry and exact Auth refusal required"
                    );
                }
                tx.rollback().await.unwrap();
                assert!(
                    before == all_rows(&pool).await,
                    "expired proof left request/event or other history"
                );
            }
        }
        f.business.close().await;
        f.startup.close().await;
    }

    #[test]
    fn company_signed_capture_bounds_are_exact_without_claiming_authentication() {
        // Raw capture success is deliberately not signature/current authority.
        let access = "a".repeat(16 * 1024);
        let proof = "p".repeat(4 * 1024);
        assert!(AccountEnrollmentCredentials::for_read(&access).is_ok());
        assert!(AccountEnrollmentCredentials::for_mutation(&access, &proof).is_ok());
        for candidate in [String::new(), "a".repeat(16 * 1024 + 1)] {
            assert!(matches!(
                AccountEnrollmentCredentials::for_read(&candidate),
                Err(AccountOperationError::AuthenticationInvalid)
            ));
            assert!(matches!(
                AccountEnrollmentCredentials::for_mutation(&candidate, &proof),
                Err(AccountOperationError::AuthenticationInvalid)
            ));
        }
        for candidate in [String::new(), "p".repeat(4 * 1024 + 1)] {
            assert!(matches!(
                AccountEnrollmentCredentials::for_mutation(&access, &candidate),
                Err(AccountOperationError::CsrfInvalid)
            ));
        }
    }

    #[sqlx::test(migrations = false)]
    async fn company_signed_nonpositive_configured_ttl_never_acquires_authority(pool: PgPool) {
        let f = fixture(&pool, false).await;
        let before = all_rows(&pool).await;
        let command = Uuid::new_v4();
        let input = bytes(f.account, command, f.account);
        let mutation = AccountEnrollmentCredentials::for_mutation(&f.access, &f.csrf).unwrap();
        let read = AccountEnrollmentCredentials::for_read(&f.access).unwrap();
        for ttl in [time::Duration::ZERO, time::Duration::seconds(-1)] {
            let mut tx = f.business.begin().await.unwrap();
            assert!(matches!(
                mutation.account_id_in_tx(&mut tx, &f.verifier, ttl).await,
                Err(AccountOperationError::AuthorityUnavailable)
            ));
            assert!(matches!(
                read.account_id_in_tx(&mut tx, &f.verifier, ttl).await,
                Err(AccountOperationError::AuthorityUnavailable)
            ));
            assert!(matches!(
                mutation
                    .lock_submit_in_tx(&mut tx, &f.verifier, ttl, command, &input)
                    .await,
                Err(AccountOperationError::AuthorityUnavailable)
            ));
            assert!(matches!(
                read.lock_status_in_tx(&mut tx, &f.verifier, ttl, command)
                    .await,
                Err(AccountOperationError::AuthorityUnavailable)
            ));
            assert!(matches!(
                mutation
                    .lock_cancel_in_tx(&mut tx, &f.verifier, ttl, command)
                    .await,
                Err(AccountOperationError::AuthorityUnavailable)
            ));
            tx.rollback().await.unwrap();
            assert!(before == all_rows(&pool).await);
        }
        f.business.close().await;
        f.startup.close().await;
    }
}
