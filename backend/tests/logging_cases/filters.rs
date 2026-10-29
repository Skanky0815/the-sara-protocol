use super::support::Logs;
use the_sara_protocol::telemetry;

fn messages(filter: Option<&str>) -> Vec<String> {
    let (logs, _guard) = Logs::capture(filter);
    tracing::info!(target: "the_sara_protocol", "app info");
    tracing::debug!(target: "the_sara_protocol", "app debug");
    tracing::info!(target: "the_sara_protocol::memories", "memory info");
    tracing::debug!(target: "the_sara_protocol::memories", "memory debug");
    logs.events()
        .iter()
        .map(|event| event["fields"]["message"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn default_filter_emits_application_info() {
    assert_eq!(messages(None), ["app info", "memory info"]);
}

#[test]
fn global_debug_enables_application_debug() {
    assert_eq!(
        messages(Some("debug")),
        ["app info", "app debug", "memory info", "memory debug"]
    );
}

#[test]
fn module_filter_overrides_application_level() {
    assert_eq!(
        messages(Some(
            "the_sara_protocol=warn,the_sara_protocol::memories=debug"
        )),
        ["memory info", "memory debug"]
    );
}

#[test]
fn dependency_and_lookalike_targets_remain_excluded_at_debug() {
    let (logs, _guard) = Logs::capture(Some("debug"));
    tracing::debug!(target: "sqlx::query", statement = "dependency-secret", "SQL payload");
    tracing::info!(target: "the_sara_protocol_untrusted", "lookalike-secret");
    assert!(logs.events().is_empty());
    assert!(!logs.text().contains("secret"));
}

#[test]
fn malformed_filter_is_rejected() {
    assert!(telemetry::log_filter(Some("[invalid")).is_err());
}
