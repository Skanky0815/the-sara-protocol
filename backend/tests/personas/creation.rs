use super::support::{json, Api};
use serde_json::json;

#[tokio::test]
async fn create_returns_location_and_persisted_fields() {
    let Some(api) = Api::new().await else {
        return;
    };
    let response = api
        .request(
            "POST",
            "/personas",
            Some(json!({"name":format!("  {}  ", api.name)})),
        )
        .await;
    assert_eq!(response.status(), 201);
    let location = response.headers()["location"].to_str().unwrap().to_owned();
    let value = json(response).await;
    let id = value["id"].as_i64().unwrap();
    assert!(id > 0);
    assert_eq!(location, format!("/personas/{id}"));
    assert_eq!(value["name"], api.name);
    assert!(value["created_at"].as_str().unwrap().ends_with('Z'));
    assert_eq!(value.as_object().unwrap().len(), 3);
    let stored = api.request("GET", &location, None).await;
    assert_eq!(stored.status(), 200);
    assert_eq!(json(stored).await, value);
    assert_eq!(api.count_name(&api.name).await, 1);
    api.remove(id).await;
}

#[tokio::test]
async fn duplicate_name_returns_conflict_without_second_row() {
    let Some(api) = Api::new().await else {
        return;
    };
    let value = api.create(&api.name).await;
    let response = api
        .request(
            "POST",
            "/personas",
            Some(json!({"name":format!(" {} ", api.name)})),
        )
        .await;
    assert_eq!(response.status(), 409);
    assert_eq!(api.count_name(&api.name).await, 1);
    api.remove(value["id"].as_i64().unwrap()).await;
}

#[tokio::test]
async fn blank_long_and_control_character_names_are_rejected_without_writes() {
    let Some(api) = Api::new().await else {
        return;
    };
    for name in [
        "".to_owned(),
        " \t\n".to_owned(),
        format!("{}{}", api.name, "x".repeat(201)),
        format!("{}\0", api.name),
        format!("{}\nname", api.name),
    ] {
        let response = api
            .request("POST", "/personas", Some(json!({"name":name})))
            .await;
        assert_eq!(response.status(), 400);
        assert_eq!(api.count_prefix().await, 0);
    }
}

#[tokio::test]
async fn missing_wrong_type_and_unknown_fields_are_rejected() {
    let Some(api) = Api::new().await else {
        return;
    };
    for body in [
        json!({}),
        json!({"name":42}),
        json!({"name":null}),
        json!({"name":api.name,"id":1}),
    ] {
        let response = api.request("POST", "/personas", Some(body)).await;
        assert_eq!(response.status(), 422);
    }
    assert_eq!(api.count_name(&api.name).await, 0);
}

#[tokio::test]
async fn unicode_name_at_character_limit_is_accepted() {
    let Some(api) = Api::new().await else {
        return;
    };
    let name = format!(
        "{}{}",
        api.name,
        "\u{1f43e}".repeat(200 - api.name.chars().count())
    );
    let value = api.create(&name).await;
    assert_eq!(value["name"], name);
    api.remove(value["id"].as_i64().unwrap()).await;
}

#[tokio::test]
async fn concurrent_creates_of_same_name_produce_one_persona() {
    let Some(api) = Api::new().await else {
        return;
    };
    let body = json!({"name":api.name});
    let (first, second) = tokio::join!(
        api.request("POST", "/personas", Some(body.clone())),
        api.request("POST", "/personas", Some(body))
    );
    let mut statuses = [first.status().as_u16(), second.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [201, 409]);
    assert_eq!(api.count_name(&api.name).await, 1);
    let created = if first.status() == 201 { first } else { second };
    api.remove(json(created).await["id"].as_i64().unwrap())
        .await;
}
