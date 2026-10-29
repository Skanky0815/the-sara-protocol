use super::support::{json, Api};
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn get_unknown_positive_id_returns_not_found() {
    let Some(api) = Api::new().await else {
        return;
    };
    let value = api.create(&api.name).await;
    let id = value["id"].as_i64().unwrap();
    api.remove(id).await;
    assert_eq!(
        api.request("GET", &format!("/personas/{id}"), None)
            .await
            .status(),
        404
    );
}

#[tokio::test]
async fn invalid_ids_are_bad_requests() {
    let Some(api) = Api::new().await else {
        return;
    };
    for id in ["0", "-1", "abc", "9223372036854775808"] {
        assert_eq!(
            api.request("GET", &format!("/personas/{id}"), None)
                .await
                .status(),
            400
        );
    }
}

#[tokio::test]
async fn database_failures_remain_empty_internal_server_errors() {
    let pool =
        sqlx_postgres::PgPool::connect_lazy("postgres://local:secret@localhost/test").unwrap();
    pool.close().await;
    let router = the_sara_protocol::app_with_pool(pool);
    for (method, path, body) in [
        ("GET", "/personas/1", Body::empty()),
        (
            "POST",
            "/personas",
            Body::from(json!({"name":"test"}).to_string()),
        ),
    ] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 500);
        assert!(to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .is_empty());
    }
}

#[tokio::test]
async fn names_with_sql_punctuation_are_data_and_do_not_affect_other_personas() {
    let Some(api) = Api::new().await else {
        return;
    };
    let first = api.create(&api.name).await;
    let second = api
        .create(&format!("{}'; DROP TABLE personas;--", api.name))
        .await;
    let response = api
        .request("GET", &format!("/personas/{}", first["id"]), None)
        .await;
    assert_eq!(response.status(), 200);
    assert_eq!(json(response).await, first);
    api.remove(first["id"].as_i64().unwrap()).await;
    api.remove(second["id"].as_i64().unwrap()).await;
}
