// Companion-only checkpoints. Original Account/Company owners and full history stay checked.
async fn verify_controller_events(
    pool: &PgPool,
    input: &mut tokio::process::ChildStdin,
    events: &mut tokio::io::BufReader<tokio::process::ChildStdout>,
    operator: Uuid,
    administrator: Uuid,
    original_company: Uuid,
    original_command: Uuid,
    original_rows: &BTreeMap<String, String>,
    checkpoints: &mut Vec<&'static str>,
) {
    let mut current = original_rows.clone();
    for name in [
        "native-first",
        "react-first",
        "keyboard-before-react",
        "autofill-before-react",
        "eventless-value-before-react",
        "ime-before-react",
        "preclaim-focus",
        "partial-bind-rollback",
        "capability-denied",
        "freeze-at-csrf",
    ] {
        let ready = event(events).await;
        exact_keys(&ready, &["kind", "phase", "control_name", "account_id"]);
        assert!(
            ready["kind"] == "CHECKPOINT"
                && ready["phase"] == "CONTROLLER_READY"
                && ready["control_name"] == name
                && ready["account_id"] == json!(operator)
        );
        assert!(
            current == all_rows(pool).await,
            "control entry changed any previous public bytes"
        );
        checkpoints.push("CONTROLLER_READY");
        browser_owner_continue(input, "CONTROLLER_READY").await;
        if name == "freeze-at-csrf" {
            let undispatched = event(events).await;
            exact_keys(
                &undispatched,
                &["kind", "phase", "control_name", "account_id"],
            );
            assert!(
                undispatched["kind"] == "CHECKPOINT"
                    && undispatched["phase"] == "CONTROLLER_PRE_DISPATCH"
                    && undispatched["control_name"] == name
                    && undispatched["account_id"] == json!(operator)
            );
            assert!(
                current == all_rows(pool).await,
                "CSRF preparation dispatched or changed durable input"
            );
            checkpoints.push("CONTROLLER_PRE_DISPATCH");
            browser_owner_continue(input, "CONTROLLER_PRE_DISPATCH").await;
            let created = event(events).await;
            exact_keys(
                &created,
                &[
                    "kind",
                    "phase",
                    "control_name",
                    "account_id",
                    "administrator_account_id",
                    "command_id",
                    "org_id",
                    "group_id",
                    "receipt_id",
                ],
            );
            assert!(
                created["kind"] == "CHECKPOINT"
                    && created["phase"] == "CONTROLLER_COMMITTED"
                    && created["control_name"] == name
                    && created["account_id"] == json!(operator)
                    && created["administrator_account_id"] == json!(administrator)
            );
            let id = |key: &str| {
                let text = created[key].as_str().unwrap();
                let id = Uuid::parse_str(text).unwrap();
                assert!(!id.is_nil() && id.to_string() == text);
                id
            };
            let command = id("command_id");
            let company = id("org_id");
            assert!(command != original_command && company != original_company);
            let result = Committed {
                command,
                receipt: id("receipt_id"),
                company,
                group: id("group_id"),
                administrator,
                result_path: format!("/account/companies/requests/{command}"),
            };
            let mut submitted = enrollment(command, administrator);
            submitted["name"] = json!("고정된 관리자 회사 <원본 & 입력>");
            submitted["slug"] = json!(format!("controller-{}", operator.simple()));
            durable(pool, &result, &submitted, operator).await;
            let after = all_rows(pool).await;
            assert_controller_company_effects(
                pool, &current, &after, &result, &submitted, operator,
            )
            .await;
            current = after;
            checkpoints.push("CONTROLLER_COMMITTED");
            browser_owner_continue(input, "CONTROLLER_COMMITTED").await;
        }
        let checked = event(events).await;
        exact_keys(&checked, &["kind", "phase", "control_name", "account_id"]);
        assert!(
            checked["kind"] == "CHECKPOINT"
                && checked["phase"] == "CONTROLLER_CHECKED"
                && checked["control_name"] == name
                && checked["account_id"] == json!(operator)
        );
        assert!(
            current == all_rows(pool).await,
            "control/reopening changed any public bytes"
        );
        checkpoints.push("CONTROLLER_CHECKED");
        browser_owner_continue(input, "CONTROLLER_CHECKED").await;
    }
}
