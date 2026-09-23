// Include inside existing native_policy::tests. Uses its capture-only fixtures.
// These tests do not claim signed CSRF validation, current authority or writes.
// capture_post's proposed result wraps existing Input in DocumentInput::Ready.

fn invalid_expiry_body() -> String {
    body("grant").replace("2028-02-29T00%3A15", "2027-02-29T00%3A15")
}

#[tokio::test]
async fn invalid_expiry_capture_retains_original_typed_draft_after_post_admission() {
    let auth = auth();
    let (_, input) = capture_post(
        &auth,
        request("grant", invalid_expiry_body()),
        company(),
        target("grant"),
    )
    .await
    .unwrap_or_else(|_| panic!("recoverable expiry lost at transport boundary"));
    match input {
        super::form::DocumentInput::GrantValidation(draft) => {
            assert_eq!(draft.selector, selector(NativeBusinessOperationV1::Grant));
            assert_eq!(draft.expected_company_epoch, 2);
            assert_eq!(*draft.recipient_account_id.as_uuid(), id(RECIPIENT));
            assert!(draft.assignment.is_none());
            assert_eq!(draft.expires_at_local, "2027-02-29T00:15");
        }
        _ => panic!("invalid expiry became executable input"),
    }
}

#[tokio::test]
async fn invalid_expiry_does_not_bypass_original_post_metadata_checks() {
    let auth = auth();
    let mut method = request("grant", invalid_expiry_body());
    *method.method_mut() = Method::GET;
    refused(
        &auth,
        company(),
        target("grant"),
        method,
        StatusCode::BAD_REQUEST,
    )
    .await;
    for (name, value, status) in [
        (
            "origin",
            "https://attacker.example.com",
            StatusCode::FORBIDDEN,
        ),
        ("content-type", "application/json", StatusCode::BAD_REQUEST),
        (
            "content-type",
            "application/x-www-form-urlencoded; charset=utf-8",
            StatusCode::BAD_REQUEST,
        ),
        ("sec-fetch-site", "cross-site", StatusCode::FORBIDDEN),
        ("sec-fetch-mode", "cors", StatusCode::FORBIDDEN),
        ("sec-fetch-dest", "iframe", StatusCode::FORBIDDEN),
        (
            "x-console-csrf",
            "replacement-proof",
            StatusCode::BAD_REQUEST,
        ),
    ] {
        let mut req = request("grant", invalid_expiry_body());
        req.headers_mut()
            .insert(name, HeaderValue::from_str(value).unwrap());
        refused(&auth, company(), target("grant"), req, status).await;
    }
    for (header, status) in [
        ("origin", StatusCode::FORBIDDEN),
        ("sec-fetch-dest", StatusCode::BAD_REQUEST),
    ] {
        let mut req = request("grant", invalid_expiry_body());
        req.headers_mut().remove(header);
        refused(&auth, company(), target("grant"), req, status).await;
    }
    // Preserve the existing allowed all-metadata-absent browser compatibility.
    let mut absent = request("grant", invalid_expiry_body());
    for name in ["sec-fetch-site", "sec-fetch-mode", "sec-fetch-dest"] {
        absent.headers_mut().remove(name);
    }
    assert!(
        capture_post(&auth, absent, company(), target("grant"))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn invalid_expiry_does_not_replace_missing_session_or_body_proof() {
    let auth = auth();
    let mut absent = request("grant", invalid_expiry_body());
    absent.headers_mut().remove(header::COOKIE);
    refused(
        &auth,
        company(),
        target("grant"),
        absent,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    let mut refresh = request("grant", invalid_expiry_body());
    refresh.headers_mut().insert(
        header::COOKIE,
        HeaderValue::from_static("__Host-console_account_refresh=capture-only-refresh"),
    );
    refused(
        &auth,
        company(),
        target("grant"),
        refresh,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    let empty_proof = invalid_expiry_body().replace(&format!("csrf_proof={PROOF}"), "csrf_proof=");
    refused(
        &auth,
        company(),
        target("grant"),
        request("grant", empty_proof),
        StatusCode::BAD_REQUEST,
    )
    .await;
    let two_proofs = format!("{}&csrf_proof={PROOF}", invalid_expiry_body());
    refused(
        &auth,
        company(),
        target("grant"),
        request("grant", two_proofs),
        StatusCode::BAD_REQUEST,
    )
    .await;
}

#[tokio::test]
async fn invalid_expiry_structural_or_size_failure_stays_neutral() {
    let auth = auth();
    for malformed in [
        format!("{}&unexpected=field", invalid_expiry_body()),
        invalid_expiry_body().replace(COMMAND, "not-a-command"),
        invalid_expiry_body().replace("expected_company_epoch=2", "expected_company_epoch=0"),
        invalid_expiry_body().replace("expected_role_revision=", "expected_role_revision=2"),
        invalid_expiry_body().replace("2027-02-29T00%3A15", "bad%00date"),
    ] {
        refused(
            &auth,
            company(),
            target("grant"),
            request("grant", malformed),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    let too_long = invalid_expiry_body().replace("2027-02-29T00%3A15", &"x".repeat(65));
    refused(
        &auth,
        company(),
        target("grant"),
        request("grant", too_long),
        StatusCode::PAYLOAD_TOO_LARGE,
    )
    .await;
}
