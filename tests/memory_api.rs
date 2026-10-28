use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    response::Response,
    Router,
};
use serde_json::{json, Value};
use sqlx_postgres::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};
use tower::ServiceExt;

fn valid_memory() -> Value {
    json!({
        "content": "A remembered fact",
        "source": "conversation",
        "observed_at": "2026-10-28T12:00:00Z",
        "confirmation_status": "confirmed"
    })
}

struct TestApp {
    pool: PgPool,
    app: Router,
    persona_id: i64,
}

impl TestApp {
    async fn new() -> Option<Self> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("Skipping integration test: DATABASE_URL is unset");
            return None;
        };
        let pool = PgPool::connect(&url).await.expect("Postgres available");
        let migrations = sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations"))
            .await
            .expect("migrations load");
        migrations.run(&pool).await.expect("migrations apply");
        // Each concurrently running test owns a distinct persona.
        static NEXT_PERSONA: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "issue11_{}_{}",
            std::process::id(),
            NEXT_PERSONA.fetch_add(1, Ordering::Relaxed)
        );
        let (persona_id,): (i64,) =
            sqlx_core::query_as::query_as("INSERT INTO personas(name) VALUES ($1) RETURNING id")
                .bind(name)
                .fetch_one(&pool)
                .await
                .unwrap();
        Some(Self {
            app: the_sara_protocol::app_with_pool(pool.clone()),
            pool,
            persona_id,
        })
    }

    async fn request(&self, method: &str, persona: i64, body: Option<Value>) -> Response {
        self.app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(format!("/personas/{persona}/memories"))
                    .header("content-type", "application/json")
                    .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    async fn list(&self) -> Vec<Value> {
        let response = self.request("GET", self.persona_id, None).await;
        assert_eq!(response.status(), StatusCode::OK);
        serde_json::from_value(response_json(response).await).unwrap()
    }

    async fn create(&self) -> Value {
        let response = self
            .request("POST", self.persona_id, Some(valid_memory()))
            .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        response_json(response).await
    }

    async fn assert_rejected(&self, field: &str, value: &str, expected: StatusCode) {
        let mut body = valid_memory();
        body[field] = value.into();
        let response = self.request("POST", self.persona_id, Some(body)).await;
        assert_eq!(response.status(), expected, "{field} = {value:?}");
        let (count,): (i64,) =
            sqlx_core::query_as::query_as("SELECT COUNT(*) FROM memories WHERE persona_id = $1")
                .bind(self.persona_id)
                .fetch_one(&self.pool)
                .await
                .unwrap();
        assert_eq!(count, 0, "Rejected requests must not persist memories");
    }

    async fn cleanup(&self) {
        sqlx_core::query::query("DELETE FROM personas WHERE id = $1")
            .bind(self.persona_id)
            .execute(&self.pool)
            .await
            .unwrap();
    }
}

async fn response_json(response: Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn memories_persist_content_and_provenance() {
    let Some(test) = TestApp::new().await else {
        return;
    };
    let created = test.create().await;
    assert_eq!(created["persona_id"], test.persona_id);
    assert_eq!(created["content"], "A remembered fact");
    assert_eq!(created["source"], "conversation");
    assert_eq!(created["confirmation_status"], "confirmed");
    assert_eq!(created["observed_at"], "2026-10-28T12:00:00.000000Z");
    assert_eq!(test.list().await, vec![created]);
    test.cleanup().await;
}

#[tokio::test]
async fn memories_remain_persona_scoped() {
    let Some(first) = TestApp::new().await else {
        return;
    };
    let second = TestApp::new().await.unwrap();
    let created_first = first.create().await;
    assert_eq!(first.list().await, vec![created_first.clone()]);
    assert!(second.list().await.is_empty());
    let created_second = second.create().await;
    assert_eq!(first.list().await, vec![created_first]);
    assert_eq!(second.list().await, vec![created_second]);
    first.cleanup().await;
    second.cleanup().await;
}

#[tokio::test]
async fn unknown_persona_returns_not_found() {
    let Some(test) = TestApp::new().await else {
        return;
    };
    // Delete a real ID to ensure it is absent without guessing another test's ID.
    test.cleanup().await;
    for method in ["GET", "POST"] {
        let response = test
            .request(method, test.persona_id, Some(valid_memory()))
            .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{method}");
    }
}

#[tokio::test]
async fn nonpositive_persona_ids_are_rejected() {
    let Some(test) = TestApp::new().await else {
        return;
    };
    for persona in [0, -1] {
        for method in ["GET", "POST"] {
            let response = test.request(method, persona, Some(valid_memory())).await;
            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "{method} persona {persona}"
            );
        }
    }
    assert!(test.list().await.is_empty());
    test.cleanup().await;
}

#[tokio::test]
async fn blank_content_is_rejected_without_persisting() {
    let Some(test) = TestApp::new().await else {
        return;
    };
    for value in ["", " \n\t"] {
        test.assert_rejected("content", value, StatusCode::BAD_REQUEST)
            .await;
    }
    test.cleanup().await;
}

#[tokio::test]
async fn blank_source_is_rejected_without_persisting() {
    let Some(test) = TestApp::new().await else {
        return;
    };
    for value in ["", " \n\t"] {
        test.assert_rejected("source", value, StatusCode::BAD_REQUEST)
            .await;
    }
    test.cleanup().await;
}

#[tokio::test]
async fn malformed_timestamp_is_rejected_without_persisting() {
    let Some(test) = TestApp::new().await else {
        return;
    };
    test.assert_rejected("observed_at", "not-a-timestamp", StatusCode::BAD_REQUEST)
        .await;
    test.cleanup().await;
}

#[tokio::test]
async fn out_of_range_timestamp_is_rejected_without_persisting() {
    let Some(test) = TestApp::new().await else {
        return;
    };
    test.assert_rejected(
        "observed_at",
        "2026-13-28T12:00:00Z",
        StatusCode::BAD_REQUEST,
    )
    .await;
    test.cleanup().await;
}

#[tokio::test]
async fn unsupported_confirmation_status_is_rejected_without_persisting() {
    let Some(test) = TestApp::new().await else {
        return;
    };
    test.assert_rejected(
        "confirmation_status",
        "unknown",
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    test.cleanup().await;
}
