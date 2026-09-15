//! Strict target parsing candidate over actual AppConfig; no database connections.
//! Runtime cluster identity, primary admission and HA fencing are separate tests.
use super::*;
use sqlx::postgres::PgConnectOptions;
use std::io::Write;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

const TARGETS: [(&str, &str); 5] = [
    ("DATABASE_URL", "console_rt"),
    ("AUTH_DATABASE_URL", "console_auth_rt"),
    ("LEAVE_COMMAND_DATABASE_URL", "console_leave_cmd"),
    ("ONTOLOGY_COMMAND_DATABASE_URL", "console_ontology_cmd"),
    (
        "PLATFORM_FORCE_COMMAND_DATABASE_URL",
        "console_platform_force_cmd",
    ),
];
const CHILD: &str = "CONSOLE_AUTH_TARGET_PARSER_CHILD";

fn pairs() -> Vec<(&'static str, String)> {
    let mut values = account_transport_config_pairs();
    values.push((
        "AUTH_DATABASE_URL",
        "postgresql://console_auth_rt:auth-fixture@localhost:5544/console".to_owned(),
    ));
    values
}

fn replace(values: &mut [(&str, String)], key: &str, value: String) {
    let matches = values
        .iter_mut()
        .filter(|(name, _)| *name == key)
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "fixture must replace exactly one real config input"
    );
    for entry in matches {
        entry.1 = value.clone();
    }
}

fn config_url<'a>(config: &'a AppConfig, key: &str) -> &'a str {
    match key {
        "DATABASE_URL" => config.database_url.as_deref(),
        "AUTH_DATABASE_URL" => config.auth_database_url.as_deref(),
        "LEAVE_COMMAND_DATABASE_URL" => config.leave_command_database_url.as_deref(),
        "ONTOLOGY_COMMAND_DATABASE_URL" => config.ontology_command_database_url.as_deref(),
        "PLATFORM_FORCE_COMMAND_DATABASE_URL" => {
            config.platform_force_command_database_url.as_deref()
        }
        _ => panic!("unknown fixture target"),
    }
    .expect("configured target remains present")
}

fn rejected(key: &str, raw: &str) -> String {
    let mut values = pairs();
    replace(&mut values, key, raw.to_owned());
    let result = AppConfig::from_pairs(values);
    assert!(
        result.is_err(),
        "STRICT_TARGET: invalid target must fail at real config boundary"
    );
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains(key),
        "diagnostic identifies fixed configuration key"
    );
    assert!(!message.contains(raw), "diagnostic must not expose raw URL");
    for secret in [
        "target-password-canary",
        "query-key-canary",
        "query-value-canary",
        "second-value-canary",
    ] {
        assert!(
            !message.contains(secret),
            "diagnostic leaked fixture secret"
        );
    }
    message
}

#[test]
fn account_target_accepts_existing_tcp_tls_and_operational_queries() {
    for (key, role) in TARGETS {
        for query in [
            "",
            "?sslmode=disable",
            "?sslmode=verify-full&sslrootcert=%2Ftmp%2Fconsole%20CA.crt",
            "?application_name=console-auth-pilot",
            "?sslmode=ReQuIrE",
            "?password=effective-distinct-password&sslmode=verify-ca&sslrootcert=%2Ftmp%2Fca.crt&application_name=console-pilot",
        ] {
            let mut values = pairs();
            replace(
                &mut values,
                key,
                format!("postgresql://{role}:target-password-canary@localhost:5544/console{query}"),
            );
            let config =
                AppConfig::from_pairs(values).expect("supported target syntax remains accepted");
            let options = PgConnectOptions::from_str(config_url(&config, key)).unwrap();
            assert_eq!(options.get_username(), role);
            assert_eq!(options.get_host(), "localhost");
            assert_eq!(options.get_port(), 5544);
            assert_eq!(options.get_database(), Some("console"));
            assert!(options.get_socket().is_none());
        }
    }
}

#[test]
fn account_target_effective_query_password_preserves_distinctness() {
    // Authority password equals business password; the query overrides it.
    let mut values = pairs();
    replace(&mut values, "AUTH_DATABASE_URL", "postgresql://console_auth_rt:runtime-fixture@localhost:5544/console?password=auth-distinct-query".to_owned());
    assert!(
        AppConfig::from_pairs(values).is_ok(),
        "effective distinct query password is accepted"
    );
    rejected(
        "AUTH_DATABASE_URL",
        "postgresql://console_auth_rt:target-password-canary@localhost:5544/console?password=runtime%2Dfixture",
    );
    rejected(
        "AUTH_DATABASE_URL",
        "postgresql://console_auth_rt:target-password-canary@localhost:5544/console?password=",
    );
}

#[test]
fn account_target_rejects_sqlx_destination_and_startup_overrides() {
    for (key, role) in TARGETS {
        for query in [
            "host=localhost",
            "hostaddr=127.0.0.1",
            "port=5544",
            "dbname=console",
            "user=console_rt",
            "options=-crole%3Dconsole_app",
            "options%5Brole%5D=console_app",
            "options=-csearch_path%3Dpublic",
            "options%5Bstatement_timeout%5D=0",
            "%68ost=localhost",
            "host=%2Ftmp",
        ] {
            rejected(
                key,
                &format!(
                    "postgresql://{role}:target-password-canary@localhost:5544/console?{query}"
                ),
            );
        }
    }
}

#[test]
fn account_target_rejects_unknown_and_duplicate_decoded_query_keys() {
    for (key, role) in TARGETS {
        for query in [
            "query-key-canary=query-value-canary",
            "target_session_attrs=read-write",
            "ssl-mode=verify-full",
            "ssl-root-cert=%2Ftmp%2Fca",
            "sslkey=secret.key",
            "sslmode=require&sslmode=verify-full",
            "sslmode=require&%73slmode=verify-full",
            "password=first&password=second-value-canary",
            "sslrootcert=a&sslrootcert=b",
            "application_name=one&application_name=two",
        ] {
            rejected(
                key,
                &format!(
                    "postgresql://{role}:target-password-canary@localhost:5544/console?{query}"
                ),
            );
        }
    }
}

#[test]
fn account_target_requires_explicit_tcp_host_database_and_no_fragment() {
    for (key, role) in TARGETS {
        for suffix in [
            "@localhost:5544",
            "@localhost:5544/",
            "@localhost:5544/%00",
            "@localhost:5544/console%2Fother",
            "@localhost:5544/console#query-value-canary",
            "@/console",
            "@%2Ftmp:5544/console",
        ] {
            rejected(
                key,
                &format!("postgresql://{role}:target-password-canary{suffix}"),
            );
        }
    }
}

#[test]
fn account_target_rejects_malformed_percent_and_invalid_allowed_values() {
    for (key, role) in TARGETS {
        for tail in [
            "console?sslmode=",
            "console?sslmode=query-value-canary",
            "console?sslrootcert=",
            "console?sslrootcert=%00",
            "console?application_name=",
            "console?application_name=%00",
            "console?application_name=%GG",
            "console?application_name=%FF",
            "console?%GG=query-value-canary",
            "con%GGsole",
            "con%FFsole",
        ] {
            rejected(
                key,
                &format!("postgresql://{role}:target-password-canary@localhost:5544/{tail}"),
            );
        }
        rejected(
            key,
            &format!(
                "postgresql://{role}:target-password-canary@localhost:5544/console?application_name={}",
                "a".repeat(64)
            ),
        );
    }
}

#[derive(Clone)]
struct Captured(Arc<Mutex<Vec<u8>>>);
impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn account_target_rejection_prevents_sqlx_unknown_parameter_secret_logging() {
    let output = Captured(Arc::new(Mutex::new(Vec::new())));
    let writer = output.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        // Positive control uses the real SQLx parser to prove the capture sees
        // the precise warning this boundary must prevent, with synthetic bytes.
        let _ = PgConnectOptions::from_str("postgresql://console_auth_rt:synthetic@localhost:5544/console?query-key-canary=query-value-canary").unwrap();
        let captured = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
        assert!(
            captured.contains("query-value-canary"),
            "capture must observe actual SQLx unknown-value warning"
        );
        output.0.lock().unwrap().clear();
        for (key, role) in TARGETS {
            let one = rejected(
                key,
                &format!(
                    "postgresql://{role}:target-password-canary@localhost:5544/console?query-key-canary=query-value-canary"
                ),
            );
            let two = rejected(
                key,
                &format!(
                    "postgresql://{role}:target-password-canary@localhost:5544/console?another-unknown=second-value-canary"
                ),
            );
            assert_eq!(
                one, two,
                "unknown-key diagnostic must not vary with supplied key/value"
            );
        }
        let captured = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
        for secret in [
            "target-password-canary",
            "query-key-canary",
            "query-value-canary",
            "second-value-canary",
        ] {
            assert!(
                !captured.contains(secret),
                "config validation leaked secret before returning error"
            );
        }
    });
}

fn run_child(test_name: &str, mode: &str, environment: &[(&str, &str)]) {
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(CHILD, mode);
    for key in [
        "PGHOST",
        "PGHOSTADDR",
        "PGPORT",
        "PGDATABASE",
        "PGUSER",
        "PGPASSWORD",
        "PGOPTIONS",
        "PGAPPNAME",
        "PGSSLMODE",
        "PGSSLROOTCERT",
        "PGSSLCERT",
        "PGSSLKEY",
        "PGPASSFILE",
    ] {
        command.env_remove(key);
    }
    command.envs(environment.iter().copied());
    let mut child = command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("run isolated configuration child");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let mut timed_out = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            _ => {
                timed_out = true;
                let _ = child.kill();
                break;
            }
        }
    }
    let output = child.wait_with_output().expect("reap configuration child");
    assert!(
        !timed_out,
        "configuration child exceeded bounded runtime or wait failed"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("running 1 test"),
        "child selection must execute exactly one test"
    );
    assert!(
        stdout.contains("TARGET_CHILD_EXECUTED"),
        "child must execute real parser branch"
    );
    assert!(
        output.status.success(),
        "isolated configuration assertion failed; no child output is echoed because it may contain canaries"
    );
}

#[test]
fn account_target_ambient_pgoptions_cannot_smuggle_role_or_gucs() {
    if std::env::var(CHILD).as_deref() == Ok("options") {
        println!("TARGET_CHILD_EXECUTED");
        assert!(
            std::env::var("PGOPTIONS")
                .unwrap()
                .contains("role=console_app")
        );
        let result = AppConfig::from_pairs(pairs());
        assert!(
            result.is_err(),
            "ambient startup options must be rejected before serving admission"
        );
        let message = result.unwrap_err().to_string();
        assert!(message.contains("DATABASE_URL"));
        assert!(!message.contains("ambient-options-canary"));
        return;
    }
    run_child(
        "auth_target_parser::account_target_ambient_pgoptions_cannot_smuggle_role_or_gucs",
        "options",
        &[(
            "PGOPTIONS",
            "-crole=console_app -capplication_name=ambient-options-canary",
        )],
    );
}
