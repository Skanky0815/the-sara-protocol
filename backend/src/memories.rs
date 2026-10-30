use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx_core::row::Row;
use sqlx_postgres::{PgPool, PgRow};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateMemory {
    /// Weder leer noch ausschließlich Whitespace; maximal 100000 UTF-8-Bytes.
    pub content: String,
    /// Weder leer noch ausschließlich Whitespace; maximal 2000 UTF-8-Bytes.
    pub source: String,
    /// RFC3339 mit Zeitzone, UTC-Jahr 0001–9999. Speicherung in UTC;
    /// Nachkommastellen unterhalb einer Mikrosekunde werden abgeschnitten.
    #[schema(format = DateTime)]
    pub observed_at: String,
    pub confirmation_status: ConfirmationStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
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

#[derive(Serialize, ToSchema)]
pub struct Memory {
    #[schema(minimum = 1)]
    pub id: i64,
    #[schema(minimum = 1)]
    pub persona_id: i64,
    /// Weder leer noch ausschließlich Whitespace; maximal 100000 UTF-8-Bytes.
    pub content: String,
    /// Weder leer noch ausschließlich Whitespace; maximal 2000 UTF-8-Bytes.
    pub source: String,
    #[schema(format = DateTime)]
    pub observed_at: String,
    #[schema(value_type = ConfirmationStatus)]
    pub confirmation_status: String,
}

fn normalize_observed_at(value: &str) -> Result<String, StatusCode> {
    let parsed = OffsetDateTime::parse(value, &Rfc3339).map_err(|_| StatusCode::BAD_REQUEST)?;
    let utc = parsed
        .checked_to_offset(UtcOffset::UTC)
        .ok_or(StatusCode::BAD_REQUEST)?;
    if !(1..=9999).contains(&utc.year()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    // Match PostgreSQL precision without rounding into the next second/year.
    let utc = utc
        .replace_nanosecond(utc.nanosecond() / 1_000 * 1_000)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    utc.format(&Rfc3339).map_err(|_| StatusCode::BAD_REQUEST)
}

fn map_row(row: PgRow) -> Result<Memory, StatusCode> {
    let decode = || -> Result<Memory, sqlx_core::Error> {
        Ok(Memory {
            id: row.try_get("id")?,
            persona_id: row.try_get("persona_id")?,
            content: row.try_get("content")?,
            source: row.try_get("source")?,
            observed_at: row.try_get("observed_at")?,
            confirmation_status: row.try_get("confirmation_status")?,
        })
    };
    let memory = decode().map_err(|err| {
        crate::telemetry::database_error("decode_memory", &err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    if normalize_observed_at(&memory.observed_at).is_err() {
        tracing::error!(
            operation = "decode_memory",
            error_kind = "invalid_stored_timestamp",
            "stored memory timestamp violates API contract"
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(memory)
}

fn valid_input(input: &CreateMemory) -> bool {
    !input.content.trim().is_empty()
        && !input.source.trim().is_empty()
        && input.content.len() <= 100_000
        && input.source.len() <= 2_000
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_memories, create_memory))
}

/// Memory mit Provenienz für eine Persona speichern.
#[utoipa::path(
    post, path = "/personas/{id}/memories", operation_id = "createMemory", tag = "Memories",
    params(("id" = i64, Path, minimum = 1, description = "Positive Persona-ID")),
    request_body = CreateMemory,
    responses(
        (status = 201, description = "Memory gespeichert", body = Memory),
        (status = 400, description = "Ungültige ID, Eingabe oder Zeitangabe; leerer Body bei fachlicher Validierung, sonst text/plain", body = String, content_type = "text/plain"),
        (status = 404, description = "Persona nicht gefunden; leerer Body"),
        (status = 413, description = "Request-Body zu groß", body = String, content_type = "text/plain"),
        (status = 415, description = "JSON-Content-Type fehlt oder ist nicht unterstützt", body = String, content_type = "text/plain"),
        (status = 422, description = "Fehlendes, unbekanntes oder falsch typisiertes JSON-Feld", body = String, content_type = "text/plain"),
        (status = 500, description = "Datenbankfehler; leerer Body")
    )
)]
async fn create_memory(
    State(state): State<AppState>,
    Path(persona_id): Path<i64>,
    Json(input): Json<CreateMemory>,
) -> Result<(StatusCode, Json<Memory>), StatusCode> {
    if persona_id <= 0 || !valid_input(&input) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let observed_at = normalize_observed_at(&input.observed_at)?;
    let row = sqlx_core::query::query(
        "INSERT INTO memories (persona_id, content, source, observed_at, confirmation_status)
         SELECT id, $2, $3, $4::timestamptz, $5 FROM personas WHERE id = $1
         RETURNING id, persona_id, content, source,
                   CASE WHEN observed_at >= '0001-01-01T00:00:00Z'::timestamptz
                             AND observed_at < '10000-01-01T00:00:00Z'::timestamptz
                        THEN to_char(observed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')
                   END AS observed_at,
                   confirmation_status",
    )
    .bind(persona_id)
    .bind(input.content)
    .bind(input.source)
    .bind(observed_at)
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
    Ok((StatusCode::CREATED, Json(map_row(row)?)))
}

/// Memories einer Persona nach ID geordnet abrufen.
#[utoipa::path(
    get, path = "/personas/{id}/memories", operation_id = "listMemories", tag = "Memories",
    params(("id" = i64, Path, minimum = 1, description = "Positive Persona-ID")),
    responses(
        (status = 200, description = "Nur Memories der angefragten Persona", body = [Memory]),
        (status = 400, description = "Ungültige ID (text/plain bei Pfadfehlern, sonst leer)", body = String, content_type = "text/plain"),
        (status = 404, description = "Persona nicht gefunden; leerer Body"),
        (status = 500, description = "Datenbankfehler; leerer Body")
    )
)]
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
                CASE WHEN observed_at >= '0001-01-01T00:00:00Z'::timestamptz
                             AND observed_at < '10000-01-01T00:00:00Z'::timestamptz
                        THEN to_char(observed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')
                   END AS observed_at,
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
    rows.into_iter()
        .map(map_row)
        .collect::<Result<Vec<_>, _>>()
        .map(Json)
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
    fn normalizes_rfc3339_offsets_and_truncates_submicroseconds() {
        for (input, expected) in [
            ("2026-10-30T16:00:00+01:00", "2026-10-30T15:00:00Z"),
            (
                "2026-10-30T15:00:00.123456789Z",
                "2026-10-30T15:00:00.123456Z",
            ),
            (
                "9999-12-31T23:59:59.999999999Z",
                "9999-12-31T23:59:59.999999Z",
            ),
        ] {
            assert_eq!(normalize_observed_at(input).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_non_rfc3339_and_out_of_api_range_timestamps() {
        for input in [
            "2026-10-30",
            "2026-10-30T15:00:00",
            "now",
            "tomorrow",
            "infinity",
            "-infinity",
            "2026-02-29T00:00:00Z",
            "0000-01-01T00:00:00Z",
            "9999-12-31T23:59:59-01:00",
        ] {
            assert_eq!(
                normalize_observed_at(input),
                Err(StatusCode::BAD_REQUEST),
                "{input}"
            );
        }
    }

    #[test]
    fn status_serializes_as_expected() {
        assert_eq!(
            serde_json::to_string(&ConfirmationStatus::Inferred).unwrap(),
            "\"inferred\""
        );
    }
}
