use std::error::Error;
use the_sara_protocol::config::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Existing environment variables take precedence over values in .env.
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    let pool = sqlx_postgres::PgPool::connect(&config.database_url).await?;
    let migrations =
        sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations")).await?;
    migrations.run(&pool).await?;

    let address = (config.host, config.port);
    let listener = TcpListener::bind(address).await?;
    println!("Listening on http://{}:{}", config.host, config.port);
    axum::serve(listener, the_sara_protocol::app()).await?;
    Ok(())
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
        migrations.run(&pool).await.expect("initial migration must apply");
        // A second run verifies that the migration is idempotent at the runner level.
        migrations.run(&pool).await.expect("migration rerun must succeed");
        sqlx_core::query::query("SELECT id FROM personas LIMIT 0")
            .execute(&pool)
            .await
            .expect("personas table must exist");
        sqlx_core::query::query("SELECT id FROM memories LIMIT 0")
            .execute(&pool)
            .await
            .expect("memories table must exist");
    }

}
