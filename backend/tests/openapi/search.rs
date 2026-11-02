use super::support::{send, Contract, PersonaCase};
use serde_json::json;

const PATH: &str = "/personas/{id}/memories/search";

#[tokio::test]
async fn search_validation_and_database_failures_match_contract() {
    let contract = Contract::offline().await;
    for query in [
        "".to_owned(),
        "q=".into(),
        "q=+++".into(),
        "q=%00".into(),
        format!("q={}", "x".repeat(501)),
        format!("q={}", "%C3%A4".repeat(251)),
        "q=x&limit=0".into(),
        "q=x&limit=101".into(),
        "q=x&limit=-1".into(),
        "q=x&limit=invalid".into(),
        "q=x&extra=y".into(),
        "q=x&q=y".into(),
    ] {
        let response = send(
            &contract.router,
            "GET",
            &format!("/personas/1/memories/search?{query}"),
            None,
            None,
        )
        .await;
        assert_eq!(response.status(), 400, "{query}");
        contract.response(PATH, "get", response).await;
    }
    for (id, expected) in [
        ("0", 400),
        ("-1", 400),
        ("invalid", 400),
        ("9223372036854775808", 400),
        ("1", 500),
    ] {
        let response = send(
            &contract.router,
            "GET",
            &format!("/personas/{id}/memories/search?q=Katze"),
            None,
            None,
        )
        .await;
        assert_eq!(response.status(), expected);
        contract.response(PATH, "get", response).await;
    }
    let parameters = contract.doc["paths"][PATH]["get"]["parameters"]
        .as_array()
        .unwrap();
    let q = parameters.iter().find(|p| p["name"] == "q").unwrap();
    assert_eq!(q["required"], true);
    let limit = parameters.iter().find(|p| p["name"] == "limit").unwrap();
    assert_eq!(limit["schema"]["maximum"], 100);
}

#[tokio::test]
async fn search_success_empty_and_not_found_match_contract() {
    let Some(test) = PersonaCase::new().await else {
        return;
    };
    let response = send(&test.contract.router, "POST", &format!("{}/memories", test.location), Some(json!({
        "content":"Katzen trinken Kaffee", "source":"synthetic contract test", "observed_at":"2026-11-02T08:00:00Z", "confirmation_status":"inferred"
    }).to_string()), Some("application/json")).await;
    assert_eq!(response.status(), 201);
    let memory = test
        .contract
        .response("/personas/{id}/memories", "post", response)
        .await
        .unwrap();
    for (q, expected) in [("Katze", json!([memory])), ("Hund", json!([]))] {
        let response = send(
            &test.contract.router,
            "GET",
            &format!("{}/memories/search?q={q}", test.location),
            None,
            None,
        )
        .await;
        assert_eq!(response.status(), 200);
        assert_eq!(
            test.contract.response(PATH, "get", response).await.unwrap(),
            expected
        );
    }
    test.remove().await;
    let response = send(
        &test.contract.router,
        "GET",
        &format!("{}/memories/search?q=Katze", test.location),
        None,
        None,
    )
    .await;
    assert_eq!(response.status(), 404);
    test.contract.response(PATH, "get", response).await;
}
