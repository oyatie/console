//! Private corruption protocol; only the ordinary receipt guard is suspended.
#![allow(dead_code)] // The four other binaries use only corrupt_legacy.

use serde_json::Value;
use sqlx::{Connection, PgConnection, PgPool, Postgres, Transaction};
use std::collections::BTreeMap;
use std::sync::mpsc::Sender;
use uuid::Uuid;

const MESSAGE: &str = "ontology action command receipts are immutable";
const BODY: &str =
    "\nBEGIN\n    RAISE EXCEPTION 'ontology action command receipts are immutable';\nEND;\n";
const DISABLE: &str = "ALTER TABLE public.ont_action_command_receipts DISABLE TRIGGER trg_ont_action_command_receipts_immutable";
const ENABLE: &str = "ALTER TABLE public.ont_action_command_receipts ENABLE TRIGGER trg_ont_action_command_receipts_immutable";
const ALWAYS: &str = "ALTER TABLE public.ont_action_command_receipts ENABLE ALWAYS TRIGGER trg_ont_action_command_receipts_immutable";

// Match migration0177 semantics, not merely the trigger's name or body.
const GUARD: &str = r#"SELECT t.tgenabled::text,
    t.tgtype=27 AND NOT t.tgisinternal AND t.tgparentid=0
    AND t.tgconstraint=0 AND t.tgconstrrelid=0 AND t.tgconstrindid=0
    AND NOT t.tgdeferrable AND NOT t.tginitdeferred
    AND t.tgqual IS NULL AND t.tgattr=''::int2vector
    AND t.tgnargs=0 AND t.tgargs='\x'::bytea
    AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
    AND t.tgfoid='public.ont_action_command_receipts_immutable()'::regprocedure
    AND n.nspname='public' AND p.proname='ont_action_command_receipts_immutable'
    AND p.proowner=c.relowner AND p.prokind='f'
    AND p.prorettype='pg_catalog.trigger'::regtype AND l.lanname='plpgsql'
    AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.provariadic=0
    AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.proargnames IS NULL
    AND p.proargdefaults IS NULL AND p.pronargdefaults=0 AND p.protrftypes IS NULL
    AND NOT p.proretset AND NOT p.prosecdef AND NOT p.proleakproof AND NOT p.proisstrict
    AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
    AND p.proconfig IS NULL AND p.probin IS NULL AND p.prosqlbody IS NULL
    AND p.procost=100 AND p.prorows=0 AND p.prosrc=$1
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid
    JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_namespace n ON n.oid=p.pronamespace
    JOIN pg_language l ON l.oid=p.prolang
    WHERE t.tgrelid='public.ont_action_command_receipts'::regclass
    AND t.tgname='trg_ont_action_command_receipts_immutable'"#;

// Names AND SQL literals come from PostgreSQL's identifier/literal quoters.
const CATALOG: &str = r#"SELECT catalog->'tables',catalog::text FROM (SELECT jsonb_build_object(
    'tables',(SELECT coalesce(jsonb_agg(jsonb_build_object(
        'name',format('%I.%I',n.nspname,c.relname),
        'literal',quote_literal(format('%I.%I',n.nspname,c.relname)))
        ORDER BY n.nspname,c.relname),'[]'::jsonb)
        FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relkind IN ('r','p')),
    'guards',(SELECT coalesce(jsonb_agg(jsonb_build_object(
        'trigger',to_jsonb(t),'definition',pg_get_triggerdef(t.oid),
        'function',to_jsonb(p),'function_definition',pg_get_functiondef(p.oid),
        'language',to_jsonb(l),'namespace',to_jsonb(pn))
        ORDER BY t.tgrelid,t.tgname),'[]'::jsonb)
        FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid
        JOIN pg_namespace n ON n.oid=c.relnamespace JOIN pg_proc p ON p.oid=t.tgfoid
        JOIN pg_namespace pn ON pn.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
        WHERE n.nspname='public'),
    'foreign_keys',(SELECT coalesce(jsonb_agg(jsonb_build_object(
        'catalog',to_jsonb(k),'definition',pg_get_constraintdef(k.oid))
        ORDER BY k.conrelid,k.conname),'[]'::jsonb)
        FROM pg_constraint k WHERE k.contype='f' AND
        (k.conrelid IN (SELECT c.oid FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public')
        OR k.confrelid IN (SELECT c.oid FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public'))),
    'receipt_keys',(SELECT coalesce(jsonb_agg(to_jsonb(k) ORDER BY k.conname),'[]'::jsonb)
        FROM pg_constraint k WHERE k.conrelid='public.ont_action_command_receipts'::regclass AND k.contype='p')) AS catalog) full_catalog"#;

#[derive(Clone)]
pub(super) struct Input {
    pub org: Uuid,
    pub command: Uuid,
    pub actor: Uuid,
    pub digest: Vec<u8>,
    pub receipt: Value,
    pub legacy: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Connect,
    Begin,
    Snapshot,
    BeforeDisable,
    Disable,
    Update,
    AfterDisable,
    Restore,
    GuardProof,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Reason {
    Sql(Phase),
    Guard,
    Key,
    Cardinality(u64),
    Shape,
    CatalogDrift,
    TableDrift,
    RowDrift,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Observed {
    Original,
    Intended,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Failure {
    Refused(Reason), // An awaited rollback AND fresh-connection original snapshot were observed.
    FencedRollback(Observed),
    FencedCommit(Observed),
    FencedPostCommit,
    Unobserved(Reason),
}

#[derive(Clone, Copy)]
pub(super) enum InitialGuard {
    Disabled,
    Replica,
    Conditional,
    Columns,
    Arguments,
    OtherFunction,
    SecurityDefiner,
    Stable,
    Strict,
    ParallelSafe,
    Config,
    Cost,
    Body,
}
pub(super) enum Fault {
    None,
    Initial(InitialGuard),
    BeforeDisable,
    AfterDisable,
    Restore,
    GuardDrift,
    RowDrift,
    TableDrift,
    ForeignKeySnapshot,
    Pause(Sender<()>),
    Panic,
    RollbackAckLost,
    CommitAckLost,
    RollbackObservationLost,
    CommitObservationLost,
    NumericPrecisionDrift,
    JsonbPrecisionDrift,
}

#[derive(Clone)]
pub(super) struct Snapshot {
    catalog: String,
    tables: Vec<(String, String)>,
    pub(super) rows: BTreeMap<String, Vec<String>>,
    projected: Option<BTreeMap<String, Vec<String>>>,
    matched: u64,
}

async fn connection(pool: &PgPool) -> Result<PgConnection, Reason> {
    let options = pool.connect_options();
    let mut conn = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        PgConnection::connect_with(options.as_ref()),
    )
    .await
    .map_err(|_| Reason::Sql(Phase::Connect))?
    .map_err(|_| Reason::Sql(Phase::Connect))?;
    // Pinned sqlx0.9 names private test databases this way; resource custody is still a harness gate.
    let private: bool =
        sqlx::query_scalar("SELECT current_database() ~ '^_sqlx_test_[A-Za-z0-9_]{52}$'")
            .fetch_one(&mut conn)
            .await
            .map_err(|_| Reason::Sql(Phase::Connect))?;
    if !private {
        return Err(Reason::Shape);
    }
    sqlx::query("SET lock_timeout = '5s'")
        .execute(&mut conn)
        .await
        .map_err(|_| Reason::Sql(Phase::Connect))?;
    sqlx::query("SET statement_timeout = '15s'")
        .execute(&mut conn)
        .await
        .map_err(|_| Reason::Sql(Phase::Connect))?;
    Ok(conn)
}

async fn snapshot(conn: &mut PgConnection, input: Option<&Input>) -> Result<Snapshot, Reason> {
    // Only catalog-quoted names/literals decode as Value; all catalog/full-row numbers stay SQL TEXT.
    let (inventory, catalog): (Value, String) = sqlx::query_as(CATALOG)
        .fetch_one(&mut *conn)
        .await
        .map_err(|_| Reason::Sql(Phase::Snapshot))?;
    let tables: Result<Vec<_>, Reason> = inventory
        .as_array()
        .ok_or(Reason::Shape)?
        .iter()
        .map(|table| {
            Ok((
                table["name"].as_str().ok_or(Reason::Shape)?.to_owned(),
                table["literal"].as_str().ok_or(Reason::Shape)?.to_owned(),
            ))
        })
        .collect();
    let tables = tables?;
    let reads: Vec<_> = tables.iter().map(|table| {
        let (name, literal) = table;
        let (projected, selected) = if input.is_some() && name == "public.ont_action_command_receipts" {
            // PostgreSQL overlays only the finite exception; no complete row is reparsed in Rust.
            ("CASE WHEN r.org_id=$1 AND r.command_id=$2 THEN to_jsonb(r) || jsonb_build_object('receipt',$3::jsonb) || CASE WHEN $4::boolean THEN jsonb_build_object('owner','ontology.action','target',NULL,'action_key',NULL,'object_type_id',NULL) ELSE '{}'::jsonb END ELSE to_jsonb(r) END",
                "count(*) FILTER (WHERE r.org_id=$1 AND r.command_id=$2)")
        } else { ("to_jsonb(r)", "0::bigint") };
        format!("SELECT {literal} AS name, coalesce(array_agg(to_jsonb(r)::text ORDER BY to_jsonb(r)::text COLLATE \"C\"),ARRAY[]::text[]) AS rows, coalesce(array_agg(({projected})::text ORDER BY ({projected})::text COLLATE \"C\"),ARRAY[]::text[]) AS projected, {selected} AS matched FROM {name} r")
    }).collect();
    if reads.is_empty() {
        return Err(Reason::Shape);
    }
    let sql = format!(
        "SELECT name,rows,projected,matched FROM ({}) all_tables ORDER BY name COLLATE \"C\"",
        reads.join(" UNION ALL ")
    );
    let query = sqlx::query_as::<_, (String, Vec<String>, Vec<String>, i64)>(&sql);
    let batch = if let Some(input) = input {
        query
            .bind(input.org)
            .bind(input.command)
            .bind(&input.receipt)
            .bind(input.legacy)
            .fetch_all(conn)
            .await
    } else {
        query.fetch_all(conn).await
    }
    .map_err(|_| Reason::Sql(Phase::Snapshot))?;
    let mut rows = BTreeMap::new();
    let mut projected = BTreeMap::new();
    let mut matched = 0_u64;
    for (name, original, expected, count) in batch {
        matched = matched
            .checked_add(u64::try_from(count).map_err(|_| Reason::Shape)?)
            .ok_or(Reason::Shape)?;
        if rows.insert(name.clone(), original).is_some() {
            return Err(Reason::Shape);
        }
        projected.insert(name, expected);
    }
    Ok(Snapshot {
        catalog,
        tables,
        rows,
        projected: input.map(|_| projected),
        matched,
    })
}

pub(super) fn compare(expected: &Snapshot, actual: &Snapshot) -> Result<(), Reason> {
    if expected.tables != actual.tables {
        return Err(Reason::TableDrift);
    }
    if expected.catalog != actual.catalog {
        return Err(Reason::CatalogDrift);
    }
    if expected.rows != actual.rows {
        return Err(Reason::RowDrift);
    }
    Ok(())
}

fn intended(before: &Snapshot) -> Result<Snapshot, Reason> {
    require_singleton(before.matched)?;
    let mut expected = before.clone();
    expected.rows = before.projected.clone().ok_or(Reason::Shape)?;
    expected.projected = None;
    Ok(expected)
}

pub(super) fn require_singleton(rows: u64) -> Result<(), Reason> {
    if rows == 1 {
        Ok(())
    } else {
        Err(Reason::Cardinality(rows))
    }
}

async fn guard(conn: &mut PgConnection) -> Result<String, Reason> {
    let shape: Option<(String, bool)> = sqlx::query_as(GUARD)
        .bind(BODY)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|_| Reason::Sql(Phase::Snapshot))?;
    let Some((mode, true)) = shape else {
        return Err(Reason::Guard);
    };
    if !matches!(mode.as_str(), "O" | "A") {
        return Err(Reason::Guard);
    }
    let key: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_constraint k WHERE k.conrelid='public.ont_action_command_receipts'::regclass AND k.contype='p' AND k.convalidated AND NOT k.condeferrable AND k.conkey=ARRAY[(SELECT attnum FROM pg_attribute WHERE attrelid=k.conrelid AND attname='org_id'),(SELECT attnum FROM pg_attribute WHERE attrelid=k.conrelid AND attname='command_id')]::smallint[])")
        .fetch_one(conn).await.map_err(|_| Reason::Sql(Phase::Snapshot))?;
    if !key {
        return Err(Reason::Key);
    }
    Ok(mode)
}

// A new connection must acquire the lock: neither Drop nor a queued ROLLBACK is proof.
pub(super) async fn observe(pool: &PgPool) -> Result<Snapshot, Reason> {
    let mut conn = connection(pool).await?;
    let mut tx = conn.begin().await.map_err(|_| Reason::Sql(Phase::Begin))?;
    sqlx::query("LOCK TABLE public.ont_action_command_receipts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *tx)
        .await
        .map_err(|_| Reason::Sql(Phase::Snapshot))?;
    let read = snapshot(&mut tx, None).await;
    tx.rollback()
        .await
        .map_err(|_| Reason::Sql(Phase::Snapshot))?;
    read
}

async fn initial_fault(conn: &mut PgConnection, shape: InitialGuard) -> Result<(), Reason> {
    let sql = match shape {
        InitialGuard::Disabled => DISABLE,
        InitialGuard::Replica => {
            "ALTER TABLE public.ont_action_command_receipts ENABLE REPLICA TRIGGER trg_ont_action_command_receipts_immutable"
        }
        InitialGuard::SecurityDefiner => {
            "ALTER FUNCTION public.ont_action_command_receipts_immutable() SECURITY DEFINER"
        }
        InitialGuard::Stable => {
            "ALTER FUNCTION public.ont_action_command_receipts_immutable() STABLE"
        }
        InitialGuard::Strict => {
            "ALTER FUNCTION public.ont_action_command_receipts_immutable() STRICT"
        }
        InitialGuard::ParallelSafe => {
            "ALTER FUNCTION public.ont_action_command_receipts_immutable() PARALLEL SAFE"
        }
        InitialGuard::Config => {
            "ALTER FUNCTION public.ont_action_command_receipts_immutable() SET search_path TO pg_catalog"
        }
        InitialGuard::Cost => {
            "ALTER FUNCTION public.ont_action_command_receipts_immutable() COST 1"
        }
        InitialGuard::Body => {
            "CREATE OR REPLACE FUNCTION public.ont_action_command_receipts_immutable() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RETURN NULL; END; $$"
        }
        InitialGuard::Conditional
        | InitialGuard::Columns
        | InitialGuard::Arguments
        | InitialGuard::OtherFunction => {
            sqlx::query("DROP TRIGGER trg_ont_action_command_receipts_immutable ON public.ont_action_command_receipts")
                .execute(&mut *conn).await.map_err(|_| Reason::Sql(Phase::BeforeDisable))?;
            match shape {
                InitialGuard::Conditional => {
                    "CREATE TRIGGER trg_ont_action_command_receipts_immutable BEFORE UPDATE OR DELETE ON public.ont_action_command_receipts FOR EACH ROW WHEN (false) EXECUTE FUNCTION public.ont_action_command_receipts_immutable()"
                }
                InitialGuard::Columns => {
                    "CREATE TRIGGER trg_ont_action_command_receipts_immutable BEFORE UPDATE OF receipt OR DELETE ON public.ont_action_command_receipts FOR EACH ROW EXECUTE FUNCTION public.ont_action_command_receipts_immutable()"
                }
                InitialGuard::Arguments => {
                    "CREATE TRIGGER trg_ont_action_command_receipts_immutable BEFORE UPDATE OR DELETE ON public.ont_action_command_receipts FOR EACH ROW EXECUTE FUNCTION public.ont_action_command_receipts_immutable('unexpected')"
                }
                InitialGuard::OtherFunction => {
                    sqlx::query("CREATE FUNCTION public.hostile_receipt_other_guard() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'ontology action command receipts are immutable'; END; $$")
                        .execute(&mut *conn).await.map_err(|_| Reason::Sql(Phase::BeforeDisable))?;
                    "CREATE TRIGGER trg_ont_action_command_receipts_immutable BEFORE UPDATE OR DELETE ON public.ont_action_command_receipts FOR EACH ROW EXECUTE FUNCTION public.hostile_receipt_other_guard()"
                }
                _ => unreachable!(),
            }
        }
    };
    sqlx::query(sql)
        .execute(conn)
        .await
        .map_err(|_| Reason::Sql(Phase::BeforeDisable))?;
    Ok(())
}

async fn inject(
    tx: &mut Transaction<'_, Postgres>,
    before: &Snapshot,
    input: &Input,
    fault: &Fault,
) -> Result<Snapshot, Reason> {
    if let Fault::Initial(shape) = fault {
        initial_fault(tx, *shape).await?;
    }
    let mode = guard(tx).await?; // All malformed-shape controls refuse BEFORE this protocol's disable.
    let expected = intended(before)?;
    if matches!(fault, Fault::BeforeDisable) {
        sqlx::query("SELECT 1/0")
            .execute(&mut **tx)
            .await
            .map_err(|_| Reason::Sql(Phase::BeforeDisable))?;
    }
    sqlx::query(DISABLE)
        .execute(&mut **tx)
        .await
        .map_err(|_| Reason::Sql(Phase::Disable))?;
    match fault {
        Fault::Pause(signal) => {
            signal.send(()).map_err(|_| Reason::Shape)?;
            std::future::pending::<()>().await;
        }
        Fault::Panic => panic!("hostile receipt cancellation control"),
        Fault::AfterDisable => {
            sqlx::query("SELECT 1/0")
                .execute(&mut **tx)
                .await
                .map_err(|_| Reason::Sql(Phase::AfterDisable))?;
        }
        Fault::RollbackAckLost | Fault::RollbackObservationLost => {
            return Err(Reason::Sql(Phase::AfterDisable));
        }
        _ => {}
    }
    let sql = if input.legacy {
        "UPDATE public.ont_action_command_receipts SET receipt=$1,owner='ontology.action',target=NULL,action_key=NULL,object_type_id=NULL WHERE org_id=$2 AND command_id=$3 AND actor_id=$4 AND payload_digest=$5"
    } else {
        "UPDATE public.ont_action_command_receipts SET receipt=$1 WHERE org_id=$2 AND command_id=$3 AND actor_id=$4 AND payload_digest=$5"
    };
    let changed = sqlx::query(sql)
        .bind(&input.receipt)
        .bind(input.org)
        .bind(input.command)
        .bind(input.actor)
        .bind(&input.digest)
        .execute(&mut **tx)
        .await
        .map_err(|_| Reason::Sql(Phase::Update))?;
    require_singleton(changed.rows_affected())?;
    let restore = if matches!(fault, Fault::Restore) {
        "ALTER TABLE public.ont_action_command_receipts ENABLE TRIGGER hostile_receipt_missing_guard"
    } else if mode == "A" {
        ALWAYS
    } else {
        ENABLE
    };
    sqlx::query(restore)
        .execute(&mut **tx)
        .await
        .map_err(|_| Reason::Sql(Phase::Restore))?;
    if guard(tx).await? != mode {
        return Err(Reason::Guard);
    }
    match fault {
        Fault::GuardDrift => {
            sqlx::query("ALTER FUNCTION public.ont_action_command_receipts_immutable() COST 1")
                .execute(&mut **tx)
                .await
                .map_err(|_| Reason::Sql(Phase::Restore))?;
        }
        Fault::RowDrift => {
            sqlx::query("UPDATE public.users SET display_name=display_name || 'fixture-control' WHERE id=$1").bind(input.actor).execute(&mut **tx).await.map_err(|_| Reason::Sql(Phase::Restore))?;
        }
        Fault::TableDrift => {
            sqlx::query("CREATE TABLE public.hostile_receipt_control_inventory(id integer)")
                .execute(&mut **tx)
                .await
                .map_err(|_| Reason::Sql(Phase::Restore))?;
        }
        Fault::NumericPrecisionDrift => {
            sqlx::query("UPDATE public.hostile_receipt_precision_control SET amount=0.10000000000000000001 WHERE id=1").execute(&mut **tx).await.map_err(|_| Reason::Sql(Phase::Restore))?;
        }
        Fault::JsonbPrecisionDrift => {
            sqlx::query(r#"UPDATE public.hostile_receipt_precision_control SET document='{"number":0.10000000000000000001}'::jsonb WHERE id=1"#).execute(&mut **tx).await.map_err(|_| Reason::Sql(Phase::Restore))?;
        }
        _ => {}
    }
    let mut actual = snapshot(tx, None).await?;
    // Fake transport corruption exercises the real comparator without touching any FK.
    if matches!(fault, Fault::ForeignKeySnapshot) {
        actual.catalog = actual.catalog.replacen(
            "\"foreign_keys\":",
            "\"foreign_keys\":null,\"foreign_keys_before_corruption\":",
            1,
        );
    }
    compare(&expected, &actual)?;
    guard(tx).await?;
    Ok(expected)
}

pub(super) fn classified(
    before: &Snapshot,
    expected: Option<&Snapshot>,
    observed: &Result<Snapshot, Reason>,
) -> Observed {
    match observed {
        Ok(actual) if compare(before, actual).is_ok() => Observed::Original,
        Ok(actual) if expected.is_some_and(|expected| compare(expected, actual).is_ok()) => {
            Observed::Intended
        }
        _ => Observed::Unknown,
    }
}

pub(super) async fn run(pool: &PgPool, input: &Input, fault: Fault) -> Result<(), Failure> {
    if input.digest.len() != 32 || input.receipt.get("target").is_some() {
        return Err(Failure::Unobserved(Reason::Shape));
    }
    let mut conn = connection(pool).await.map_err(Failure::Unobserved)?;
    let mut tx = conn
        .begin()
        .await
        .map_err(|_| Failure::Unobserved(Reason::Sql(Phase::Begin)))?;
    sqlx::query("LOCK TABLE public.ont_action_command_receipts IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *tx)
        .await
        .map_err(|_| Failure::Unobserved(Reason::Sql(Phase::Snapshot)))?;
    let before = match snapshot(&mut tx, Some(input)).await {
        Ok(before) => before,
        Err(reason) => {
            tx.rollback()
                .await
                .map_err(|_| Failure::FencedRollback(Observed::Unknown))?;
            return Err(Failure::Unobserved(reason));
        }
    };
    let expected = match inject(&mut tx, &before, input, &fault).await {
        Ok(expected) => expected,
        Err(reason) => {
            let rollback_ok = if matches!(fault, Fault::RollbackAckLost) {
                drop(tx);
                false // Inject a missing rollback acknowledgement; Drop alone is never success.
            } else {
                tx.rollback().await.is_ok()
            };
            drop(conn);
            let mut observed = observe(pool).await;
            // Fake lost observation response tests the actual unknown-state fence; no network fault is claimed.
            if matches!(fault, Fault::RollbackObservationLost) {
                observed = Err(Reason::Sql(Phase::Snapshot));
            }
            let state = classified(&before, None, &observed);
            return if rollback_ok && state == Observed::Original {
                Err(Failure::Refused(reason))
            } else {
                Err(Failure::FencedRollback(state))
            };
        }
    };
    let commit_ok = tx.commit().await.is_ok();
    drop(conn);
    let mut observed = observe(pool).await;
    if matches!(fault, Fault::CommitObservationLost) {
        observed = Err(Reason::Sql(Phase::Snapshot));
    }
    if !commit_ok || matches!(fault, Fault::CommitAckLost | Fault::CommitObservationLost) {
        // Commit errors may have committed the intended JSON; no replay permission is returned.
        return Err(Failure::FencedCommit(classified(
            &before,
            Some(&expected),
            &observed,
        )));
    }
    if classified(&before, Some(&expected), &observed) != Observed::Intended {
        return Err(Failure::FencedPostCommit);
    }
    prove_restored_guard(pool, input, &expected)
        .await
        .map_err(|_| Failure::FencedPostCommit)
}

async fn prove_restored_guard(
    pool: &PgPool,
    input: &Input,
    expected: &Snapshot,
) -> Result<(), Reason> {
    for sql in [
        "UPDATE public.ont_action_command_receipts SET receipt=receipt WHERE org_id=$1 AND command_id=$2",
        "DELETE FROM public.ont_action_command_receipts WHERE org_id=$1 AND command_id=$2",
    ] {
        let mut conn = connection(pool).await?;
        let mut tx = conn
            .begin()
            .await
            .map_err(|_| Reason::Sql(Phase::GuardProof))?;
        let attempted = sqlx::query(sql)
            .bind(input.org)
            .bind(input.command)
            .execute(&mut *tx)
            .await;
        let refused = attempted.err().is_some_and(|error| {
            error.as_database_error().is_some_and(|error| {
                error.code().as_deref() == Some("P0001") && error.message() == MESSAGE
            })
        });
        tx.rollback()
            .await
            .map_err(|_| Reason::Sql(Phase::GuardProof))?;
        if !refused {
            return Err(Reason::Guard);
        }
    }
    let mut conn = connection(pool).await?;
    guard(&mut conn).await?;
    compare(expected, &observe(pool).await?)
}

pub(super) async fn corrupt_legacy(
    pool: &PgPool,
    org: Uuid,
    command: Uuid,
    actor: Uuid,
    digest: &[u8],
    receipt: Value,
) -> Result<(), Failure> {
    run(
        pool,
        &Input {
            org,
            command,
            actor,
            digest: digest.to_vec(),
            receipt,
            legacy: true,
        },
        Fault::None,
    )
    .await
}
