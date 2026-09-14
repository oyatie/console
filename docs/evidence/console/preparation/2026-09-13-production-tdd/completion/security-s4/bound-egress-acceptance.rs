//! Actual projection -> ordinary admitted storage binding -> actual HTTP client.
//! All receivers are isolated loopback peers and credentials conspicuously fake.
use crate::{
    local_egress_peer::Peer,
    native_fixture::{TestResult, build_native_deployment},
    security_grant_producer::replace_projection,
};
use console_payroll_adapter_postgres::action30 as payroll;
use console_platform_storage::bound_egress as egress;
use serde_json::json;
use sqlx::PgPool;
#[sqlx::test(migrations = false)]
async fn bound_egress_checks_each_destination_identity_before_any_actual_outbound_request(
    pool: PgPool,
) -> TestResult {
    let d = build_native_deployment(pool, "bound-egress", 1).await?;
    let f = &d.companies[0];
    let (run, _) = f.calculated().await?;
    let a = Peer::start().await?;
    let b = Peer::start().await?;
    // Same ordinary provider-custody owner used for object storage deployment.
    // Registration authenticates operator, exact local TEST_ONLY environment and
    // current purpose; the returned binding is opaque and cannot be deserialized.
    let proposal = egress::ProviderBindingProposal {
        environment: egress::Environment::TestOnly,
        endpoint: a.origin.clone(),
        provider_id: "test.object.a".into(),
        credential_id: "test.credential.a".into(),
        credential: egress::Secret::new("TEST_ONLY_A"),
        recipient_account_id: d.accounts.submitter.account_id,
        purpose: "protected.object.download".into(),
    };
    let binding =
        egress::register_provider_binding(&d.runtime.config, &d.operator, proposal).await?;
    let projection =
        payroll::read_native_authorized_export(&d.pool, &f.submitter, run.run_id, &binding).await?;
    let actual_bytes = console_ontology_rest::projection::encode_authorized_export(&projection)?;
    assert!(std::str::from_utf8(&actual_bytes)?.contains("3000000"));
    let choices = binding.untrusted_destination_selection();
    let client = egress::BoundTransport::from_deployment(&d.runtime.config).await?;
    for dimension in [
        "scheme",
        "host",
        "port",
        "provider",
        "credential",
        "recipient",
        "purpose",
    ] {
        let mut altered = serde_json::to_value(&choices)?;
        match dimension {
            "scheme" => altered["scheme"] = json!("https"),
            "host" => altered["host"] = json!("localhost"),
            "port" => altered["port"] = json!(b.origin.port()),
            "provider" => altered["provider_id"] = json!("test.object.b"),
            "credential" => altered["credential_id"] = json!("test.credential.b"),
            "recipient" => altered["recipient_account_id"] = json!(d.accounts.reviewer.account_id),
            "purpose" => altered["purpose"] = json!("public.csv.export"),
            _ => unreachable!(),
        }
        let selected = egress::DestinationSelection::parse(&serde_json::to_vec(&altered)?)?;
        let refused =
            egress::admit_bound_egress(&d.pool, &f.submitter, &projection, &binding, selected)
                .await;
        assert!(
            matches!(refused, Err(egress::EgressError::BindingMismatch)),
            "wrong binding classified independently of transport"
        );
        assert!(a.requests.lock().await.is_empty());
        assert!(b.requests.lock().await.is_empty());
    }
    let admitted =
        egress::admit_bound_egress(&d.pool, &f.submitter, &projection, &binding, choices).await?;
    tokio::time::timeout(std::time::Duration::from_secs(10), client.send(admitted)).await??;
    let sent = a.requests.lock().await.clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].body, actual_bytes);
    // A real grant revocation after projection but before egress admission blocks.
    replace_projection(
        &d,
        0,
        d.accounts.submitter.account_id,
        &[],
        "native30.standard",
    )
    .await?;
    assert!(matches!(
        egress::admit_bound_egress(
            &d.pool,
            &f.submitter,
            &projection,
            &binding,
            binding.untrusted_destination_selection()
        )
        .await,
        Err(egress::EgressError::StaleContext) | Err(egress::EgressError::PermissionDenied)
    ));
    assert_eq!(a.finish().await?.len(), 1);
    assert!(b.finish().await?.is_empty());
    Ok(())
}
