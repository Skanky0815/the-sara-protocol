use super::support::{send, Contract};
use serde_json::json;

#[tokio::test]
async fn path_validation_and_server_errors_match_described_responses() {
    let contract = Contract::offline().await;
    for template in ["/personas/{id}", "/personas/{id}/memories"] {
        for (id, status) in [
            ("0", 400),
            ("invalid", 400),
            ("9223372036854775808", 400),
            ("1", 500),
        ] {
            let response = send(
                &contract.router,
                "GET",
                &template.replace("{id}", id),
                None,
                None,
            )
            .await;
            assert_eq!(response.status(), status);
            contract.response(template, "get", response).await;
        }
    }
}

#[tokio::test]
async fn persona_json_rejections_match_generated_contract() {
    let contract = Contract::offline().await;
    for (body, kind, status) in [
        ("{".to_owned(), Some("application/json"), 400),
        (
            json!({"name":" "}).to_string(),
            Some("application/json"),
            400,
        ),
        (json!({}).to_string(), Some("application/json"), 422),
        (
            json!({"name":42}).to_string(),
            Some("application/json"),
            422,
        ),
        (json!({"name":"x"}).to_string(), None, 415),
    ] {
        let response = send(&contract.router, "POST", "/personas", Some(body), kind).await;
        assert_eq!(response.status(), status);
        contract.response("/personas", "post", response).await;
    }
}

#[tokio::test]
async fn memory_json_rejections_match_generated_contract() {
    let contract = Contract::offline().await;
    for (body, kind, status) in [
        ("{".to_owned(), Some("application/json"), 400),
        (json!({}).to_string(), Some("application/json"), 422),
        (json!({}).to_string(), None, 415),
    ] {
        let response = send(
            &contract.router,
            "POST",
            "/personas/1/memories",
            Some(body),
            kind,
        )
        .await;
        assert_eq!(response.status(), status);
        contract
            .response("/personas/{id}/memories", "post", response)
            .await;
    }
}

#[tokio::test]
async fn oversized_json_is_documented_for_both_post_routes() {
    let contract = Contract::offline().await;
    for (path, template) in [
        ("/personas", "/personas"),
        ("/personas/1/memories", "/personas/{id}/memories"),
    ] {
        let response = send(
            &contract.router,
            "POST",
            path,
            Some("x".repeat(2 * 1024 * 1024 + 1)),
            Some("application/json"),
        )
        .await;
        assert_eq!(response.status(), 413);
        contract.response(template, "post", response).await;
    }
}

#[tokio::test]
async fn post_database_errors_match_documented_empty_responses() {
    let contract = Contract::offline().await;
    for (path, template, body) in [
        ("/personas", "/personas", json!({"name":"test"})),
        (
            "/personas/1/memories",
            "/personas/{id}/memories",
            json!({"content":"test","source":"test","observed_at":"2026-10-29T12:00:00Z","confirmation_status":"confirmed"}),
        ),
    ] {
        let response = send(
            &contract.router,
            "POST",
            path,
            Some(body.to_string()),
            Some("application/json"),
        )
        .await;
        assert_eq!(response.status(), 500);
        assert!(contract
            .response(template, "post", response)
            .await
            .is_none());
    }
}
