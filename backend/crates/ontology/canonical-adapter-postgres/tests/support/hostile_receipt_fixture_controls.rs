//! Company-only controls drive the shared helper, never a duplicate protocol.
use super::hostile_receipt_fixture::{
    self as fault, Failure, Fault, InitialGuard, Input, Observed, Phase, Reason,
};
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

async fn accepted(pool: &PgPool) -> (PgCompanyPort, CompanyCommand, Input) {
    let (org, actor, port) = fixture(pool).await;
    let command = command(org, actor, revise("fixture-control"));
    let accepted = execute(&port, command.clone())
        .await
        .expect("canonical producer");
    let input = Input {
        org: ORG,
        command: *command.command_id.as_uuid(),
        actor: *actor.as_uuid(),
        digest: accepted.payload_digest().to_vec(),
        receipt: json!({ "version": 1 }),
        legacy: true,
    };
    (port, command, input)
}

async fn run_then_owner(
    pool: &PgPool,
    port: &PgCompanyPort,
    command: CompanyCommand,
    input: &Input,
    injected: Fault,
    replays: &AtomicUsize,
) -> Result<Result<CommandReceipt, CompanyError>, Failure> {
    fault::run(pool, input, injected).await?;
    replays.fetch_add(1, Ordering::SeqCst);
    Ok(execute(port, command).await)
}

async fn refused(
    pool: &PgPool,
    port: &PgCompanyPort,
    command: &CompanyCommand,
    input: &Input,
    injected: Fault,
    expected: Failure,
) {
    let before = fault::observe(pool).await.expect("baseline observation");
    let replays = AtomicUsize::new(0);
    let result = run_then_owner(pool, port, command.clone(), input, injected, &replays).await;
    assert!(
        matches!(result, Err(actual) if actual == expected),
        "unexpected fixture failure class"
    );
    assert_eq!(
        replays.load(Ordering::SeqCst),
        0,
        "owner replay must be fenced"
    );
    let after = fault::observe(pool)
        .await
        .expect("fresh recovery observation");
    assert!(
        fault::compare(&before, &after).is_ok(),
        "original full rows/table inventory/guards/FKs must be restored"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_no_match_rolls_back_and_fences_owner(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    let mut wrong_actor = input.clone();
    wrong_actor.actor = Uuid::nil();
    refused(
        &pool,
        &port,
        &command,
        &wrong_actor,
        Fault::None,
        Failure::Refused(Reason::Cardinality(0)),
    )
    .await;
    let mut wrong_digest = input;
    wrong_digest.digest[0] ^= 1; // Deliberate mismatch of the real accepted digest, never a fixture digest encoding.
    refused(
        &pool,
        &port,
        &command,
        &wrong_digest,
        Fault::None,
        Failure::Refused(Reason::Cardinality(0)),
    )
    .await;
}

#[test]
fn hostile_fixture_singleton_uses_the_actual_result_boundary() {
    // 2 is a FAKE result-boundary value; the intact real primary key makes it unreachable in SQL.
    assert_eq!(fault::require_singleton(0), Err(Reason::Cardinality(0)));
    assert_eq!(fault::require_singleton(1), Ok(()));
    assert_eq!(fault::require_singleton(2), Err(Reason::Cardinality(2)));
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_refuses_initial_guard_shapes_before_disable(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    for shape in [
        InitialGuard::Disabled,
        InitialGuard::Replica,
        InitialGuard::Conditional,
        InitialGuard::Columns,
        InitialGuard::Arguments,
        InitialGuard::OtherFunction,
        InitialGuard::SecurityDefiner,
        InitialGuard::Stable,
        InitialGuard::Strict,
        InitialGuard::ParallelSafe,
        InitialGuard::Config,
        InitialGuard::Cost,
        InitialGuard::Body,
    ] {
        refused(
            &pool,
            &port,
            &command,
            &input,
            Fault::Initial(shape),
            Failure::Refused(Reason::Guard),
        )
        .await;
    }
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_sql_errors_await_observed_rollback(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    refused(
        &pool,
        &port,
        &command,
        &input,
        Fault::BeforeDisable,
        Failure::Refused(Reason::Sql(Phase::BeforeDisable)),
    )
    .await;
    refused(
        &pool,
        &port,
        &command,
        &input,
        Fault::AfterDisable,
        Failure::Refused(Reason::Sql(Phase::AfterDisable)),
    )
    .await;
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_restore_and_drift_refusals_restore_all_rows_and_catalogs(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    for (injected, reason) in [
        (Fault::Restore, Reason::Sql(Phase::Restore)),
        (Fault::GuardDrift, Reason::CatalogDrift),
        (Fault::RowDrift, Reason::RowDrift),
        (Fault::TableDrift, Reason::TableDrift),
        // FAKE snapshot-transport corruption: actual FK rows/keys/ALWAYS guards are untouched.
        (Fault::ForeignKeySnapshot, Reason::CatalogDrift),
    ] {
        refused(
            &pool,
            &port,
            &command,
            &input,
            injected,
            Failure::Refused(reason),
        )
        .await;
    }
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_restores_owner_update_delete_p0001_and_legacy_branch(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    let replays = AtomicUsize::new(0);
    // run verifies owner UPDATE and DELETE P0001/message, guard shape and all rows/catalogs before returning success.
    let result = run_then_owner(&pool, &port, command, &input, Fault::None, &replays)
        .await
        .expect("restored fixture");
    assert!(
        matches!(result, Err(CompanyError::UnreadableReceipt(id, _)) if id == input.command),
        "legacy missing-target refusal"
    );
    assert_eq!(replays.load(Ordering::SeqCst), 1);
    let binding: (String, Option<String>, Option<String>, Option<Uuid>) = sqlx::query_as("SELECT owner,target,action_key,object_type_id FROM public.ont_action_command_receipts WHERE org_id=$1 AND command_id=$2")
        .bind(input.org).bind(input.command).fetch_one(&pool).await.unwrap();
    assert!(
        binding.0 == "ontology.action"
            && binding.1.is_none()
            && binding.2.is_none()
            && binding.3.is_none(),
        "retain exact legacy default metadata branch"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_canonical_metadata_also_reaches_original_result_target_refusal(
    pool: PgPool,
) {
    let (port, command, mut input) = accepted(&pool).await;
    input.legacy = false;
    let replays = AtomicUsize::new(0);
    let result = run_then_owner(&pool, &port, command, &input, Fault::None, &replays)
        .await
        .expect("restored fixture");
    assert!(
        matches!(result, Err(CompanyError::UnreadableReceipt(id, _)) if id == input.command),
        "canonical missing-target refusal"
    );
    assert_eq!(replays.load(Ordering::SeqCst), 1);
    let binding: (String, Option<String>) = sqlx::query_as("SELECT owner,target FROM public.ont_action_command_receipts WHERE org_id=$1 AND command_id=$2")
        .bind(input.org).bind(input.command).fetch_one(&pool).await.unwrap();
    assert!(
        binding.0 == "company" && binding.1.as_deref() == Some("company.revise"),
        "canonical metadata retained"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_restores_the_original_always_mode(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    // Ordinary receipt guard only, in this test's private database; every NativePeople ALWAYS guard is untouched.
    sqlx::query("ALTER TABLE public.ont_action_command_receipts ENABLE ALWAYS TRIGGER trg_ont_action_command_receipts_immutable")
        .execute(&pool).await.unwrap();
    let replays = AtomicUsize::new(0);
    let result = run_then_owner(&pool, &port, command, &input, Fault::None, &replays)
        .await
        .expect("restored ALWAYS fixture");
    assert!(matches!(result, Err(CompanyError::UnreadableReceipt(id, _)) if id == input.command));
    let mode: String = sqlx::query_scalar("SELECT tgenabled::text FROM pg_trigger WHERE tgrelid='public.ont_action_command_receipts'::regclass AND tgname='trg_ont_action_command_receipts_immutable'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(mode, "A");
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_cancellation_observes_restoration_on_a_fresh_connection(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    let before = fault::observe(&pool).await.expect("baseline observation");
    let (signal, waiting) = mpsc::channel();
    let replays = Arc::new(AtomicUsize::new(0));
    let task_pool = pool.clone();
    let task_replays = replays.clone();
    let task = tokio::spawn(async move {
        run_then_owner(
            &task_pool,
            &port,
            command,
            &input,
            Fault::Pause(signal),
            &task_replays,
        )
        .await
    });
    let disabled = tokio::task::spawn_blocking(move || {
        waiting.recv_timeout(std::time::Duration::from_secs(20))
    })
    .await
    .unwrap();
    // Abort and join even if the handshake fails; never detach a disabled transaction.
    task.abort();
    let joined = task.await;
    let after = fault::observe(&pool)
        .await
        .expect("fresh lock acquisition observes server rollback");
    assert!(disabled.is_ok(), "control must reach disabled phase");
    assert!(
        joined.is_err_and(|error| error.is_cancelled()),
        "disabled fixture task must cancel"
    );
    assert_eq!(replays.load(Ordering::SeqCst), 0);
    assert!(
        fault::compare(&before, &after).is_ok(),
        "Drop is insufficient; fresh original rows/catalogs are required"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_panic_observes_restoration_on_a_fresh_connection(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    let before = fault::observe(&pool).await.expect("baseline observation");
    let replays = Arc::new(AtomicUsize::new(0));
    let task_pool = pool.clone();
    let task_replays = replays.clone();
    let task = tokio::spawn(async move {
        run_then_owner(
            &task_pool,
            &port,
            command,
            &input,
            Fault::Panic,
            &task_replays,
        )
        .await
    });
    let joined = task.await;
    let after = fault::observe(&pool)
        .await
        .expect("fresh lock acquisition observes server rollback");
    assert!(
        joined.is_err_and(|error| error.is_panic()),
        "control must panic while disabled"
    );
    assert_eq!(replays.load(Ordering::SeqCst), 0);
    assert!(
        fault::compare(&before, &after).is_ok(),
        "fresh original rows/catalogs are required"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_missing_rollback_acknowledgement_fences_owner_even_after_observation(
    pool: PgPool,
) {
    let (port, command, input) = accepted(&pool).await;
    // Injected missing acknowledgement; no real network or server fault is claimed.
    refused(
        &pool,
        &port,
        &command,
        &input,
        Fault::RollbackAckLost,
        Failure::FencedRollback(Observed::Original),
    )
    .await;
    // Fake observer-response loss drives the real unknown branch; this control independently observes recovery.
    refused(
        &pool,
        &port,
        &command,
        &input,
        Fault::RollbackObservationLost,
        Failure::FencedRollback(Observed::Unknown),
    )
    .await;
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_missing_commit_acknowledgement_fences_owner_and_reports_intended_state(
    pool: PgPool,
) {
    let (port, command, input) = accepted(&pool).await;
    let before = fault::observe(&pool).await.expect("baseline observation");
    let replays = AtomicUsize::new(0);
    // Injected lost acknowledgement AFTER a real commit: original rows must not be claimed restored.
    let result = run_then_owner(
        &pool,
        &port,
        command,
        &input,
        Fault::CommitAckLost,
        &replays,
    )
    .await;
    assert!(
        matches!(result, Err(Failure::FencedCommit(Observed::Intended))),
        "uncertain commit must fence replay"
    );
    assert_eq!(replays.load(Ordering::SeqCst), 0);
    let after = fault::observe(&pool)
        .await
        .expect("fresh committed-state observation");
    assert!(
        fault::compare(&before, &after).is_err(),
        "committed intended mutation differs from original"
    );
    let binding: (String, Option<String>, serde_json::Value) = sqlx::query_as("SELECT owner,target,receipt FROM public.ont_action_command_receipts WHERE org_id=$1 AND command_id=$2")
        .bind(input.org).bind(input.command).fetch_one(&pool).await.unwrap();
    assert!(
        binding.0 == "ontology.action" && binding.1.is_none() && binding.2 == input.receipt,
        "observe intended committed receipt without logging it"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_unknown_commit_observation_never_returns_replay_permission(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    let replays = AtomicUsize::new(0);
    // Fake observer-response loss after a real commit; separately observe actual state without relabeling it original.
    let result = run_then_owner(
        &pool,
        &port,
        command,
        &input,
        Fault::CommitObservationLost,
        &replays,
    )
    .await;
    assert!(
        matches!(result, Err(Failure::FencedCommit(Observed::Unknown))),
        "unknown outcome must fence replay"
    );
    assert_eq!(replays.load(Ordering::SeqCst), 0);
    fault::observe(&pool)
        .await
        .expect("independent observation must complete after the fenced result");
    let receipt: serde_json::Value = sqlx::query_scalar(
        "SELECT receipt FROM public.ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
    )
    .bind(input.org)
    .bind(input.command)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        receipt == input.receipt,
        "actual committed receipt is intended, never an inferred restoration"
    );
}

async fn precision_table(pool: &PgPool) {
    // A private control table, not a product migration or an existing constraint/guard change.
    sqlx::query("CREATE TABLE public.hostile_receipt_precision_control(id integer PRIMARY KEY,amount numeric NOT NULL,document jsonb NOT NULL)")
        .execute(pool).await.unwrap();
    sqlx::query(r#"INSERT INTO public.hostile_receipt_precision_control VALUES(1,0.1,'{"number":0.1}'::jsonb)"#)
        .execute(pool).await.unwrap();
    // Demonstrate the actual current Value decoder collision; the snapshot must not use it for full rows.
    let short: serde_json::Value = serde_json::from_str("0.1").unwrap();
    let precise: serde_json::Value = serde_json::from_str("0.10000000000000000001").unwrap();
    assert!(
        short == precise,
        "these distinct PostgreSQL decimals collide under the pinned Value decoder"
    );
    let short_json: serde_json::Value = serde_json::from_str(r#"{"number":0.1}"#).unwrap();
    let precise_json: serde_json::Value =
        serde_json::from_str(r#"{"number":0.10000000000000000001}"#).unwrap();
    assert!(
        short_json == precise_json,
        "the same collision occurs inside JSONB-shaped values"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_precision_drift_is_refused_by_the_actual_snapshot(pool: PgPool) {
    let (port, command, input) = accepted(&pool).await;
    precision_table(&pool).await;
    // Both real SQL mutations run inside the actual helper, after restoration, before its full-row comparison.
    refused(
        &pool,
        &port,
        &command,
        &input,
        Fault::NumericPrecisionDrift,
        Failure::Refused(Reason::RowDrift),
    )
    .await;
    refused(
        &pool,
        &port,
        &command,
        &input,
        Fault::JsonbPrecisionDrift,
        Failure::Refused(Reason::RowDrift),
    )
    .await;
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_precision_classification_cannot_mislabel_original_or_intended(
    pool: PgPool,
) {
    let (_port, _command, input) = accepted(&pool).await;
    precision_table(&pool).await;
    let original = fault::observe(&pool)
        .await
        .expect("lossless original snapshot");
    sqlx::query("UPDATE public.hostile_receipt_precision_control SET amount=0.10000000000000000001 WHERE id=1")
        .execute(&pool).await.unwrap();
    let drift = fault::observe(&pool)
        .await
        .expect("lossless numeric-drift snapshot");
    assert_eq!(
        fault::classified(&original, None, &Ok(drift)),
        Observed::Unknown,
        "numeric drift that rounds equal must never classify Original"
    );
    sqlx::query("UPDATE public.hostile_receipt_precision_control SET amount=0.1 WHERE id=1")
        .execute(&pool)
        .await
        .unwrap();
    fault::run(&pool, &input, Fault::None)
        .await
        .expect("real selected-row projection and restored guard");
    let intended = fault::observe(&pool)
        .await
        .expect("lossless intended snapshot");
    assert_eq!(
        fault::classified(&original, Some(&intended), &Ok(intended.clone())),
        Observed::Intended
    );
    sqlx::query(r#"UPDATE public.hostile_receipt_precision_control SET document='{"number":0.10000000000000000001}'::jsonb WHERE id=1"#)
        .execute(&pool).await.unwrap();
    let drift = fault::observe(&pool)
        .await
        .expect("lossless JSONB-drift snapshot");
    assert_eq!(
        fault::classified(&original, Some(&intended), &Ok(drift)),
        Observed::Unknown,
        "JSONB drift that rounds equal must never classify Intended"
    );
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn hostile_fixture_selected_row_projection_keeps_untouched_precision_lossless(pool: PgPool) {
    let (_port, _command, input) = accepted(&pool).await;
    // Extra private fixture fields make untouched numeric/JSONB data present in the selected receipt row.
    // Adding defaults does not UPDATE rows or suspend any trigger/FK; production schema stays unchanged.
    sqlx::query(r#"ALTER TABLE public.ont_action_command_receipts ADD COLUMN hostile_fixture_precision numeric DEFAULT 0.10000000000000000001, ADD COLUMN hostile_fixture_precision_document jsonb DEFAULT '{"number":0.10000000000000000001}'::jsonb"#)
        .execute(&pool).await.unwrap();
    let before = fault::observe(&pool)
        .await
        .expect("lossless selected-row snapshot");
    let before_rows = &before.rows["public.ont_action_command_receipts"];
    assert!(
        before_rows.len() == 1 && before_rows[0].matches("0.10000000000000000001").count() == 2,
        "actual full-row TEXT must retain both untouched precise values"
    );
    fault::run(&pool, &input, Fault::None)
        .await
        .expect("database-projected exception must keep untouched digits");
    let after = fault::observe(&pool)
        .await
        .expect("lossless projected selected-row snapshot");
    let after_rows = &after.rows["public.ont_action_command_receipts"];
    assert!(
        after_rows.len() == 1 && after_rows[0].matches("0.10000000000000000001").count() == 2,
        "intended construction must not reparse or round untouched full-row fields"
    );
}
