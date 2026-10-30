use super::support::{send, Contract};
use serde_json::json;
use std::collections::BTreeSet;

#[tokio::test]
async fn full_document_has_all_five_operations_and_unique_ids() {
    let contract = Contract::offline().await;
    assert_eq!(contract.doc["openapi"], "3.1.0");
    let paths = contract.doc["paths"].as_object().unwrap();
    assert_eq!(
        paths.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "/health",
            "/personas",
            "/personas/{id}",
            "/personas/{id}/memories"
        ])
    );
    let ids: Vec<_> = paths
        .values()
        .flat_map(|path| path.as_object().unwrap().values())
        .map(|operation| operation["operationId"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 5);
    assert_eq!(ids.iter().collect::<BTreeSet<_>>().len(), 5);
    assert_eq!(contract.doc["servers"][0]["url"], "/");
}

#[tokio::test]
async fn health_response_matches_generated_schema() {
    let contract = Contract::offline().await;
    let response = send(&contract.router, "GET", "/health", None, None).await;
    assert_eq!(response.status(), 200);
    assert_eq!(
        contract.response("/health", "get", response).await.unwrap(),
        json!({"status":"ok"})
    );
}

#[tokio::test]
async fn health_only_app_documents_only_its_registered_endpoint() {
    let contract = Contract::new(the_sara_protocol::app()).await;
    assert_eq!(contract.doc["paths"].as_object().unwrap().len(), 1);
    assert!(contract.doc["paths"].get("/health").is_some());
}

#[tokio::test]
async fn request_schemas_capture_required_fields_and_enum_values() {
    let contract = Contract::offline().await;
    let schemas = &contract.doc["components"]["schemas"];
    let persona = &schemas["CreatePersona"];
    assert_eq!(persona["additionalProperties"], false);
    contract.validate(persona, &json!({"name":"Example"}));
    let memory = &schemas["CreateMemory"];
    contract.validate(memory, &json!({"content":"example", "source":"test", "observed_at":"2026-10-29T12:00:00Z", "confirmation_status":"confirmed"}));
    assert_eq!(memory["additionalProperties"], false);
    assert_eq!(
        schemas["ConfirmationStatus"]["enum"],
        json!(["confirmed", "inferred"])
    );
}

#[tokio::test]
async fn schema_validator_detects_contract_violations() {
    let contract = Contract::offline().await;
    let schema = json!({"$ref":"#/components/schemas/HealthResponse", "components":contract.doc["components"]});
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&json!({"status":"ok"})));
    assert!(!validator.is_valid(&json!({"status":42})));
    assert!(!validator.is_valid(&json!({})));
}
