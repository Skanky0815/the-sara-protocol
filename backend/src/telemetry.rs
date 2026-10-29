use axum::{body::Body, extract::MatchedPath, http::Request, Router};
use sqlx_core::connection::ConnectOptions;
use sqlx_postgres::{PgConnectOptions, PgPool};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tower_http::trace::TraceLayer;
use tracing::{Span, Subscriber};
use tracing_subscriber::{
    filter::filter_fn, fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Layer,
};

const DEFAULT_FILTER: &str = "the_sara_protocol=info";

pub fn log_filter(value: Option<&str>) -> Result<EnvFilter, &'static str> {
    EnvFilter::builder()
        .with_regex(false)
        .parse(
            value
                .filter(|v| !v.trim().is_empty())
                .unwrap_or(DEFAULT_FILTER),
        )
        .map_err(|_| "invalid log filter")
}

pub fn subscriber<W>(filter: EnvFilter, writer: W) -> impl Subscriber + Send + Sync
where
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Even RUST_LOG=debug must not enable dependency payload/SQL diagnostics.
    // Only application events with explicitly selected fields are emitted.
    tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer)
            .with_filter(filter_fn(|metadata| {
                metadata.target() == "the_sara_protocol"
                    || metadata.target().starts_with("the_sara_protocol::")
            }))
            .with_filter(filter),
    )
}

pub fn init() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    let requested = std::env::var("RUST_LOG").ok();
    let parsed = log_filter(requested.as_deref());
    let invalid = parsed.is_err();
    let filter = parsed.unwrap_or_else(|_| log_filter(None).expect("valid default filter"));
    tracing::subscriber::set_global_default(subscriber(filter, std::io::stderr))?;
    if invalid {
        tracing::warn!(
            stage = "logging",
            error_kind = "invalid_filter",
            "invalid RUST_LOG; using default filter"
        );
    }
    Ok(())
}

#[derive(Debug)]
pub struct DatabaseConnectionError;

pub async fn connect_database(url: &str) -> Result<PgPool, DatabaseConnectionError> {
    let options = url
        .parse::<PgConnectOptions>()
        .map_err(|_| {
            tracing::error!(
                stage = "database_config",
                error_kind = "invalid_database_url",
                "database configuration rejected"
            );
            DatabaseConnectionError
        })?
        .disable_statement_logging();
    let pool = PgPool::connect_with(options).await.map_err(|error| {
        database_error("connect", &error);
        DatabaseConnectionError
    })?;
    tracing::info!(stage = "database", "database connected");
    Ok(pool)
}

pub fn database_error(operation: &'static str, error: &sqlx_core::Error) {
    let kind = match error {
        sqlx_core::Error::PoolTimedOut => "pool_timeout",
        sqlx_core::Error::PoolClosed => "pool_closed",
        sqlx_core::Error::Io(_) => "connection_error",
        sqlx_core::Error::Database(db)
            if matches!(db.code().as_deref(), Some("22007" | "22008")) =>
        {
            "invalid_timestamp"
        }
        sqlx_core::Error::Database(_) => "database_error",
        _ => "query_error",
    };
    if kind == "invalid_timestamp" {
        tracing::warn!(operation, error_kind = kind, "database rejected timestamp");
    } else {
        tracing::error!(operation, error_kind = kind, "database operation failed");
    }
}

pub fn trace_requests(router: Router) -> Router {
    static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let route = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str)
                    .unwrap_or("<unmatched>");
                tracing::info_span!("http.request",
                    request_id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed),
                    method = %request.method(), route)
            })
            .on_request(())
            .on_response(
                |response: &axum::http::Response<Body>, duration: Duration, _span: &Span| {
                    let status = response.status().as_u16();
                    let duration_ms = duration.as_secs_f64() * 1_000.0;
                    if response.status().is_server_error() {
                        tracing::error!(status, duration_ms, "request completed");
                    } else if response.status().is_client_error() {
                        tracing::warn!(status, duration_ms, "request completed");
                    } else {
                        tracing::info!(status, duration_ms, "request completed");
                    }
                },
            )
            .on_failure(()),
    )
}
