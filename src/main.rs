use std::error::Error;
use the_sara_protocol::config::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Existing environment variables take precedence over values in .env.
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let address = (config.host, config.port);
    let listener = TcpListener::bind(address).await?;
    println!("Listening on http://{}:{}", config.host, config.port);
    axum::serve(listener, the_sara_protocol::app()).await?;
    Ok(())
}
