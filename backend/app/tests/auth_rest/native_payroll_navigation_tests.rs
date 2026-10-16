// Real mounted navigation uses native Account/Company/policy owners. This is
// transport/DB evidence; populated browser input has a separate real UI journey.
mod native_payroll_navigation_tests {
    use super::*;

    fn payroll_anchor(body: &str, href: &str) -> usize {
        body.split("<a ")
            .skip(1)
            .filter(|part| {
                let Some((opening, rest)) = part.split_once('>') else {
                    return false;
                };
                let Some((label, _)) = rest.split_once("</a>") else {
                    return false;
                };
                let attributes: Vec<_> = opening.split_ascii_whitespace().collect();
                attributes.contains(&href)
                    && !attributes.contains(&"hidden")
                    && !attributes.contains(&"aria-hidden=\"true\"")
                    && label == "급여"
            })
            .count()
    }
    #[test]
    fn navigation_anchor_requires_same_target_and_label() {
        let href = "href=\"/companies/actual/payroll\"";
        assert_eq!(
            payroll_anchor("<a href=\"/companies/actual/payroll\">급여</a>", href),
            1
        );
        for body in [
            "<span href=\"/companies/actual/payroll\"></span><a href=\"/wrong\">급여</a>",
            "<a href=\"/companies/actual/payroll\">권한</a><a href=\"/wrong\">급여</a>",
            "<a hidden href=\"/companies/actual/payroll\">급여</a>",
        ] {
            assert_eq!(payroll_anchor(body, href), 0);
        }
    }
    async fn navigation_page(
        pool: &PgPool,
        f: &FixtureRead,
        path: &str,
        cookies: &Cookies,
        status: StatusCode,
        visible: bool,
    ) {
        let before = all_rows(pool).await;
        let started = OffsetDateTime::now_utc();
        let response = document(&f.app, path, cookies).await;
        let finished = OffsetDateTime::now_utc();
        assert_eq!(response.status, status);
        response.private();
        let body = String::from_utf8(response.bytes).unwrap();
        let href = format!("href=\"/companies/{}/payroll\"", f.created.company);
        assert_eq!(
            payroll_anchor(&body, &href),
            usize::from(visible),
            "Payroll navigation must match the current viewer, at {path}"
        );
        if !visible {
            assert!(!body.contains(&href));
        }
        let after = all_rows(pool).await;
        if path.ends_with("/policy/payroll-read/install") {
            assert!(
                policy_preflight_effects(&before, &after, started, finished),
                "preflight may change only its exact existing proof limiter buckets"
            );
        } else {
            assert!(
                before == after,
                "navigation must not write a list audit or another business effect"
            );
        }
    }

    #[sqlx::test(migrations = false)]
    async fn company_workspace_discovers_payroll_only_after_current_grant(pool: PgPool) {
        let f = FixtureRead::new(&pool).await;
        let result = AssertUnwindSafe(async {
            let company = format!("/companies/{}", f.created.company);
            navigation_page(&pool, &f, &company, &f.cookies, StatusCode::OK, false).await;
            f.install().await;
            navigation_page(&pool, &f, &company, &f.cookies, StatusCode::OK, false).await;
            let grant_id = Uuid::new_v4();
            let now: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool)
                .await
                .unwrap();
            let until =
                time::OffsetDateTime::from_unix_timestamp((now.unix_timestamp() / 60 + 1440) * 60)
                    .unwrap();
            let grant = f
                .command(
                    NativeCompanyBusinessCommandV1::grant(
                        grant_id,
                        f.company(),
                        2,
                        AccountId::from_uuid(f.created.administrator).unwrap(),
                        None,
                        until,
                    )
                    .unwrap(),
                )
                .await;
            let assignment = match grant.outcome {
                NativePolicyOutcome::Committed(NativePolicyEffect::Granted {
                    assignment, ..
                }) => assignment.expectation,
                _ => panic!("real grant prerequisite failed"),
            };
            // This is the named RED: the current mounted workspace lacks its link.
            navigation_page(&pool, &f, &company, &f.cookies, StatusCode::OK, true).await;
            let receipt = format!("{company}/policy/payroll-read/requests/grant/{grant_id}");
            let preflight = format!("{company}/policy/payroll-read/install");
            for path in [&receipt, &preflight] {
                navigation_page(&pool, &f, path, &f.cookies, StatusCode::OK, true).await;
            }
            // A distinct, genuinely enrolled Account cannot borrow the recipient's grant.
            let (_, foreign) = enrolled(&f.app).await;
            for path in [&company, &receipt, &preflight] {
                navigation_page(&pool, &f, path, &foreign, StatusCode::NOT_FOUND, false).await;
            }
            // Rendering a workspace must not query payroll rows then throw them away.
            let before = all_rows(&pool).await;
            let mut blocker = pool.begin().await.unwrap();
            sqlx::query("LOCK TABLE public.payroll_draft_runs IN ACCESS EXCLUSIVE MODE")
                .execute(blocker.as_mut())
                .await
                .unwrap();
            let response = document(&f.app, &company, &f.cookies).await;
            blocker.rollback().await.unwrap();
            assert_eq!(response.status, StatusCode::OK);
            assert_eq!(
                payroll_anchor(
                    &String::from_utf8(response.bytes).unwrap(),
                    &format!("href=\"{company}/payroll\"")
                ),
                1
            );
            assert!(before == all_rows(&pool).await);
            let before = all_rows(&pool).await;
            for path in [&company, &receipt, &preflight] {
                let response = request(
                    &f.app,
                    "HEAD",
                    path,
                    &f.cookies,
                    None,
                    &[
                        ("Sec-Fetch-Mode", "navigate"),
                        ("Sec-Fetch-Dest", "document"),
                    ],
                )
                .await;
                assert_eq!(response.status, StatusCode::METHOD_NOT_ALLOWED);
                assert!(response.bytes.is_empty());
            }
            assert!(before == all_rows(&pool).await);
            let revoked = f
                .command(
                    NativeCompanyBusinessCommandV1::revoke(
                        Uuid::new_v4(),
                        f.company(),
                        3,
                        assignment,
                    )
                    .unwrap(),
                )
                .await;
            assert!(matches!(
                revoked.outcome,
                NativePolicyOutcome::Committed(NativePolicyEffect::Revoked { .. })
            ));
            for path in [&company, &receipt, &preflight] {
                navigation_page(&pool, &f, path, &f.cookies, StatusCode::OK, false).await;
            }
            let csrf = proof(&f.app, &f.cookies).await;
            let response = request(
                &f.app,
                "POST",
                "/api/v2/auth/logout",
                &f.cookies,
                Some(json!({})),
                &[("X-Console-CSRF", &csrf)],
            )
            .await;
            assert_eq!(
                response.json(StatusCode::OK),
                json!({"outcome":"COMMITTED"})
            );
            navigation_page(
                &pool,
                &f,
                &company,
                &f.cookies,
                StatusCode::UNAUTHORIZED,
                false,
            )
            .await;
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
    #[sqlx::test(migrations = false)]
    async fn visible_policy_operator_cannot_borrow_distinct_recipients_payroll_navigation(
        pool: PgPool,
    ) {
        let (app, key, state) = configured_fixture(&pool, true).await;
        let (app, operator, cookies, startup, _) = designated_fixture(&pool, app).await;
        let (admin, admin_cookies) = enrolled(&app).await;
        assert_ne!(operator.account, admin.account);
        let create = Uuid::new_v4();
        let input = enrollment(create, admin.account);
        let csrf = proof(&app, &cookies).await;
        let created = committed(
            &submit(&app, &cookies, &csrf, &input).await,
            StatusCode::CREATED,
            create,
            admin.account,
            false,
        );
        durable(&pool, &created, &input, operator.account).await;
        startup.close().await;
        let config = account_browser_config(&pool, app._artifacts.root.clone(), &key);
        let (verifier, issuer, ttl) = bindings(&config);
        let runtime = login_test_pool(&pool, TestDatabaseLogin::Business).await;
        let store = PgOrgStore::new(runtime.clone()).with_native_account_policy(
            verifier.clone(),
            issuer,
            ttl,
        );
        let f = FixtureRead {
            app,
            state,
            runtime,
            store,
            verifier,
            ttl,
            cookies,
            created,
            policy: Arc::new(CompanyPolicy::new().unwrap()),
        };
        let result = AssertUnwindSafe(async {
            f.install().await;
            let id = Uuid::new_v4();
            let now: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
                .fetch_one(&pool)
                .await
                .unwrap();
            let until =
                OffsetDateTime::from_unix_timestamp((now.unix_timestamp() / 60 + 1440) * 60)
                    .unwrap();
            let terminal = f
                .command(
                    NativeCompanyBusinessCommandV1::grant(
                        id,
                        f.company(),
                        2,
                        AccountId::from_uuid(admin.account).unwrap(),
                        None,
                        until,
                    )
                    .unwrap(),
                )
                .await;
            assert!(matches!(
                terminal.outcome,
                NativePolicyOutcome::Committed(NativePolicyEffect::Granted { .. })
            ));
            let company = format!("/companies/{}", f.created.company);
            // Positive, visible recipient; same live Company and actual grant.
            navigation_page(&pool, &f, &company, &admin_cookies, StatusCode::OK, true).await;
            // Operator CAN read its genuine grant receipt/form, but cannot read Payroll.
            navigation_page(
                &pool,
                &f,
                &format!("{company}/policy/payroll-read/requests/grant/{id}"),
                &f.cookies,
                StatusCode::OK,
                false,
            )
            .await;
            navigation_page(
                &pool,
                &f,
                &format!("{company}/policy/payroll-read/install"),
                &f.cookies,
                StatusCode::OK,
                false,
            )
            .await;
        })
        .catch_unwind()
        .await;
        f.close(result).await;
    }
}
