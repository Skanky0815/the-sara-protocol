pub mod config;
pub mod memories;
pub mod personas;
pub mod telemetry;

use axum::{routing::get, Json, Router};
use memories::AppState;
use serde::Serialize;
use sqlx_postgres::PgPool;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    #[schema(example = "ok")]
    status: &'static str,
}

/// Dienst ist bereit, HTTP-Anfragen zu verarbeiten.
#[utoipa::path(get, path = "/health", operation_id = "getHealth", tag = "Health",
    responses((status = 200, description = "Dienst erreichbar", body = HealthResponse)))]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

fn with_documentation(router: OpenApiRouter) -> Router {
    let (router, mut document) = router.split_for_parts();
    document.info.title = "The Sara Protocol".into();
    document.info.description = Some(
        "Lokale Persona- und Memory-API. Benutzer-Authentifizierung ist noch nicht implementiert."
            .into(),
    );
    document.servers = Some(vec![utoipa::openapi::Server::new("/")]);
    telemetry::trace_requests(
        router.route("/openapi.json", get(move || async move { Json(document) })),
    )
}

pub fn app() -> Router {
    with_documentation(OpenApiRouter::new().routes(routes!(health)))
}

pub fn app_with_pool(pool: PgPool) -> Router {
    with_documentation(
        OpenApiRouter::new()
            .routes(routes!(health))
            .merge(memories::routes())
            .merge(personas::routes())
            .with_state(AppState { pool }),
    )
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
