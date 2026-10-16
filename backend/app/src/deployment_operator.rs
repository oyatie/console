//! Private-file, one-shot adapter over the existing deployment SQL owners.
//! Receipts describe committed history, never current deployment authority.
use rustls::pki_types::{CertificateDer, ServerName, pem::PemObject};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
    sync::Arc,
    time::Duration,
};
use tokio_postgres::{Client, Config, Row, config::SslMode};
use uuid::Uuid;
use zeroize::Zeroizing;

const LOGIN: &str = "console_auth_startup";
const OPTIONS: &str = "-c search_path=pg_catalog,pg_temp -c statement_timeout=30000 -c lock_timeout=5000 -c idle_in_transaction_session_timeout=30000 -c transaction_timeout=45000 -c log_parameter_max_length_on_error=0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    version: u8,
    target: Target,
    command: Command,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    tls_host: String,
    port: u16,
    database_name: String,
    system_identifier: String,
    database_oid: u32,
    expected_login: String,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Command {
    Designate {
        command_id: Uuid,
        account_id: Uuid,
        security_generation: i64,
        expected_revision: i64,
    },
    Revoke {
        command_id: Uuid,
        account_id: Uuid,
        expected_revision: i64,
        reason: String,
    },
}
impl Request {
    fn valid(&self) -> bool {
        let t = &self.target;
        let system = t.system_identifier.parse::<u64>().ok();
        let target = self.version == 1
            && t.expected_login == LOGIN
            && t.port != 0
            && t.database_oid != 0
            && !t.database_name.is_empty()
            && t.database_name.len() <= 63
            && !t.database_name.contains('\0')
            && system.is_some_and(|n| n > 0 && n.to_string() == t.system_identifier)
            && !t.tls_host.is_empty()
            && t.tls_host.len() <= 253
            && !t.tls_host.bytes().any(|b| {
                b.is_ascii_control() || b.is_ascii_whitespace() || matches!(b, b'/' | b',' | b'%')
            })
            && ServerName::try_from(t.tls_host.as_str()).is_ok();
        target
            && match &self.command {
                Command::Designate {
                    command_id,
                    account_id,
                    security_generation,
                    expected_revision,
                } => {
                    !command_id.is_nil()
                        && !account_id.is_nil()
                        && *security_generation > 0
                        && *expected_revision >= 0
                }
                Command::Revoke {
                    command_id,
                    account_id,
                    expected_revision,
                    reason,
                } => {
                    !command_id.is_nil()
                        && !account_id.is_nil()
                        && *expected_revision >= 0
                        && !reason.is_empty()
                        && reason.len() <= 512
                        && !reason.contains('\0')
                        && reason
                            .bytes()
                            .any(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n' | 0x0b | 0x0c))
                }
            }
    }
}

#[derive(Serialize)]
struct Outcome {
    schema_version: u8,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    revision: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replayed: Option<bool>,
    #[serde(skip)]
    exit: u8,
}
impl Outcome {
    fn failure(status: &'static str, reason: &'static str, exit: u8) -> Self {
        Self {
            schema_version: 1,
            status,
            reason: Some(reason),
            receipt_id: None,
            revision: None,
            replayed: None,
            exit,
        }
    }
    fn input() -> Self {
        Self::failure("input_invalid", "invalid_input", 2)
    }
    fn prerequisite(reason: &'static str) -> Self {
        Self::failure("prerequisite_failed", reason, 3)
    }
    fn unknown() -> Self {
        Self::failure("outcome_unknown", "reconcile_same_command", 6)
    }
    fn receipt(row: &Row) -> Result<Self, ()> {
        let id: Uuid = row.try_get(0).map_err(|_| ())?;
        let revision: i64 = row.try_get(1).map_err(|_| ())?;
        let replayed: bool = row.try_get(2).map_err(|_| ())?;
        if id.is_nil() || revision <= 0 {
            return Err(());
        }
        Ok(Self {
            schema_version: 1,
            status: "committed",
            reason: None,
            receipt_id: Some(id),
            revision: Some(revision),
            replayed: Some(replayed),
            exit: 0,
        })
    }
}

fn read_file(path: &Path, limit: usize, private: bool) -> Result<Zeroizing<Vec<u8>>, ()> {
    use rustix::fs::{Mode, OFlags};
    let fd = rustix::fs::open(
        path,
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOCTTY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|_| ())?;
    let file = File::from(fd);
    let metadata = file.metadata().map_err(|_| ())?;
    let mode = metadata.mode() & 0o7777;
    if !metadata.is_file()
        || (metadata.uid() != 0 && metadata.uid() != rustix::process::geteuid().as_raw())
        || (private && !matches!(mode, 0o400 | 0o600))
        || (!private && mode & 0o7133 != 0)
    {
        return Err(());
    }
    let mut data = Zeroizing::new(Vec::new());
    file.take(limit as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|_| ())?;
    if data.len() > limit {
        return Err(());
    }
    Ok(data)
}

struct Prepared {
    request: Request,
    config: Config,
    tls: tokio_postgres_rustls::MakeRustlsConnect,
}
fn prepare(args: impl Iterator<Item = OsString>) -> Result<Prepared, ()> {
    let mut request_path = None;
    let mut password_path = None;
    let mut ca_path = None;
    let mut args = args;
    while let Some(flag) = args.next() {
        let slot = match flag.to_str() {
            Some("--request-file") => &mut request_path,
            Some("--password-file") => &mut password_path,
            Some("--ca-file") => &mut ca_path,
            _ => return Err(()),
        };
        if slot.is_some() {
            return Err(());
        }
        *slot = Some(args.next().ok_or(())?);
    }
    let request_bytes = read_file(Path::new(&request_path.ok_or(())?), 16 * 1024, true)?;
    let request: Request = serde_json::from_slice(&request_bytes).map_err(|_| ())?;
    if !request.valid() {
        return Err(());
    }
    let password = read_file(Path::new(&password_path.ok_or(())?), 4096, true)?;
    if password.is_empty()
        || std::str::from_utf8(&password).is_err()
        || password.iter().any(|b| matches!(b, 0 | b'\n' | b'\r'))
    {
        return Err(());
    }
    let ca = read_file(Path::new(&ca_path.ok_or(())?), 64 * 1024, false)?;
    let mut roots = rustls::RootCertStore::empty();
    for cert in CertificateDer::pem_slice_iter(&ca) {
        roots.add(cert.map_err(|_| ())?).map_err(|_| ())?;
    }
    if roots.is_empty() {
        return Err(());
    }
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|_| ())?
    .with_root_certificates(roots)
    .with_no_client_auth();
    // Never parse a URL or read ambient PG*, application or telemetry settings.
    let mut config = Config::new();
    config
        .host(&request.target.tls_host)
        .port(request.target.port)
        .dbname(&request.target.database_name)
        .user(LOGIN)
        .password(password.as_slice())
        .ssl_mode(SslMode::Require)
        .application_name("console-deployment-operator")
        .options(OPTIONS);
    Ok(Prepared {
        request,
        config,
        tls: tokio_postgres_rustls::MakeRustlsConnect::new(tls),
    })
}

async fn preflight(client: &Client) -> Result<(), ()> {
    let row = client
        .query_one(super::serving_database_identity_query(), &[])
        .await
        .map_err(|_| ())?;
    let session: String = row.try_get("session_user").map_err(|_| ())?;
    let current: String = row.try_get("current_user").map_err(|_| ())?;
    let attributes = super::RoleAttributes {
        can_login: row.try_get("can_login").map_err(|_| ())?,
        is_superuser: row.try_get("is_superuser").map_err(|_| ())?,
        bypasses_rls: row.try_get("bypasses_rls").map_err(|_| ())?,
        inherits_privileges: row.try_get("inherits_privileges").map_err(|_| ())?,
        can_create_db: row.try_get("can_create_db").map_err(|_| ())?,
        can_create_role: row.try_get("can_create_role").map_err(|_| ())?,
        can_replicate: row.try_get("can_replicate").map_err(|_| ())?,
    };
    let memberships: bool = row
        .try_get("has_forbidden_membership_edge")
        .map_err(|_| ())?;
    if session != LOGIN
        || current != LOGIN
        || attributes != super::RoleAttributes::HARDENED_LOGIN
        || memberships
    {
        return Err(());
    }
    for key in [
        "statement_timeout_matches",
        "idle_in_transaction_session_timeout_matches",
        "transaction_timeout_matches",
    ] {
        if !row.try_get::<_, bool>(key).map_err(|_| ())? {
            return Err(());
        }
    }
    let row = client.query_one("SELECT current_setting('log_parameter_max_length')='0' AND current_setting('log_parameter_max_length_on_error')='0' AND current_setting('lock_timeout')::interval=interval '5 seconds' AND current_setting('search_path')='pg_catalog,pg_temp'", &[]).await.map_err(|_| ())?;
    if !row.try_get::<_, bool>(0).map_err(|_| ())? {
        return Err(());
    }
    Ok(())
}

fn server_failure(error: &tokio_postgres::Error) -> Outcome {
    let Some(db) = error.as_db_error() else {
        return Outcome::unknown();
    };
    match db.code().code() {
        "P0001" => {
            let reason = match db.message() {
                "deployment_operator.target_mismatch" => "target_mismatch",
                "deployment_operator.target_ineligible" => "target_ineligible",
                "deployment_operator.command_conflict" => "command_conflict",
                "deployment_operator.already_initialized" => "already_initialized",
                "deployment_operator.stale_revision" => "stale_revision",
                "deployment_operator.already_revoked" => "already_revoked",
                "deployment_operator.invalid_command" => "invalid_command",
                _ => return Outcome::prerequisite("server_failure"),
            };
            Outcome::failure("rejected", reason, 4)
        }
        "42501" => Outcome::failure("rejected", "permission_denied", 4),
        "55P03" => Outcome::failure("retryable", "lock_timeout", 5),
        "57014" => Outcome::failure("retryable", "statement_timeout", 5),
        _ => Outcome::prerequisite("server_failure"),
    }
}

async fn transact(client: &mut Client, request: &Request, submitted: &mut bool) -> Outcome {
    if preflight(client).await.is_err() {
        return Outcome::prerequisite("unsafe_connection_profile");
    }
    let tx = match client
        .build_transaction()
        .isolation_level(tokio_postgres::IsolationLevel::ReadCommitted)
        .start()
        .await
    {
        Ok(tx) => tx,
        Err(_) => return Outcome::prerequisite("transaction_unavailable"),
    };
    let t = &request.target;
    let oid = i64::from(t.database_oid);
    *submitted = true;
    let rows = match &request.command {
        Command::Designate { command_id, account_id, security_generation, expected_revision } => tx.query(
            "SELECT receipt_id,revision,replayed FROM public.deployment_operator_designate_v1($1,$2,$3,$4,$5,$6,$7)",
            &[&t.system_identifier, &t.database_name, &oid, command_id, account_id, security_generation, expected_revision]).await,
        Command::Revoke { command_id, account_id, expected_revision, reason } => tx.query(
            "SELECT receipt_id,revision,replayed FROM public.deployment_operator_revoke_v1($1,$2,$3,$4,$5,$6,$7)",
            &[&t.system_identifier, &t.database_name, &oid, command_id, account_id, expected_revision, reason]).await,
    };
    let receipt = match rows {
        Ok(rows) if rows.len() == 1 => Outcome::receipt(&rows[0]),
        Ok(_) => Err(()),
        Err(error) => {
            let outcome = server_failure(&error);
            return if tx.rollback().await.is_ok() {
                outcome
            } else {
                Outcome::unknown()
            };
        }
    };
    let Ok(receipt) = receipt else {
        return if tx.rollback().await.is_ok() {
            Outcome::prerequisite("invalid_receipt")
        } else {
            Outcome::unknown()
        };
    };
    match tx.commit().await {
        Ok(()) => receipt,
        Err(_) => Outcome::unknown(),
    }
}

async fn interrupted() {
    use tokio::signal::unix::{SignalKind, signal};
    let Ok(mut terminate) = signal(SignalKind::terminate()) else {
        return;
    };
    tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
}

async fn execute(prepared: Prepared) -> Outcome {
    let connection = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(10), prepared.config.connect(prepared.tls)) => result,
        _ = interrupted() => return Outcome::failure("not_submitted", "interrupted", 3),
    };
    let (mut client, connection) = match connection {
        Ok(Ok(connected)) => connected,
        _ => return Outcome::prerequisite("tls_or_connect_failed"),
    };
    // This executable never installs a logger/subscriber. Do not format notices.
    let mut driver = tokio::spawn(connection);
    let mut submitted = false;
    let result = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(50), transact(&mut client, &prepared.request, &mut submitted)) => result.ok(),
        _ = interrupted() => None,
    };
    let outcome = result.unwrap_or_else(|| {
        if submitted {
            Outcome::unknown()
        } else {
            Outcome::failure("not_submitted", "interrupted_or_deadline", 3)
        }
    });
    drop(client);
    // Acknowledged COMMIT is final, regardless of connection cleanup outcome.
    if tokio::time::timeout(Duration::from_secs(2), &mut driver)
        .await
        .is_err()
    {
        driver.abort();
    }
    outcome
}

/// Run only from the dedicated executable: no general application initialization.
pub fn run(args: impl Iterator<Item = OsString>) -> u8 {
    let outcome = match prepare(args) {
        Err(()) => Outcome::input(),
        Ok(prepared) => match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => {
                let outcome = runtime.block_on(execute(prepared));
                // Timed-out DNS work may still occupy a blocking worker.
                runtime.shutdown_timeout(Duration::from_secs(2));
                outcome
            }
            Err(_) => Outcome::prerequisite("runtime_unavailable"),
        },
    };
    let mut output = Vec::new();
    if serde_json::to_writer(&mut output, &outcome).is_err() {
        return 6;
    }
    output.push(b'\n');
    let mut stdout = std::io::stdout().lock();
    if stdout
        .write_all(&output)
        .and_then(|()| stdout.flush())
        .is_err()
    {
        return 6;
    }
    outcome.exit
}
