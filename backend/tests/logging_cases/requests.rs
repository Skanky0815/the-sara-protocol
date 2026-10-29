use super::support::{closed_pool_router, health_request, memory_request, Logs};
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use the_sara_protocol::app;
use tower::ServiceExt;

#[tokio::test(flavor = "current_thread")]
async fn health_event_has_safe_route_method_status_and_duration() {
    let (logs, _guard) = Logs::capture(None);
    let response = app().oneshot(health_request()).await.unwrap();
    assert_eq!(response.status(), 200);
    let events = logs.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["fields"]["message"], "request completed");
    assert_eq!(events[0]["fields"]["status"], 200);
    assert!(events[0]["fields"]["duration_ms"].as_f64().unwrap() >= 0.0);
    assert_eq!(events[0]["span"]["route"], "/health");
    assert_eq!(events[0]["span"]["method"], "GET");
}

#[tokio::test(flavor = "current_thread")]
async fn server_error_is_correlated_without_exposing_internal_details() {
    let (logs, _guard) = Logs::capture(Some("debug"));
    let response = closed_pool_router()
        .await
        .oneshot(memory_request(88442211, "2026-10-28T12:00:00Z"))
        .await
        .unwrap();
    assert_eq!(response.status(), 500);
    assert!(to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .is_empty());
    let events = logs.events();
    let error = events
        .iter()
        .find(|e| e["fields"]["error_kind"] == "pool_closed")
        .unwrap();
    let completed = events
        .iter()
        .find(|e| e["fields"]["message"] == "request completed")
        .unwrap();
    assert_eq!(error["fields"]["operation"], "create_memory");
    assert_eq!(error["span"]["request_id"], completed["span"]["request_id"]);
    assert_eq!(error["level"], "ERROR");
    assert_eq!(completed["level"], "ERROR");
    assert_eq!(completed["fields"]["status"], 500);
    assert!(completed["fields"]["duration_ms"].as_f64().unwrap() >= 0.0);
    assert_eq!(completed["span"]["route"], "/personas/{id}/memories");
    assert_eq!(completed["span"]["method"], "POST");
}

#[tokio::test(flavor = "current_thread")]
async fn unmatched_route_is_a_sanitized_client_error() {
    let (logs, _guard) = Logs::capture(Some("debug"));
    let request = Request::builder()
        .uri("/unknown-path-secret")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        closed_pool_router()
            .await
            .oneshot(request)
            .await
            .unwrap()
            .status(),
        404
    );
    let events = logs.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["span"]["route"], "<unmatched>");
    assert_eq!(events[0]["span"]["method"], "GET");
    assert_eq!(events[0]["fields"]["status"], 404);
    assert!(events[0]["fields"]["duration_ms"].as_f64().unwrap() >= 0.0);
    assert_eq!(events[0]["level"], "WARN");
    assert!(!logs.text().contains("secret"));
}

#[tokio::test(flavor = "current_thread")]
async fn requests_get_distinct_identifiers() {
    let (logs, _guard) = Logs::capture(None);
    let router = app();
    router.clone().oneshot(health_request()).await.unwrap();
    router.oneshot(health_request()).await.unwrap();
    let events = logs.events();
    assert_eq!(events.len(), 2);
    assert!(events[0]["span"]["request_id"].is_u64());
    assert!(events[1]["span"]["request_id"].is_u64());
    assert_ne!(
        events[0]["span"]["request_id"],
        events[1]["span"]["request_id"]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn debug_logs_exclude_headers_queries_parameters_and_bodies() {
    let (logs, _guard) = Logs::capture(Some("debug"));
    let router = closed_pool_router().await;
    router.clone().oneshot(health_request()).await.unwrap();
    router
        .oneshot(memory_request(88442211, "2026-10-28T12:00:00Z"))
        .await
        .unwrap();
    assert!(!logs.events().is_empty());
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
