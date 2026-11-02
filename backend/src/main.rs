use std::process::ExitCode;
use the_sara_protocol::{config::Config, mcp, telemetry};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    // The local pilot has separate credentials; never fall back to the HTTP .env.
    // Existing environment variables retain precedence in either mode.
    if std::env::args_os().nth(1).is_some_and(|arg| arg == "--mcp") {
        dotenvy::from_path(".env.mcp").ok();
    } else {
        dotenvy::dotenv().ok();
    }
    if telemetry::init().is_err() {
        // The subscriber is unavailable; do not print potentially sensitive errors.
        return ExitCode::FAILURE;
    }
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}

async fn run() -> Result<(), ()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mcp_mode = args.len() == 1 && args[0] == "--mcp";
    if !args.is_empty() && !mcp_mode {
        tracing::error!(
            stage = "config",
            error_kind = "invalid_arguments",
            "use no arguments for HTTP or --mcp for local MCP"
        );
        return Err(());
    }
    let persona_id = if mcp_mode {
        Some(mcp::parse_persona_id(std::env::var("MCP_PERSONA_ID").ok().as_deref()).map_err(|reason| {
            tracing::error!(stage = "config", error_kind = "invalid_mcp_persona", %reason, "MCP configuration rejected");
        })?)
    } else {
        None
    };
    tracing::info!(stage = "startup", "starting backend");
    let config = Config::from_env().map_err(|reason| {
        // Config returns fixed validation messages, never supplied values.
        tracing::error!(stage = "config", error_kind = "invalid_config", %reason, "configuration rejected");
    })?;
    let pool = telemetry::connect_database(&config.database_url)
        .await
        .map_err(|_| ())?;
    let migrations = sqlx_core::migrate::Migrator::new(config.migrations_dir.as_path())
        .await
        .map_err(|_| {
            tracing::error!(
                stage = "migration_load",
                error_kind = "migration_load_failed",
                "cannot load migrations"
            );
        })?;
    migrations.run(&pool).await.map_err(|_| {
        tracing::error!(
            stage = "migration_apply",
            error_kind = "migration_failed",
            "cannot apply migrations"
        );
    })?;
    tracing::info!(stage = "migrations", "migrations applied");

    if let Some(persona_id) = persona_id {
        return mcp::serve_stdio(pool, persona_id).await.map_err(|_| ());
    }

    let listener = TcpListener::bind((config.host, config.port))
        .await
        .map_err(|_| {
            tracing::error!(
                stage = "http_bind",
                error_kind = "bind_failed",
                "cannot bind HTTP listener"
            );
        })?;
    tracing::info!(stage = "http_listen", host = %config.host, port = config.port, "backend ready");
    axum::serve(listener, the_sara_protocol::app_with_pool(pool))
        .await
        .map_err(|_| {
            tracing::error!(
                stage = "http_serve",
                error_kind = "serve_failed",
                "HTTP server stopped with an error"
            );
        })
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn migration_directory_loads_initial_schema() {
        let migrations = sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations"))
            .await
            .expect("migration files must be available at runtime");
        assert!(migrations
            .iter()
            .any(|migration| migration.version == 20261027200000));
        assert!(migrations
            .iter()
            .any(|migration| migration.version == 20261028130000));
    }

    // CI supplies DATABASE_URL and a fresh PostgreSQL service. Local unit tests
    // can still run without Docker; the integration check is skipped in that case.
    #[tokio::test]
    async fn postgres_applies_migrations_and_creates_tables() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("Skipping PostgreSQL integration test: DATABASE_URL is unset");
            return;
        };
        let pool = sqlx_postgres::PgPool::connect(&url)
            .await
            .expect("PostgreSQL must be reachable");
        let migrations = sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations"))
            .await
            .expect("migration files must load");
        migrations.run(&pool).await.expect("migrations must apply");
        migrations
            .run(&pool)
            .await
            .expect("migration rerun must succeed");
        sqlx_core::query::query("SELECT id FROM personas LIMIT 0")
            .execute(&pool)
            .await
            .expect("personas table must exist");
        sqlx_core::query::query(
            "SELECT id, observed_at, confirmation_status FROM memories LIMIT 0",
        )
        .execute(&pool)
        .await
        .expect("memory provenance columns must exist");
    }
}
