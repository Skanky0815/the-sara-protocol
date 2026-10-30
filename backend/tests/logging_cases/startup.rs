use super::support::{parse_events, startup};

#[test]
fn startup_rejects_configuration_and_invalid_filters_without_echoing_secrets() {
    let output = startup(&[
        ("RUST_LOG", "[invalid-filter-secret"),
        ("HOST", "host-secret"),
        (
            "DATABASE_URL",
            "postgres://user-secret:pass-secret@localhost/db",
        ),
    ]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let text = String::from_utf8(output.stderr).unwrap();
    let events = parse_events(&text);
    assert!(events
        .iter()
        .any(|e| e["fields"]["error_kind"] == "invalid_filter"));
    assert!(events
        .iter()
        .any(|e| e["fields"]["stage"] == "config" && e["level"] == "ERROR"));
    assert!(!text.contains("secret"));
}

#[test]
fn startup_logs_migration_failure_without_dumping_error_details() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        return;
    };
    let output = startup(&[("RUST_LOG", "debug"), ("DATABASE_URL", &url)]);
    assert!(!output.status.success());
    let text = String::from_utf8(output.stderr).unwrap();
    let events = parse_events(&text);
    assert!(events.iter().any(|e| e["fields"]["stage"] == "startup"));
    assert!(events.iter().any(|e| e["fields"]["stage"] == "database"));
    assert!(events
        .iter()
        .any(|e| e["fields"]["stage"] == "migration_load" && e["level"] == "ERROR"));
    assert!(!text.contains(&url));
    assert!(!text.contains("sara_ci_test"));
}

#[test]
fn configured_missing_migration_directory_fails_without_echoing_path() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        return;
    };
    let output = startup(&[
        ("DATABASE_URL", &url),
        ("MIGRATIONS_DIR", "/missing/migration-path-secret"),
    ]);
    assert!(!output.status.success());
    let text = String::from_utf8(output.stderr).unwrap();
    assert!(parse_events(&text)
        .iter()
        .any(|e| e["fields"]["stage"] == "migration_load"));
    assert!(!text.contains("migration-path-secret"));
}

#[test]
fn configured_migrations_apply_before_http_bind_from_foreign_directory() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        return;
    };
    // Keeping this listener open forces a deterministic bind failure after migrations.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port().to_string();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let output = startup(&[
        ("DATABASE_URL", &url),
        ("PORT", &port),
        ("MIGRATIONS_DIR", path.to_str().unwrap()),
    ]);
    assert!(!output.status.success());
    let text = String::from_utf8(output.stderr).unwrap();
    let events = parse_events(&text);
    assert!(events.iter().any(|e| e["fields"]["stage"] == "migrations"));
    assert!(events.iter().any(|e| e["fields"]["stage"] == "http_bind"));
    assert!(!text.contains(&url));
}

#[test]
fn invalid_configured_migration_fails_before_http_start() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        return;
    };
    let path = std::env::temp_dir().join(super::support::unique_name());
    std::fs::create_dir(&path).unwrap();
    // SQLx rejects a migration filename whose version cannot be parsed.
    std::fs::write(path.join("invalid_version.sql"), "SELECT 1;").unwrap();
    let output = startup(&[
        ("DATABASE_URL", &url),
        ("MIGRATIONS_DIR", path.to_str().unwrap()),
    ]);
    std::fs::remove_dir_all(&path).unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8(output.stderr).unwrap();
    let events = parse_events(&text);
    assert!(events
        .iter()
        .any(|e| e["fields"]["stage"] == "migration_load"));
    assert!(!events.iter().any(|e| e["fields"]["stage"] == "http_listen"));
    assert!(!text.contains(&url));
}
