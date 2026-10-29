use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx_core::row::Row;
use sqlx_postgres::PgRow;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{memories::AppState, telemetry};

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreatePersona {
    /// Name wird außen getrimmt; danach 1 bis 200 Zeichen ohne Steuerzeichen.
    name: String,
}

#[derive(Serialize, ToSchema)]
pub struct Persona {
    #[schema(minimum = 1)]
    id: i64,
    name: String,
    #[schema(format = DateTime)]
    created_at: String,
}

fn persona(row: PgRow) -> Persona {
    Persona {
        id: row.get("id"),
        name: row.get("name"),
        created_at: row.get("created_at"),
    }
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create, find))
}

/// Persona anlegen.
///
/// Namen werden vor der Validierung außen getrimmt. Das vorhandene Schema
/// erzwingt eindeutige Namen; Namenskonflikte liefern 409 ohne Response-Body.
#[utoipa::path(
    post, path = "/personas", operation_id = "createPersona", tag = "Personas",
    request_body = CreatePersona,
    responses(
        (status = 201, description = "Persona angelegt", body = Persona,
            headers(("Location" = String, description = "Relativer Pfad zur neuen Persona"))),
        (status = 400, description = "Ungültiges JSON (text/plain) oder ungültiger Name (leerer Body)", body = String, content_type = "text/plain"),
        (status = 409, description = "Name bereits vorhanden; leerer Body"),
        (status = 413, description = "Request-Body zu groß", body = String, content_type = "text/plain"),
        (status = 415, description = "JSON-Content-Type fehlt oder ist nicht unterstützt", body = String, content_type = "text/plain"),
        (status = 422, description = "Fehlendes, unbekanntes oder falsch typisiertes JSON-Feld", body = String, content_type = "text/plain"),
        (status = 500, description = "Datenbankfehler; leerer Body")
    )
)]
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

/// Eine Persona gezielt per ID abrufen.
#[utoipa::path(
    get, path = "/personas/{id}", operation_id = "getPersona", tag = "Personas",
    params(("id" = i64, Path, minimum = 1, description = "Positive Persona-ID")),
    responses(
        (status = 200, description = "Angefragte Persona", body = Persona),
        (status = 400, description = "Ungültige ID (text/plain bei Pfadfehlern, sonst leer)", body = String, content_type = "text/plain"),
        (status = 404, description = "Persona nicht gefunden; leerer Body"),
        (status = 500, description = "Datenbankfehler; leerer Body")
    )
)]
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
