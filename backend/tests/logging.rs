use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
use the_sara_protocol::{app, app_with_pool, telemetry};
use tower::ServiceExt;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct Logs(Arc<Mutex<Vec<u8>>>);
impl Write for Logs {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl<'a> MakeWriter<'a> for Logs {
    type Writer = Self;
    fn make_writer(&'a self) -> Self {
        self.clone()
    }
}
impl Logs {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
    fn events(&self) -> Vec<Value> {
        self.text()
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSON log event"))
            .collect()
    }
}

#[test]
fn default_global_and_module_filters_work_without_dependency_logs() {
    for (filter, expected) in [
        (None, vec!["app info", "memory info"]),
        (
            Some("debug"),
            vec!["app info", "app debug", "memory info", "memory debug"],
        ),
        (
            Some("the_sara_protocol=warn,the_sara_protocol::memories=debug"),
            vec!["memory info", "memory debug"],
        ),
    ] {
        let logs = Logs::default();
        let subscriber =
            telemetry::subscriber(telemetry::log_filter(filter).unwrap(), logs.clone());
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "the_sara_protocol", "app info");
            tracing::debug!(target: "the_sara_protocol", "app debug");
            tracing::info!(target: "the_sara_protocol::memories", "memory info");
            tracing::debug!(target: "the_sara_protocol::memories", "memory debug");
            tracing::debug!(target: "sqlx::query", statement = "dependency-secret", "SQL payload");
            tracing::info!(target: "the_sara_protocol_untrusted", "lookalike-secret");
        });
        let events = logs.events();
        let messages: Vec<_> = events
            .iter()
            .map(|event| event["fields"]["message"].as_str().unwrap())
            .collect();
        assert_eq!(messages, expected);
        assert!(!logs.text().contains("secret"));
    }
    assert!(telemetry::log_filter(Some("[invalid")).is_err());
}

#[tokio::test(flavor = "current_thread")]
async fn requests_have_safe_routes_status_duration_and_correlated_errors() {
    let logs = Logs::default();
    let subscriber =
        telemetry::subscriber(telemetry::log_filter(Some("debug")).unwrap(), logs.clone());
    let _guard = tracing::subscriber::set_default(subscriber);
    let pool = sqlx_postgres::PgPool::connect_lazy(
        "postgres://db-user-secret:db-pass-secret@localhost/test",
    )
    .unwrap();
    pool.close().await;
    let router = app_with_pool(pool);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health?token=query-secret")
                .header("authorization", "Bearer auth-secret")
                .header("cookie", "session=cookie-secret")
                .body(Body::from("health-body-secret"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/personas/88442211/memories?secret=query-secret")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"content":"memory-body-secret","source":"source-secret",
            "observed_at":"2026-10-28T12:00:00Z","confirmation_status":"confirmed"})
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(
        bytes.is_empty(),
        "logging must not add internal API error details"
    );
    let response = router
        .oneshot(
            Request::builder()
                .uri("/unknown-path-secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let events = logs.events();
    let completed: Vec<_> = events
        .iter()
        .filter(|event| event["fields"]["message"] == "request completed")
        .collect();
    assert_eq!(completed.len(), 3);
    for (event, (status, route, method)) in completed.iter().zip([
        (200, "/health", "GET"),
        (500, "/personas/{id}/memories", "POST"),
        (404, "<unmatched>", "GET"),
    ]) {
        assert_eq!(event["fields"]["status"], status);
        assert!(event["fields"]["duration_ms"].as_f64().unwrap() >= 0.0);
        assert_eq!(event["span"]["route"], route);
        assert_eq!(event["span"]["method"], method);
    }
    assert_ne!(
        completed[0]["span"]["request_id"],
        completed[1]["span"]["request_id"]
    );
    let error = events
        .iter()
        .find(|event| event["fields"]["error_kind"] == "pool_closed")
        .unwrap();
    assert_eq!(error["fields"]["operation"], "create_memory");
    assert_eq!(
        error["span"]["request_id"],
        completed[1]["span"]["request_id"]
    );
    assert_eq!(error["level"], "ERROR");
    assert_eq!(completed[1]["level"], "ERROR");
    assert_eq!(completed[2]["level"], "WARN");
    for forbidden in [
        "secret",
        "88442211",
        "authorization",
        "cookie",
        "postgres://",
    ] {
        assert!(
            !logs.text().contains(forbidden),
            "logged forbidden data: {forbidden}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn raw_database_errors_and_connection_values_are_not_logged() {
    let logs = Logs::default();
    let _guard = tracing::subscriber::set_default(telemetry::subscriber(
        telemetry::log_filter(Some("debug")).unwrap(),
        logs.clone(),
    ));
    telemetry::database_error(
        "read",
        &sqlx_core::Error::Protocol("raw-error-secret".into()),
    );
    assert!(telemetry::connect_database(
        "postgres://user-secret:password-secret@host:invalid-port/db"
    )
    .await
    .is_err());
    let events = logs.events();
    assert_eq!(events[0]["fields"]["error_kind"], "query_error");
    assert_eq!(events[1]["fields"]["stage"], "database_config");
    assert!(!logs.text().contains("secret"));
}

#[tokio::test(flavor = "current_thread")]
async fn health_router_is_traced_without_database() {
    let logs = Logs::default();
    let _guard = tracing::subscriber::set_default(telemetry::subscriber(
        telemetry::log_filter(None).unwrap(),
        logs.clone(),
    ));
    app()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.events().len(), 1);
}

#[test]
fn startup_rejects_configuration_and_invalid_filters_without_echoing_secrets() {
    let dir = std::env::temp_dir().join(format!("sara-logging-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_the-sara-protocol"))
        .current_dir(&dir)
        .env_clear()
        .env("RUST_LOG", "[invalid-filter-secret")
        .env("HOST", "host-secret")
        .env(
            "DATABASE_URL",
            "postgres://user-secret:pass-secret@localhost/db",
        )
        .output()
        .unwrap();
    std::fs::remove_dir(&dir).unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let text = String::from_utf8(output.stderr).unwrap();
    let events: Vec<Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(events
        .iter()
        .any(|e| e["fields"]["error_kind"] == "invalid_filter"));
    assert!(events
        .iter()
        .any(|e| e["fields"]["stage"] == "config" && e["level"] == "ERROR"));
    assert!(!text.contains("secret"));
}

#[test]
fn configuration_debug_redacts_connection_credentials() {
    let config = the_sara_protocol::config::Config {
        host: "127.0.0.1".parse().unwrap(),
        port: 3000,
        database_url: "postgres://user-secret:password-secret@localhost/db".into(),
    };
    let debug = format!("{config:?}");
    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("secret"));
}

#[tokio::test(flavor = "current_thread")]
async fn postgres_rejection_keeps_sensitive_input_out_of_debug_logs() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        return;
    };
    let logs = Logs::default();
    let _guard = tracing::subscriber::set_default(telemetry::subscriber(
        telemetry::log_filter(Some("debug")).unwrap(),
        logs.clone(),
    ));
    let pool = telemetry::connect_database(&url).await.unwrap();
    sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();
    let (id,): (i64,) = sqlx_core::query_as::query_as(
        "INSERT INTO personas (name) VALUES ('logging-test-persona-secret') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let response = app_with_pool(pool.clone()).oneshot(Request::builder().method("POST")
        .uri(format!("/personas/{id}/memories"))
        .header("content-type", "application/json")
        .body(Body::from(json!({"content":"payload-secret", "source":"source-secret", "observed_at":"timestamp-secret", "confirmation_status":"confirmed"}).to_string())).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    sqlx_core::query::query("DELETE FROM personas WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
    let events = logs.events();
    let error = events
        .iter()
        .find(|e| e["fields"]["error_kind"] == "invalid_timestamp")
        .unwrap();
    let response = events
        .iter()
        .find(|e| e["fields"]["status"] == 400)
        .unwrap();
    assert_eq!(error["level"], "WARN");
    assert_eq!(error["span"]["request_id"], response["span"]["request_id"]);
    assert!(!logs.text().contains("secret"));
    assert!(!logs.text().contains(&url));
}

#[test]
fn startup_logs_migration_failure_without_dumping_error_details() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        return;
    };
    let dir = std::env::temp_dir().join(format!("sara-migration-log-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_the-sara-protocol"))
        .current_dir(&dir)
        .env_clear()
        .env("RUST_LOG", "debug")
        .env("DATABASE_URL", &url)
        .output()
        .unwrap();
    std::fs::remove_dir(&dir).unwrap();
    assert!(!output.status.success());
    let text = String::from_utf8(output.stderr).unwrap();
    let events: Vec<Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(events.iter().any(|e| e["fields"]["stage"] == "startup"));
    assert!(events.iter().any(|e| e["fields"]["stage"] == "database"));
    assert!(events
        .iter()
        .any(|e| e["fields"]["stage"] == "migration_load" && e["level"] == "ERROR"));
    assert!(!text.contains(&url));
    assert!(!text.contains("sara_ci_test"));
}
