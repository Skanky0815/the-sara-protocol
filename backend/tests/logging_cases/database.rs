use super::support::{memory_request, unique_name, Logs};
use the_sara_protocol::{app_with_pool, telemetry};
use tower::ServiceExt;

#[tokio::test(flavor = "current_thread")]
async fn raw_database_error_is_replaced_by_category() {
    let (logs, _guard) = Logs::capture(Some("debug"));
    telemetry::database_error(
        "read",
        &sqlx_core::Error::Protocol("raw-error-secret".into()),
    );
    let events = logs.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["fields"]["error_kind"], "query_error");
    assert!(!logs.text().contains("secret"));
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_database_url_is_not_echoed() {
    let (logs, _guard) = Logs::capture(Some("debug"));
    assert!(telemetry::connect_database(
        "postgres://user-secret:password-secret@host:invalid-port/db"
    )
    .await
    .is_err());
    let events = logs.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["fields"]["stage"], "database_config");
    assert!(!logs.text().contains("secret"));
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_timestamp_is_rejected_without_database_error_or_sensitive_logs() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        return;
    };
    let (logs, _guard) = Logs::capture(Some("debug"));
    let pool = telemetry::connect_database(&url).await.unwrap();
    sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();
    let (id,): (i64,) =
        sqlx_core::query_as::query_as("INSERT INTO personas (name) VALUES ($1) RETURNING id")
            .bind(unique_name())
            .fetch_one(&pool)
            .await
            .unwrap();
    let response = app_with_pool(pool.clone())
        .oneshot(memory_request(id, "timestamp-secret"))
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    sqlx_core::query::query("DELETE FROM personas WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
    let events = logs.events();
    assert!(events.iter().all(|e| e["fields"]["error_kind"].is_null()));
    let completed = events
        .iter()
        .find(|e| e["fields"]["status"] == 400)
        .unwrap();
    assert!(completed["span"]["request_id"].is_u64());
    assert!(!logs.text().contains("secret"));
    assert!(!logs.text().contains(&url));
}
