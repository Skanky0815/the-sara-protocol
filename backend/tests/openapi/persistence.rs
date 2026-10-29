use super::support::{send, PersonaCase};
use serde_json::json;

#[tokio::test]
async fn persona_creation_has_documented_location_and_response_schema() {
    let Some(case) = PersonaCase::new().await else {
        return;
    };
    assert_eq!(case.location, format!("/personas/{}", case.persona["id"]));
    assert!(
        case.contract.doc["paths"]["/personas"]["post"]["responses"]["201"]["headers"]
            .get("Location")
            .is_some()
    );
    case.remove().await;
}

#[tokio::test]
async fn persona_lookup_matches_generated_response_schema() {
    let Some(case) = PersonaCase::new().await else {
        return;
    };
    let response = send(&case.contract.router, "GET", &case.location, None, None).await;
    assert_eq!(response.status(), 200);
    assert_eq!(
        case.contract
            .response("/personas/{id}", "get", response)
            .await
            .unwrap(),
        case.persona
    );
    case.remove().await;
}

#[tokio::test]
async fn persona_conflict_matches_documented_empty_response() {
    let Some(case) = PersonaCase::new().await else {
        return;
    };
    let response = send(
        &case.contract.router,
        "POST",
        "/personas",
        Some(json!({"name":case.persona["name"]}).to_string()),
        Some("application/json"),
    )
    .await;
    assert_eq!(response.status(), 409);
    assert!(case
        .contract
        .response("/personas", "post", response)
        .await
        .is_none());
    case.remove().await;
}

#[tokio::test]
async fn missing_persona_get_responses_match_contract() {
    let Some(case) = PersonaCase::new().await else {
        return;
    };
    case.remove().await;
    for (path, template) in [
        (case.location.clone(), "/personas/{id}"),
        (
            format!("{}/memories", case.location),
            "/personas/{id}/memories",
        ),
    ] {
        let response = send(&case.contract.router, "GET", &path, None, None).await;
        assert_eq!(response.status(), 404);
        assert!(case
            .contract
            .response(template, "get", response)
            .await
            .is_none());
    }
}

fn memory_body() -> String {
    json!({"content":"test memory", "source":"OpenAPI test", "observed_at":"2026-10-29T12:00:00Z", "confirmation_status":"inferred"}).to_string()
}

#[tokio::test]
async fn memory_create_and_list_follow_generated_schemas() {
    let Some(case) = PersonaCase::new().await else {
        return;
    };
    let path = format!("{}/memories", case.location);
    let response = send(
        &case.contract.router,
        "POST",
        &path,
        Some(memory_body()),
        Some("application/json"),
    )
    .await;
    assert_eq!(response.status(), 201);
    let memory = case
        .contract
        .response("/personas/{id}/memories", "post", response)
        .await
        .unwrap();
    let response = send(&case.contract.router, "GET", &path, None, None).await;
    assert_eq!(response.status(), 200);
    assert_eq!(
        case.contract
            .response("/personas/{id}/memories", "get", response)
            .await
            .unwrap(),
        json!([memory])
    );
    case.remove().await;
}

#[tokio::test]
async fn creating_memory_for_missing_persona_matches_contract() {
    let Some(case) = PersonaCase::new().await else {
        return;
    };
    case.remove().await;
    let response = send(
        &case.contract.router,
        "POST",
        &format!("{}/memories", case.location),
        Some(memory_body()),
        Some("application/json"),
    )
    .await;
    assert_eq!(response.status(), 404);
    assert!(case
        .contract
        .response("/personas/{id}/memories", "post", response)
        .await
        .is_none());
}
