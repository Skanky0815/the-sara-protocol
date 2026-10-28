pub mod config;
pub mod memories;

use axum::{routing::get, Json, Router};
use memories::AppState;
use serde::Serialize;
use sqlx_postgres::PgPool;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub fn app() -> Router {
    Router::new().route("/health", get(health))
}

pub fn app_with_pool(pool: PgPool) -> Router {
    app().merge(memories::routes().with_state(AppState { pool }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_ok() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
