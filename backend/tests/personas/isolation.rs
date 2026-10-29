use super::support::{json, Api};
use serde_json::json;

#[tokio::test]
async fn personas_and_their_memories_remain_isolated() {
    let Some(api) = Api::new().await else {
        return;
    };
    let first = api.create(&api.name).await;
    let second = api.create(&format!("{}_other", api.name)).await;
    let first_path = format!("/personas/{}", first["id"]);
    let second_path = format!("/personas/{}", second["id"]);
    let response = api
        .request(
            "POST",
            &format!("{first_path}/memories"),
            Some(json!({
                "content":"only first persona", "source":"persona API test",
                "observed_at":"2026-10-29T12:00:00Z", "confirmation_status":"confirmed"
            })),
        )
        .await;
    assert_eq!(response.status(), 201);
    let memory = json(response).await;
    assert_ne!(first["id"], second["id"]);
    assert_eq!(
        json(api.request("GET", &first_path, None).await).await,
        first
    );
    assert_eq!(
        json(api.request("GET", &second_path, None).await).await,
        second
    );
    assert_eq!(
        json(
            api.request("GET", &format!("{first_path}/memories"), None)
                .await
        )
        .await,
        json!([memory])
    );
    assert_eq!(
        json(
            api.request("GET", &format!("{second_path}/memories"), None)
                .await
        )
        .await,
        json!([])
    );
    api.remove(first["id"].as_i64().unwrap()).await;
    api.remove(second["id"].as_i64().unwrap()).await;
}
