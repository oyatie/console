// Included inside deployment_operator_designation: reuse the actual owner,
// registration, restricted LOGIN and lossless snapshot helpers unchanged.
mod subprocess_transport {
    use super::*;
    use std::io::{Read as _, Write as _};
    use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _, PermissionsExt as _};
    use std::path::{Path, PathBuf};
    use std::process::{ExitStatus, Stdio};
    use tokio::io::AsyncReadExt as _;

    const PASSWORD_ENV: &str = "CONSOLE_TEST_DEPLOYMENT_OPERATOR_PASSWORD_FILE";
    const CA_ENV: &str = "CONSOLE_TEST_DEPLOYMENT_OPERATOR_CA_FILE";
    const BINARY_ENV: &str = "CONSOLE_TEST_DEPLOYMENT_OPERATOR_BIN";
    const OUTPUT_LIMIT: usize = 4096;
    const REASON: &str = " \tTEST_ONLY private transport reason 현장 인계\r\n";
    const AMBIENT_PASSWORD: &str = "TEST_ONLY_AMBIENT_PASSWORD_MUST_NOT_BE_USED";

    struct PrivateInputs {
        root: PathBuf,
        binary: PathBuf,
        password: Vec<u8>,
        host: String,
        port: u16,
        cleaned: bool,
    }

    impl PrivateInputs {
        fn new(pool: &PgPool) -> Self {
            let binary = PathBuf::from(
                std::env::var_os(BINARY_ENV)
                    .expect("PREREQUISITE: harness-frozen actual operator executable"),
            );
            assert!(
                binary.is_absolute()
                    && std::fs::symlink_metadata(&binary).is_ok_and(|m| m.file_type().is_file()),
                "PREREQUISITE: regular absolute candidate binary"
            );
            let endpoint = Url::parse(
                &std::env::var("CONSOLE_STARTUP_AUTH_DATABASE_URL").unwrap_or_else(|_| {
                    panic!("PREREQUISITE: separately provisioned startup transport")
                }),
            )
            .unwrap_or_else(|_| panic!("PREREQUISITE: valid startup URL"));
            let options = pool.connect_options();
            assert!(
                endpoint.username() == "console_auth_startup"
                    && endpoint.host_str() == Some(options.get_host())
                    && endpoint.port().unwrap_or(5432) == options.get_port()
                    && endpoint.query().is_none()
                    && endpoint.fragment().is_none(),
                "PREREQUISITE: startup and observer point to the same owned endpoint"
            );

            let password = Self::harness_file(PASSWORD_ENV, 4096);
            assert!(
                !password.is_empty()
                    && std::str::from_utf8(&password).is_ok()
                    && !password.iter().any(|b| matches!(b, 0 | b'\r' | b'\n')),
                "PREREQUISITE: exact nonempty newline-free startup password"
            );
            let ca = Self::harness_file(CA_ENV, 65536);
            assert!(!ca.is_empty(), "PREREQUISITE: owned TLS CA");
            let input_root = PathBuf::from(
                std::env::var_os("CONSOLE_TEST_DEPLOYMENT_OPERATOR_INPUT_ROOT")
                    .expect("PREREQUISITE: supervisor-owned private input directory"),
            );
            assert!(
                input_root.is_absolute()
                    && std::fs::symlink_metadata(&input_root)
                        .is_ok_and(|m| { m.is_dir() && m.permissions().mode() & 0o7777 == 0o700 }),
                "PREREQUISITE: regular private0700 supervisor input directory"
            );
            let root = input_root.join(format!("console-operator-lifecycle-{}", Uuid::new_v4()));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&root)
                .expect("create unique private input directory");
            let inputs = Self {
                root,
                binary,
                password,
                host: options.get_host().to_owned(),
                port: options.get_port(),
                cleaned: false,
            };
            inputs.write_private("password", &inputs.password);
            inputs.write_private("ca.pem", &ca);
            inputs
        }

        fn cleanup(&mut self) {
            std::fs::remove_dir_all(&self.root)
                .unwrap_or_else(|_| panic!("private operator input cleanup failed"));
            assert!(
                std::fs::symlink_metadata(&self.root)
                    .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
                "private operator input directory remains after cleanup"
            );
            self.cleaned = true;
        }

        async fn plaintext_startup_refused(&self, pool: &PgPool) {
            let password = std::str::from_utf8(&self.password)
                .unwrap_or_else(|_| panic!("PREREQUISITE: UTF-8 startup password"));
            let options = pool
                .connect_options()
                .as_ref()
                .clone()
                .username("console_auth_startup")
                .password(password)
                .ssl_mode(sqlx::postgres::PgSslMode::Disable);
            let attempt = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                sqlx::PgConnection::connect_with(&options),
            )
            .await
            .unwrap_or_else(|_| panic!("PREREQUISITE: bounded actual plaintext refusal"));
            match attempt {
                Ok(connection) => {
                    drop(connection);
                    panic!("PREREQUISITE: startup plaintext connection must be refused");
                }
                Err(error) => {
                    let database = error
                        .as_database_error()
                        .unwrap_or_else(|| panic!("PREREQUISITE: actual database HBA refusal"));
                    assert!(
                        database.code().as_deref() == Some("28000")
                            && database.message().contains("pg_hba.conf")
                            && database.message().contains("no encryption"),
                        "PREREQUISITE: exact plaintext HBA refusal, not connection/password failure"
                    );
                }
            }
        }

        fn harness_file(key: &str, limit: usize) -> Vec<u8> {
            let path = PathBuf::from(
                std::env::var_os(key)
                    .expect("PREREQUISITE: harness-owned operator TLS/credential file"),
            );
            let file = std::fs::File::open(path)
                .unwrap_or_else(|_| panic!("PREREQUISITE: readable harness file"));
            assert!(
                file.metadata().is_ok_and(|m| m.is_file()),
                "PREREQUISITE: regular harness file"
            );
            let mut bytes = Vec::new();
            file.take((limit + 1) as u64)
                .read_to_end(&mut bytes)
                .unwrap_or_else(|_| panic!("PREREQUISITE: readable bounded harness file"));
            assert!(bytes.len() <= limit, "PREREQUISITE: bounded harness file");
            bytes
        }

        fn write_private(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.root.join(name);
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
                .expect("create a fresh private test input");
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .expect("private input mode");
            file.write_all(bytes).expect("write private test input");
            path
        }

        fn request(&self, name: &str, target: &Target, command: Value) -> PathBuf {
            self.request_with_host(name, target, command, &self.host)
        }

        fn request_with_host(
            &self,
            name: &str,
            target: &Target,
            command: Value,
            host: &str,
        ) -> PathBuf {
            let value = json!({
                "version": 1,
                "target": {
                    "tls_host": host,
                    "port": self.port,
                    "database_name": target.database,
                    "system_identifier": target.system,
                    "database_oid": target.oid,
                    "expected_login": "console_auth_startup"
                },
                "command": command
            });
            let bytes = serde_json::to_vec(&value)
                .unwrap_or_else(|_| panic!("serialize typed test request"));
            self.write_private(name, &bytes)
        }

        async fn run(&self, request: &Path, target: &Target, account: Uuid) -> ProcessOutput {
            self.run_with_files(
                request,
                target,
                account,
                &self.root.join("password"),
                &self.root.join("ca.pem"),
            )
            .await
        }

        async fn run_with_files(
            &self,
            request: &Path,
            target: &Target,
            account: Uuid,
            password_file: &Path,
            ca_file: &Path,
        ) -> ProcessOutput {
            let mut command = tokio::process::Command::new(&self.binary);
            command
                .args(["--request-file"])
                .arg(request)
                .arg("--password-file")
                .arg(password_file)
                .arg("--ca-file")
                .arg(ca_file)
                .env_clear()
                .env("RUST_BACKTRACE", "0")
                // Deliberate hostile ambient settings. The legitimate private
                // request must still commit via its specified startup LOGIN/TLS.
                .env("PGHOST", "203.0.113.1")
                .env("PGPORT", "1")
                .env("PGUSER", "postgres")
                .env("PGDATABASE", "wrong_database")
                .env("PGPASSWORD", AMBIENT_PASSWORD)
                .env("PGSSLMODE", "disable")
                .env("PGSSLROOTCERT", self.root.join("missing-ca"))
                .env("PGSSLKEY", self.root.join("missing-client-key"))
                .env("PGSERVICE", "must-not-be-used")
                .env(
                    "PGOPTIONS",
                    "-c role=postgres -c log_parameter_max_length=-1",
                )
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            let mut child = command
                .spawn()
                .unwrap_or_else(|_| panic!("PREREQUISITE: operator subprocess executable"));
            let stdout = child.stdout.take().expect("captured stdout");
            let stderr = child.stderr.take().expect("captured stderr");
            let output = tokio::time::timeout(std::time::Duration::from_secs(70), async move {
                let mut out = Vec::new();
                let mut err = Vec::new();
                let mut stdout = stdout.take((OUTPUT_LIMIT + 1) as u64);
                let mut stderr = stderr.take((OUTPUT_LIMIT + 1) as u64);
                let (status, _, _) = tokio::try_join!(
                    child.wait(),
                    stdout.read_to_end(&mut out),
                    stderr.read_to_end(&mut err)
                )?;
                Ok::<_, std::io::Error>(ProcessOutput { status, out, err })
            })
            .await
            .unwrap_or_else(|_| panic!("operator subprocess exceeded bounded execution deadline"))
            .unwrap_or_else(|_| panic!("operator subprocess observation failed"));
            assert!(
                output.out.len() <= OUTPUT_LIMIT && output.err.len() <= OUTPUT_LIMIT,
                "operator output exceeded its closed response budget"
            );
            let account = account.to_string();
            for private in [
                self.password.as_slice(),
                REASON.as_bytes(),
                AMBIENT_PASSWORD.as_bytes(),
                target.database.as_bytes(),
                target.system.as_bytes(),
                account.as_bytes(),
            ] {
                assert!(!private.is_empty(), "nonempty canary required");
                assert!(
                    !output.out.windows(private.len()).any(|w| w == private)
                        && !output.err.windows(private.len()).any(|w| w == private),
                    "private input leaked into process output"
                );
            }
            assert!(
                output.err.is_empty(),
                "unexpected operator diagnostic output"
            );
            output
        }
    }

    impl Drop for PrivateInputs {
        fn drop(&mut self) {
            // Only this newly created UUID directory, never supplied fixture paths.
            if !self.cleaned {
                // Panic fallback only. The supervisor owns the parent directory
                // and must verify/remove any failure residue independently.
                let _ = std::fs::remove_dir_all(&self.root);
            }
        }
    }

    struct ProcessOutput {
        status: ExitStatus,
        out: Vec<u8>,
        err: Vec<u8>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Committed {
        schema_version: u8,
        status: String,
        receipt_id: Uuid,
        revision: i64,
        replayed: bool,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Rejected {
        schema_version: u8,
        status: String,
        reason: String,
    }

    impl ProcessOutput {
        fn committed(self) -> Receipt {
            assert!(
                self.status.code() == Some(0),
                "expected committed process status"
            );
            let value: Committed = serde_json::from_slice(&self.out)
                .unwrap_or_else(|_| panic!("committed output is not the exact closed schema"));
            assert!(
                value.schema_version == 1
                    && value.status == "committed"
                    && !value.receipt_id.is_nil()
                    && value.revision > 0,
                "invalid committed receipt"
            );
            (value.receipt_id, value.revision, value.replayed)
        }

        fn failure(self, exit: i32, status: &str, reason: &str) {
            assert!(
                self.status.code() == Some(exit),
                "unexpected bounded refusal exit status"
            );
            let value: Rejected = serde_json::from_slice(&self.out)
                .unwrap_or_else(|_| panic!("failure output is not the exact closed schema"));
            assert!(
                value.schema_version == 1 && value.status == status && value.reason == reason,
                "unexpected bounded refusal classification"
            );
        }

        fn conflict(self) {
            assert!(
                self.status.code() == Some(4),
                "expected actual owner rejection status"
            );
            let value: Rejected = serde_json::from_slice(&self.out)
                .unwrap_or_else(|_| panic!("rejected output is not the exact closed schema"));
            assert!(
                value.schema_version == 1
                    && value.status == "rejected"
                    && value.reason == "command_conflict",
                "expected precise existing-command payload conflict"
            );
        }
    }

    async fn safe_logging_prerequisite(pool: &PgPool, database: &str) {
        assert!(
            database.starts_with("_sqlx_test_"),
            "only owned disposable SQLx database"
        );
        let actual_admin: bool = sqlx::query_scalar(
            "SELECT session_user=current_user AND rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        assert!(
            actual_admin,
            "PREREQUISITE: distinct actual fixture administrator"
        );
        let database = database.replace('"', "\"\"");
        // Actual infrastructure prerequisite, scoped only to this disposable
        // database. No grants, ownership/schema changes, protected row fixtures,
        // or global role configuration changes. Dropping the DB removes it.
        for setting in [
            "log_parameter_max_length",
            "log_parameter_max_length_on_error",
        ] {
            let sql = format!(
                "ALTER ROLE console_auth_startup IN DATABASE \"{database}\" SET {setting}=0"
            );
            sqlx::query(sqlx::AssertSqlSafe(sql))
                .execute(pool)
                .await
                .expect("install database-local startup logging prerequisite");
        }
    }

    async fn one_receipt(pool: &PgPool, receipt: Uuid) -> String {
        sqlx::query_scalar(
            "SELECT to_jsonb(r)::text FROM public.deployment_operator_receipts r WHERE receipt_id=$1",
        )
        .bind(receipt)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn protected_unchanged(pool: &PgPool, expected: &[String], account: Uuid) {
        assert!(
            rows(pool, TABLES).await == expected,
            "transport changed protected Account/credential/Company/legacy histories"
        );
        assert_no_company_identity(pool, account).await;
    }

    #[sqlx::test(migrations = false)]
    async fn actual_tls_process_designates_replays_conflicts_and_revokes(pool: PgPool) {
        let mut files = PrivateInputs::new(&pool);
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        // Existing real HTTP/WebAuthn enrollment; the fixture's terms publisher
        // seed remains explicitly test-only, not ordinary publication proof.
        let (account, _) = enrolled(&app).await;
        let input = designation(&pool, account.account).await;
        safe_logging_prerequisite(&pool, &input.target.database).await;
        let login = startup(&pool).await;
        let ready: bool = sqlx::query_scalar(
            "SELECT current_setting('log_parameter_max_length')='0' AND current_setting('log_parameter_max_length_on_error')='0' AND (SELECT ssl FROM pg_catalog.pg_stat_ssl WHERE pid=pg_backend_pid())",
        )
        .fetch_one(&login)
        .await
        .unwrap();
        assert!(
            ready,
            "PREREQUISITE: genuine TLS startup LOGIN with safe bind-log profile"
        );
        login.close().await;
        files.plaintext_startup_refused(&pool).await;
        let untouched = rows(&pool, TABLES).await;
        assert!(operator_rows(&pool).await.iter().all(|value| value == "[]"));

        let designate_command = json!({
            "kind": "designate",
            "command_id": input.command,
            "account_id": input.account,
            "security_generation": input.generation,
            "expected_revision": 0
        });
        let designate_file =
            files.request("designate.json", &input.target, designate_command.clone());
        let started: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let first = files
            .run(&designate_file, &input.target, input.account)
            .await
            .committed();
        assert!(first.1 == 1 && !first.2);
        let exact: bool = sqlx::query_scalar(
            "SELECT r.command_id=$2 AND r.system_identifier=$3 AND r.database_name=$4 AND r.database_oid=$5 AND r.account_id=$6 AND r.kind='DESIGNATE' AND r.expected_revision=0 AND r.revision=1 AND r.expected_security_generation=$7 AND r.reason IS NULL AND r.recorded_at BETWEEN $8 AND clock_timestamp() AND h.singleton=1 AND h.account_id=r.account_id AND h.system_identifier=r.system_identifier AND h.database_name=r.database_name AND h.database_oid=r.database_oid AND h.revision=1 FROM public.deployment_operator_receipts r JOIN public.deployment_operator_head h ON h.receipt_id=r.receipt_id WHERE r.receipt_id=$1",
        ).bind(first.0).bind(input.command).bind(&input.target.system).bind(&input.target.database)
            .bind(input.target.oid).bind(input.account).bind(input.generation).bind(started)
            .fetch_one(&pool).await.unwrap();
        assert!(
            exact,
            "process receipt must match every original typed owner field"
        );
        let counts: (i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM public.deployment_operator_head),(SELECT count(*) FROM public.deployment_operator_receipts)")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(counts, (1, 1));
        let designated = operator_rows(&pool).await;
        let original_receipt = one_receipt(&pool, first.0).await;
        let head_identity: String = sqlx::query_scalar("SELECT (to_jsonb(h)-'revision'-'receipt_id')::text FROM public.deployment_operator_head h WHERE singleton=1")
            .fetch_one(&pool).await.unwrap();
        protected_unchanged(&pool, &untouched, input.account).await;

        let replay = files
            .run(&designate_file, &input.target, input.account)
            .await
            .committed();
        assert_eq!(replay, (first.0, 1, true));
        assert!(
            operator_rows(&pool).await == designated,
            "replay rewrote receipt/head history"
        );
        protected_unchanged(&pool, &untouched, input.account).await;

        let mut changed = designate_command;
        changed["expected_revision"] = json!(1);
        let conflict_file = files.request("conflict.json", &input.target, changed);
        files
            .run(&conflict_file, &input.target, input.account)
            .await
            .conflict();
        assert!(
            operator_rows(&pool).await == designated,
            "conflict changed committed history"
        );
        protected_unchanged(&pool, &untouched, input.account).await;

        let revoke_command = Uuid::new_v4();
        let revoke_file = files.request(
            "revoke.json",
            &input.target,
            json!({
                "kind": "revoke",
                "command_id": revoke_command,
                "account_id": input.account,
                "expected_revision": 1,
                "reason": REASON
            }),
        );
        let revoke_started: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let revoked = files
            .run(&revoke_file, &input.target, input.account)
            .await
            .committed();
        assert!(revoked.0 != first.0 && revoked.1 == 2 && !revoked.2);
        let exact: bool = sqlx::query_scalar(
            "SELECT r.command_id=$2 AND r.system_identifier=$3 AND r.database_name=$4 AND r.database_oid=$5 AND r.account_id=$6 AND r.kind='REVOKE' AND r.expected_revision=1 AND r.revision=2 AND r.expected_security_generation IS NULL AND r.reason=$7 AND r.recorded_at BETWEEN $8 AND clock_timestamp() AND h.singleton=1 AND h.account_id=r.account_id AND h.system_identifier=r.system_identifier AND h.database_name=r.database_name AND h.database_oid=r.database_oid AND h.revision=2 FROM public.deployment_operator_receipts r JOIN public.deployment_operator_head h ON h.receipt_id=r.receipt_id WHERE r.receipt_id=$1",
        ).bind(revoked.0).bind(revoke_command).bind(&input.target.system).bind(&input.target.database)
            .bind(input.target.oid).bind(input.account).bind(REASON).bind(revoke_started)
            .fetch_one(&pool).await.unwrap();
        assert!(
            exact,
            "revocation must preserve exact private UTF-8/whitespace reason and descriptor"
        );
        let counts: (i64,i64) = sqlx::query_as("SELECT (SELECT count(*) FROM public.deployment_operator_head),(SELECT count(*) FROM public.deployment_operator_receipts)")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(counts, (1, 2));
        assert!(
            one_receipt(&pool, first.0).await == original_receipt,
            "revoke rewrote original receipt"
        );
        let next_head_identity: String = sqlx::query_scalar("SELECT (to_jsonb(h)-'revision'-'receipt_id')::text FROM public.deployment_operator_head h WHERE singleton=1")
            .fetch_one(&pool).await.unwrap();
        assert!(
            next_head_identity == head_identity,
            "revocation changed original head identity"
        );
        let tombstone = operator_rows(&pool).await;
        protected_unchanged(&pool, &untouched, input.account).await;

        let replay = files
            .run(&revoke_file, &input.target, input.account)
            .await
            .committed();
        assert_eq!(replay, (revoked.0, 2, true));
        assert!(
            operator_rows(&pool).await == tombstone,
            "revoke replay changed immutable history"
        );
        protected_unchanged(&pool, &untouched, input.account).await;

        let historical = files
            .run(&designate_file, &input.target, input.account)
            .await
            .committed();
        assert_eq!(historical, (first.0, 1, true));
        assert!(
            operator_rows(&pool).await == tombstone,
            "historical replay reopened revoked authority"
        );
        protected_unchanged(&pool, &untouched, input.account).await;
        files.cleanup();
    }

    #[sqlx::test(migrations = false)]
    async fn actual_tls_process_refuses_unsafe_profile_bad_trust_and_wrong_target(pool: PgPool) {
        let mut files = PrivateInputs::new(&pool);
        assert!(
            files.host == "localhost",
            "PREREQUISITE: owned localhost endpoint with DNS-only localhost certificate"
        );
        let wrong_ca =
            PrivateInputs::harness_file("CONSOLE_TEST_DEPLOYMENT_OPERATOR_WRONG_CA_FILE", 65536);
        let right_ca = PrivateInputs::harness_file(CA_ENV, 65536);
        assert!(
            !wrong_ca.is_empty() && wrong_ca != right_ca,
            "PREREQUISITE: distinct valid wrong-CA fixture"
        );
        let wrong_ca_file = files.write_private("wrong-ca.pem", &wrong_ca);
        const WRONG_PASSWORD: &[u8] = b"TEST_ONLY_UNRELATED_STARTUP_PASSWORD_59e0c7d3";
        assert!(
            files.password != WRONG_PASSWORD,
            "PREREQUISITE: wrong credential differs from actual startup credential"
        );
        let wrong_password_file = files.write_private("wrong-password", WRONG_PASSWORD);
        let actual_password_file = files.root.join("password");
        let actual_ca_file = files.root.join("ca.pem");

        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (account, _) = enrolled(&app).await;
        let input = designation(&pool, account.account).await;
        let untouched = rows(&pool, TABLES).await;
        let empty_history = operator_rows(&pool).await;
        assert!(empty_history.iter().all(|value| value == "[]"));

        let command = json!({
            "kind": "designate",
            "command_id": input.command,
            "account_id": input.account,
            "security_generation": input.generation,
            "expected_revision": 0
        });
        let healthy_file = files.request("healthy.json", &input.target, command.clone());
        let wrong_host_file = files.request_with_host(
            "wrong-host.json",
            &input.target,
            command.clone(),
            "127.0.0.1",
        );
        let mut wrong_target = input.target.clone();
        wrong_target.system = if input.target.system == "1" { "2" } else { "1" }.to_owned();
        let wrong_target_file = files.request("wrong-target.json", &wrong_target, command);

        let admin: bool = sqlx::query_scalar(
            "SELECT session_user=current_user AND rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user",
        ).fetch_one(&pool).await.unwrap();
        assert!(admin, "PREREQUISITE: real fixture administrator");
        let database = input.target.database.replace('"', "\"\"");
        let unsafe_profile = format!(
            "ALTER ROLE console_auth_startup IN DATABASE \"{database}\" SET log_parameter_max_length=-1"
        );
        sqlx::query(sqlx::AssertSqlSafe(unsafe_profile))
            .execute(&pool)
            .await
            .expect("install DB-local unsafe bind-log profile");
        let login = startup(&pool).await;
        let unsafe_ready: bool = sqlx::query_scalar(
            "SELECT current_setting('log_parameter_max_length')='-1' AND (SELECT ssl FROM pg_catalog.pg_stat_ssl WHERE pid=pg_backend_pid())",
        ).fetch_one(&login).await.unwrap();
        assert!(
            unsafe_ready,
            "PREREQUISITE: actual inherited unsafe logging profile over TLS"
        );
        login.close().await;
        files.plaintext_startup_refused(&pool).await;

        files
            .run(&healthy_file, &input.target, input.account)
            .await
            .failure(3, "prerequisite_failed", "unsafe_connection_profile");
        assert!(
            operator_rows(&pool).await == empty_history,
            "unsafe profile caused authority effects"
        );
        protected_unchanged(&pool, &untouched, input.account).await;

        safe_logging_prerequisite(&pool, &input.target.database).await;
        let login = startup(&pool).await;
        let safe_ready: bool = sqlx::query_scalar(
            "SELECT current_setting('log_parameter_max_length')='0' AND current_setting('log_parameter_max_length_on_error')='0'",
        ).fetch_one(&login).await.unwrap();
        assert!(
            safe_ready,
            "PREREQUISITE: actual safe logging profile restored"
        );
        login.close().await;

        files
            .run_with_files(
                &healthy_file,
                &input.target,
                input.account,
                &actual_password_file,
                &wrong_ca_file,
            )
            .await
            .failure(3, "prerequisite_failed", "tls_or_connect_failed");
        assert!(
            operator_rows(&pool).await == empty_history,
            "wrong CA caused authority effects"
        );
        protected_unchanged(&pool, &untouched, input.account).await;

        files
            .run(&wrong_host_file, &input.target, input.account)
            .await
            .failure(3, "prerequisite_failed", "tls_or_connect_failed");
        assert!(
            operator_rows(&pool).await == empty_history,
            "wrong TLS host caused authority effects"
        );
        protected_unchanged(&pool, &untouched, input.account).await;

        let wrong_password_output = files
            .run_with_files(
                &healthy_file,
                &input.target,
                input.account,
                &wrong_password_file,
                &actual_ca_file,
            )
            .await;
        assert!(
            !wrong_password_output
                .out
                .windows(WRONG_PASSWORD.len())
                .any(|w| w == WRONG_PASSWORD)
                && !wrong_password_output
                    .err
                    .windows(WRONG_PASSWORD.len())
                    .any(|w| w == WRONG_PASSWORD),
            "wrong password leaked into process output"
        );
        wrong_password_output.failure(3, "prerequisite_failed", "tls_or_connect_failed");
        assert!(
            operator_rows(&pool).await == empty_history,
            "wrong password caused authority effects"
        );
        protected_unchanged(&pool, &untouched, input.account).await;

        files
            .run(&wrong_target_file, &input.target, input.account)
            .await
            .failure(4, "rejected", "target_mismatch");
        assert!(
            operator_rows(&pool).await == empty_history,
            "wrong descriptor caused authority effects"
        );
        protected_unchanged(&pool, &untouched, input.account).await;

        // A healthy final command uses the same UUID as all refused attempts.
        // Refusal cannot silently consume idempotency or mint authority.
        let started: time::OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .unwrap();
        let receipt = files
            .run(&healthy_file, &input.target, input.account)
            .await
            .committed();
        assert!(receipt.1 == 1 && !receipt.2);
        let exact: bool = sqlx::query_scalar(
            "SELECT r.command_id=$2 AND r.system_identifier=$3 AND r.database_name=$4 AND r.database_oid=$5 AND r.account_id=$6 AND r.kind='DESIGNATE' AND r.expected_revision=0 AND r.revision=1 AND r.expected_security_generation=$7 AND r.reason IS NULL AND r.recorded_at BETWEEN $8 AND clock_timestamp() AND h.singleton=1 AND h.account_id=r.account_id AND h.system_identifier=r.system_identifier AND h.database_name=r.database_name AND h.database_oid=r.database_oid AND h.revision=1 FROM public.deployment_operator_receipts r JOIN public.deployment_operator_head h ON h.receipt_id=r.receipt_id WHERE r.receipt_id=$1",
        ).bind(receipt.0).bind(input.command).bind(&input.target.system).bind(&input.target.database)
            .bind(input.target.oid).bind(input.account).bind(input.generation).bind(started)
            .fetch_one(&pool).await.unwrap();
        assert!(
            exact,
            "healthy positive control must store every exact typed field"
        );
        let counts: (i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM public.deployment_operator_head),(SELECT count(*) FROM public.deployment_operator_receipts)",
        ).fetch_one(&pool).await.unwrap();
        assert_eq!(counts, (1, 1));
        protected_unchanged(&pool, &untouched, input.account).await;
        files.cleanup();
    }
}
