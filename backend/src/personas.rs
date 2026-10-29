use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx_core::row::Row;
use sqlx_postgres::PgRow;

use crate::{memories::AppState, telemetry};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatePersona {
    name: String,
}

#[derive(Serialize)]
pub struct Persona {
    id: i64,
    name: String,
    created_at: String,
}

fn persona(row: PgRow) -> Persona {
    Persona {
        id: row.get("id"),
        name: row.get("name"),
        created_at: row.get("created_at"),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/personas", post(create))
        .route("/personas/{id}", get(find))
}

async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreatePersona>,
) -> Result<(StatusCode, [(header::HeaderName, String); 1], Json<Persona>), StatusCode> {
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 200 || name.chars().any(char::is_control) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let row = sqlx_core::query::query(
        "INSERT INTO personas (name) VALUES ($1)
         RETURNING id, name,
           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS created_at",
    )
    .bind(name)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        if error.as_database_error().is_some_and(|db| db.is_unique_violation()) {
            tracing::warn!(operation = "create_persona", error_kind = "name_conflict", "persona name already exists");
            StatusCode::CONFLICT
        } else {
            telemetry::database_error("create_persona", &error);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    let value = persona(row);
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/personas/{}", value.id))],
        Json(value),
    ))
}

async fn find(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Persona>, StatusCode> {
    if id <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let row = sqlx_core::query::query(
        "SELECT id, name,
           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS created_at
         FROM personas WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|error| {
        telemetry::database_error("get_persona", &error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    row.map(|row| Json(persona(row)))
        .ok_or(StatusCode::NOT_FOUND)
}
