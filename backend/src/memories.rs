use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx_core::row::Row;
use sqlx_postgres::{PgPool, PgRow};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateMemory {
    pub content: String,
    pub source: String,
    pub observed_at: String,
    pub confirmation_status: ConfirmationStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationStatus {
    Confirmed,
    Inferred,
}

impl ConfirmationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Inferred => "inferred",
        }
    }
}

#[derive(Serialize)]
pub struct Memory {
    pub id: i64,
    pub persona_id: i64,
    pub content: String,
    pub source: String,
    pub observed_at: String,
    pub confirmation_status: String,
}

fn map_row(row: PgRow) -> Memory {
    Memory {
        id: row.get("id"),
        persona_id: row.get("persona_id"),
        content: row.get("content"),
        source: row.get("source"),
        observed_at: row.get("observed_at"),
        confirmation_status: row.get("confirmation_status"),
    }
}

fn valid_input(input: &CreateMemory) -> bool {
    !input.content.trim().is_empty()
        && !input.source.trim().is_empty()
        && input.content.len() <= 100_000
        && input.source.len() <= 2_000
}

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/personas/{id}/memories",
        get(list_memories).post(create_memory),
    )
}

async fn create_memory(
    State(state): State<AppState>,
    Path(persona_id): Path<i64>,
    Json(input): Json<CreateMemory>,
) -> Result<(StatusCode, Json<Memory>), StatusCode> {
    if persona_id <= 0 || !valid_input(&input) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let row = sqlx_core::query::query(
        "INSERT INTO memories (persona_id, content, source, observed_at, confirmation_status)
         SELECT id, $2, $3, $4::timestamptz, $5 FROM personas WHERE id = $1
         RETURNING id, persona_id, content, source,
                   to_char(observed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS observed_at,
                   confirmation_status",
    )
    .bind(persona_id)
    .bind(input.content)
    .bind(input.source)
    .bind(input.observed_at)
    .bind(input.confirmation_status.as_str())
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        crate::telemetry::database_error("create_memory", &err);
        if let sqlx_core::Error::Database(db) = &err {
            if db.code().as_deref() == Some("22007") || db.code().as_deref() == Some("22008") {
                return StatusCode::BAD_REQUEST;
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let row = row.ok_or(StatusCode::NOT_FOUND)?;
    Ok((StatusCode::CREATED, Json(map_row(row))))
}

async fn list_memories(
    State(state): State<AppState>,
    Path(persona_id): Path<i64>,
) -> Result<Json<Vec<Memory>>, StatusCode> {
    if persona_id <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let exists = sqlx_core::query::query("SELECT id FROM personas WHERE id = $1")
        .bind(persona_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|err| {
            crate::telemetry::database_error("find_persona", &err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    if exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let rows = sqlx_core::query::query(
        "SELECT id, persona_id, content, source,
                to_char(observed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS observed_at,
                confirmation_status
         FROM memories WHERE persona_id = $1 ORDER BY id",
    )
    .bind(persona_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|err| {
        crate::telemetry::database_error("list_memories", &err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(rows.into_iter().map(map_row).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_memory_input() {
        let mut input = CreateMemory {
            content: "Something remembered".into(),
            source: "conversation".into(),
            observed_at: "2026-10-28T12:00:00Z".into(),
            confirmation_status: ConfirmationStatus::Confirmed,
        };
        assert!(valid_input(&input));
        input.content = " ".into();
        assert!(!valid_input(&input));
    }
    #[test]
    fn status_serializes_as_expected() {
        assert_eq!(
            serde_json::to_string(&ConfirmationStatus::Inferred).unwrap(),
            "\"inferred\""
        );
    }
}
