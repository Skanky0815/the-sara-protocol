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
}
