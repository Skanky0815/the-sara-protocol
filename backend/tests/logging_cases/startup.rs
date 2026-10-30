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
