// Explicit durability configuration regressions through the public app API.
// Unreachable fixture URLs prove configuration rejection precedes transport.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use console_app::{AppConfig, AppError, AppState, DatabaseDependency};

const POLICY_KEY: &str = "CONSOLE_DATABASE_DURABILITY";
const LOCAL: &str = r#"{"mode":"local_development"}"#;

fn configured_pairs(role: &str) -> Vec<(&'static str, String)> {
    let mut pairs = vec![
        ("CONSOLE_APP_ROLE", role.to_owned()),
        (
            "DATABASE_URL",
            "postgresql://console_rt:runtime-fixture@127.0.0.1:1/console".to_owned(),
        ),
    ];
    if role == "api" {
        pairs.extend([
            (
                "LEAVE_COMMAND_DATABASE_URL",
                "postgresql://console_leave_cmd:leave-fixture@127.0.0.1:1/console".to_owned(),
            ),
            (
                "ONTOLOGY_COMMAND_DATABASE_URL",
                "postgresql://console_ontology_cmd:ontology-fixture@127.0.0.1:1/console".to_owned(),
            ),
            (
                "PLATFORM_FORCE_COMMAND_DATABASE_URL",
                "postgresql://console_platform_force_cmd:force-fixture@127.0.0.1:1/console"
                    .to_owned(),
            ),
        ]);
    }
    pairs
}

fn assert_policy_error<T>(result: Result<T, AppError>, marker: &str) {
    match result {
        Err(AppError::Config(message)) => assert!(
            message.contains(POLICY_KEY),
            "{marker}: unrelated configuration refusal: {message}"
        ),
        Err(_) => panic!("{marker}: unrelated non-configuration refusal"),
        Ok(_) => panic!("{marker}: missing or invalid durability policy was accepted"),
    }
}

#[test]
fn configured_api_and_worker_require_explicit_durability_policy() {
    for role in ["api", "worker"] {
        let mut positive = configured_pairs(role);
        positive.push((POLICY_KEY, LOCAL.to_owned()));
        AppConfig::from_pairs(positive).expect("all unrelated configuration is valid");
        assert_policy_error(
            AppConfig::from_pairs(configured_pairs(role)),
            "DURABILITY_POLICY_REQUIRED",
        );
    }
}

#[test]
fn invalid_explicit_durability_policy_never_selects_local() {
    for role in ["api", "worker"] {
        for invalid in [
            "",
            "   ",
            "null",
            "[]",
            "{}",
            r#"{"mode":"local"}"#,
            r#"{"mode":"required_remote_apply"}"#,
            r#"{"mode":"local_development","timeout_ms":15}"#,
        ] {
            let mut pairs = configured_pairs(role);
            pairs.push((POLICY_KEY, invalid.to_owned()));
            assert_policy_error(AppConfig::from_pairs(pairs), "DURABILITY_POLICY_INVALID");
        }
    }
}

#[test]
fn explicit_local_policy_keeps_single_node_development_usable() {
    for role in ["api", "worker"] {
        let mut pairs = configured_pairs(role);
        pairs.push((POLICY_KEY, LOCAL.to_owned()));
        let config = AppConfig::from_pairs(pairs).expect("explicit local policy must be accepted");
        assert!(config.database_url.is_some());
    }
}

#[test]
fn database_free_api_and_migrate_do_not_require_durability_policy() {
    let api = AppConfig::from_pairs([("CONSOLE_APP_ROLE", "api")]).unwrap();
    assert!(api.database_url.is_none());
    AppState::new(api, DatabaseDependency::NotConfigured).unwrap();
    let migrate = AppConfig::from_pairs([
        ("CONSOLE_APP_ROLE", "migrate"),
        (
            "DATABASE_URL",
            "postgresql://console_app:migration-fixture@127.0.0.1:1/console",
        ),
    ])
    .expect("migration mode retains its existing owner-only configuration");
    assert!(migrate.database_url.is_some());
}

#[tokio::test]
async fn injected_postgres_requires_policy_even_without_database_url() {
    for role in ["api", "worker"] {
        let config = AppConfig::from_pairs([("CONSOLE_APP_ROLE", role)]).unwrap();
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgresql://console_rt:runtime-fixture@127.0.0.1:1/console")
            .unwrap();
        assert_policy_error(
            AppState::new(config, DatabaseDependency::Postgres(pool)),
            "DURABILITY_INJECTED_POLICY_REQUIRED",
        );
    }
}

#[tokio::test]
async fn injected_postgres_accepts_explicit_local_without_connecting() {
    for role in ["api", "worker"] {
        let config =
            AppConfig::from_pairs([("CONSOLE_APP_ROLE", role), (POLICY_KEY, LOCAL)]).unwrap();
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgresql://console_rt:runtime-fixture@127.0.0.1:1/console")
            .unwrap();
        AppState::new(config, DatabaseDependency::Postgres(pool))
            .expect("explicit local policy permits dependency injection without a connection");
    }
}

// Additive V2: absence and explicit-invalid policy differ even where omission
// is allowed. Existing credential-precedence tests remain their own regression.
#[test]
fn invalid_explicit_policy_is_rejected_for_database_free_api_and_migrate() {
    for role in ["api", "migrate"] {
        for invalid in [
            "",
            "   ",
            "null",
            "[]",
            "{}",
            r#"{"mode":"local"}"#,
            r#"{"mode":"required_remote_apply"}"#,
            r#"{"mode":"local_development","timeout_ms":15}"#,
        ] {
            let mut pairs = vec![("CONSOLE_APP_ROLE", role.to_owned())];
            if role == "migrate" {
                pairs.push((
                    "DATABASE_URL",
                    "postgresql://console_app:migration-fixture@127.0.0.1:1/console".to_owned(),
                ));
            }
            AppConfig::from_pairs(pairs.clone())
                .expect("absent policy remains valid for this non-serving configuration");
            pairs.push((POLICY_KEY, invalid.to_owned()));
            assert_policy_error(
                AppConfig::from_pairs(pairs),
                "DURABILITY_EXPLICIT_INVALID_IS_NOT_ABSENCE",
            );
        }
    }
}

#[tokio::test]
async fn public_config_mutation_requires_policy_before_database_transport() {
    for role in ["api", "worker"] {
        let mut config = AppConfig::from_pairs([("CONSOLE_APP_ROLE", role)])
            .expect("database-free configuration may omit policy");
        assert!(config.database_url.is_none());
        // Public fields can bypass from_pairs' configured-database checks.
        // Reuse the exact valid/distinct role credentials; the loopback port has
        // no test database. Only a policy-specific Config error satisfies this
        // assertion, never an attempted connection's Database error or timeout.
        for (key, value) in configured_pairs(role) {
            match key {
                "CONSOLE_APP_ROLE" => {}
                "DATABASE_URL" => config.database_url = Some(value),
                "LEAVE_COMMAND_DATABASE_URL" => config.leave_command_database_url = Some(value),
                "ONTOLOGY_COMMAND_DATABASE_URL" => {
                    config.ontology_command_database_url = Some(value);
                }
                "PLATFORM_FORCE_COMMAND_DATABASE_URL" => {
                    config.platform_force_command_database_url = Some(value);
                }
                _ => panic!("unexpected configured fixture field"),
            }
        }
        assert!(config.database_url.is_some());
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(4),
            AppState::from_config(config),
        )
        .await
        .expect("missing policy must refuse before waiting for a database transport");
        assert_policy_error(result, "DURABILITY_MUTATED_CONFIG_POLICY_REQUIRED");
    }
}

#[tokio::test]
async fn migrate_tagged_injected_postgres_requires_policy() {
    let config = AppConfig::from_pairs([("CONSOLE_APP_ROLE", "migrate")])
        .expect("migration configuration without a database may omit policy");
    assert!(config.database_url.is_none());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgresql://console_rt:runtime-fixture@127.0.0.1:1/console")
        .unwrap();
    assert_policy_error(
        AppState::new(config, DatabaseDependency::Postgres(pool)),
        "DURABILITY_MIGRATE_TAGGED_INJECTION_POLICY_REQUIRED",
    );
}
