use axum::{
    body::{to_bytes, Body},
    http::Request,
    response::Response,
    Router,
};
use serde_json::Value;
use sqlx_postgres::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};
use tower::ServiceExt;

pub struct Api {
    pub pool: PgPool,
    pub router: Router,
    pub name: String,
}

impl Api {
    pub async fn new() -> Option<Self> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("Skipping persona integration test: DATABASE_URL is unset");
            return None;
        };
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations"))
            .await
            .unwrap()
            .run(&pool)
            .await
            .unwrap();
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Some(Self {
            router: the_sara_protocol::app_with_pool(pool.clone()),
            pool,
            name: format!(
                "persona_api_{}_{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ),
        })
    }

    pub async fn request(&self, method: &str, uri: &str, body: Option<Value>) -> Response {
        self.router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(body.map_or_else(Body::empty, |v| Body::from(v.to_string())))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn create(&self, name: &str) -> Value {
        let response = self
            .request("POST", "/personas", Some(serde_json::json!({"name":name})))
            .await;
        assert_eq!(response.status(), 201);
        json(response).await
    }

    pub async fn remove(&self, id: i64) {
        sqlx_core::query::query("DELETE FROM personas WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub async fn count_prefix(&self) -> i64 {
        let (count,): (i64,) = sqlx_core::query_as::query_as(
            "SELECT COUNT(*) FROM personas WHERE starts_with(name, $1)",
        )
        .bind(&self.name)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        count
    }

    pub async fn count_name(&self, name: &str) -> i64 {
        let (count,): (i64,) =
            sqlx_core::query_as::query_as("SELECT COUNT(*) FROM personas WHERE name = $1")
                .bind(name)
                .fetch_one(&self.pool)
                .await
                .unwrap();
        count
    }
}

pub async fn json(response: Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
