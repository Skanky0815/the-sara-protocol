use axum::{body::Body, http::Request, Router};
use serde_json::{json, Value};
use std::{
    io::{self, Write},
    process::{Command, Output},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use the_sara_protocol::telemetry;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
pub struct Logs(Arc<Mutex<Vec<u8>>>);

impl Logs {
    pub fn capture(filter: Option<&str>) -> (Self, tracing::subscriber::DefaultGuard) {
        let logs = Self::default();
        let guard = tracing::subscriber::set_default(telemetry::subscriber(
            telemetry::log_filter(filter).unwrap(),
            logs.clone(),
        ));
        (logs, guard)
    }

    pub fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }

    pub fn events(&self) -> Vec<Value> {
        parse_events(&self.text())
    }
}

impl Write for Logs {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Logs {
    type Writer = Self;
    fn make_writer(&'a self) -> Self {
        self.clone()
    }
}

pub fn parse_events(text: &str) -> Vec<Value> {
    text.lines()
        .map(|line| serde_json::from_str(line).expect("JSON log event"))
        .collect()
}

pub fn health_request() -> Request<Body> {
    Request::builder()
        .uri("/health?token=query-secret")
        .header("authorization", "Bearer auth-secret")
        .header("cookie", "session=cookie-secret")
        .body(Body::from("health-body-secret"))
        .unwrap()
}

pub fn memory_request(id: i64, timestamp: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/personas/{id}/memories?secret=query-secret"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"content":"memory-body-secret", "source":"source-secret",
            "observed_at":timestamp,"confirmation_status":"confirmed"})
            .to_string(),
        ))
        .unwrap()
}

pub async fn closed_pool_router() -> Router {
    let pool = sqlx_postgres::PgPool::connect_lazy(
        "postgres://db-user-secret:db-pass-secret@localhost/test",
    )
    .unwrap();
    pool.close().await;
    the_sara_protocol::app_with_pool(pool)
}

pub fn unique_name() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "logging-test-{}-{}-secret",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

struct WorkingDirectory(std::path::PathBuf);
impl Drop for WorkingDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir(&self.0).expect("remove isolated test directory");
    }
}

pub fn startup(env: &[(&str, &str)]) -> Output {
    let dir = WorkingDirectory(std::env::temp_dir().join(unique_name()));
    std::fs::create_dir(&dir.0).unwrap();
    Command::new(env!("CARGO_BIN_EXE_the-sara-protocol"))
        .current_dir(&dir.0)
        .env_clear()
        .envs(env.iter().copied())
        .output()
        .unwrap()
}
