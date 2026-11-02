use rmcp::{
    model::{CallToolRequestParams, CallToolResult},
    service::RunningService,
    transport::TokioChildProcess,
    RoleClient, ServiceExt,
};
use serde_json::{json, Value};
use sqlx_postgres::PgPool;
use std::{
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use the_sara_protocol::mcp::{parse_persona_id, LocalMcp};
use tokio::io::AsyncReadExt;

async fn connect(server: LocalMcp) -> RunningService<RoleClient, ()> {
    let (server_io, client_io) = tokio::io::duplex(65536);
    tokio::spawn(async move {
        let _ = server.serve(server_io).await.unwrap().waiting().await;
    });
    tokio::time::timeout(Duration::from_secs(10), ().serve(client_io))
        .await
        .unwrap()
        .unwrap()
}

async fn call(client: &RunningService<RoleClient, ()>, name: &str, args: Value) -> CallToolResult {
    client
        .call_tool(
            CallToolRequestParams::new(name.to_owned())
                .with_arguments(args.as_object().unwrap().clone()),
        )
        .await
        .unwrap()
}

async fn closed_pool() -> PgPool {
    let pool =
        PgPool::connect_lazy("postgres://private-secret:password-secret@localhost/test").unwrap();
    pool.close().await;
    pool
}

#[test]
fn mcp_persona_configuration_is_required_positive_and_never_echoed() {
    for value in [
        None,
        Some(""),
        Some("0"),
        Some("-1"),
        Some("9223372036854775808"),
        Some("private-secret"),
    ] {
        let error = parse_persona_id(value).unwrap_err();
        assert_eq!(error, "MCP_PERSONA_ID must be a positive 64-bit integer");
    }
    assert_eq!(parse_persona_id(Some("42")).unwrap(), 42);
}

#[tokio::test]
async fn real_mcp_discovery_advertises_only_two_read_only_persona_bound_tools() {
    let pool = closed_pool().await;
    assert!(LocalMcp::new(pool.clone(), 0).is_err());
    assert!(LocalMcp::new(pool.clone(), -1).is_err());
    let client = connect(LocalMcp::new(pool, 42).unwrap()).await;
    let tools = client.list_all_tools().await.unwrap();
    assert_eq!(
        tools.iter().map(|t| t.name.as_ref()).collect::<Vec<_>>(),
        ["get_persona", "search_memories"]
    );
    for tool in &tools {
        assert_eq!(
            tool.annotations.as_ref().unwrap().read_only_hint,
            Some(true)
        );
        assert_eq!(
            tool.annotations.as_ref().unwrap().open_world_hint,
            Some(false)
        );
        assert_eq!(tool.input_schema["additionalProperties"], false);
        assert!(tool.input_schema["properties"].get("persona_id").is_none());
        jsonschema::validator_for(&Value::Object((*tool.input_schema).clone())).unwrap();
    }
    let info = client.peer_info().unwrap();
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.resources.is_none());
    assert!(info.capabilities.prompts.is_none());
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn mcp_rejects_persona_overrides_invalid_search_and_write_tools() {
    let client = connect(LocalMcp::new(closed_pool().await, 42).unwrap()).await;
    for (name, args) in [
        ("get_persona", json!({"persona_id":43})),
        ("search_memories", json!({"q":"Katze", "persona_id":43})),
        ("search_memories", json!({})),
        ("search_memories", json!({"q":" "})),
        ("search_memories", json!({"q":"\u{0}"})),
        ("search_memories", json!({"q":"ä".repeat(251)})),
        ("search_memories", json!({"q":"Katze", "limit":0})),
        ("search_memories", json!({"q":"Katze", "limit":21})),
        ("search_memories", json!({"q":"Katze", "limit":"1"})),
    ] {
        let result = call(&client, name, args).await;
        assert_eq!(result.is_error, Some(true));
        assert!(result.structured_content.is_none());
    }
    for name in ["create_memory", "list_personas", "anything"] {
        assert!(client
            .call_tool(CallToolRequestParams::new(name))
            .await
            .is_err());
    }
    // Valid calls reach the closed backend, whose private diagnostics are hidden.
    for (name, args) in [
        ("get_persona", json!({})),
        ("search_memories", json!({"q":"private-query-secret"})),
    ] {
        let result = call(&client, name, args).await;
        assert_eq!(result.is_error, Some(true));
        let text = serde_json::to_string(&result).unwrap();
        assert!(!text.contains("secret"));
        assert!(!text.contains("postgres://"));
        assert!(!text.contains("SELECT"));
    }
    client.cancel().await.unwrap();
}

struct Fixture {
    pool: PgPool,
    url: String,
    first: i64,
    second: i64,
}
impl Fixture {
    async fn new() -> Option<Self> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("Skipping MCP database test: DATABASE_URL is unset");
            return None;
        };
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations"))
            .await
            .unwrap()
            .run(&pool)
            .await
            .unwrap();
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let mut ids = Vec::new();
        for _ in 0..2 {
            let name = format!(
                "mcp_synthetic_{}_{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            );
            let (id,): (i64,) =
                sqlx_core::query_as::query_as("INSERT INTO personas(name) VALUES($1) RETURNING id")
                    .bind(name)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            ids.push(id);
        }
        Some(Self {
            pool,
            url,
            first: ids[0],
            second: ids[1],
        })
    }
    async fn memory(&self, persona: i64, content: &str) {
        sqlx_core::query::query("INSERT INTO memories(persona_id, content, source, observed_at, confirmation_status) VALUES ($1, $2, 'synthetic MCP test', '2026-11-02T08:00:00Z', 'inferred')")
            .bind(persona).bind(content).execute(&self.pool).await.unwrap();
    }
    async fn cleanup(&self) {
        sqlx_core::query::query("DELETE FROM personas WHERE id = $1 OR id = $2")
            .bind(self.first)
            .bind(self.second)
            .execute(&self.pool)
            .await
            .unwrap();
    }
    async fn subprocess(&self) -> (RunningService<RoleClient, ()>, tokio::process::ChildStderr) {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_the-sara-protocol"));
        command
            .arg("--mcp")
            .env_clear()
            .env("DATABASE_URL", &self.url)
            .env("MCP_PERSONA_ID", self.first.to_string())
            .env(
                "MIGRATIONS_DIR",
                std::fs::canonicalize("migrations").unwrap(),
            )
            .env("RUST_LOG", "debug");
        let (transport, stderr) = TokioChildProcess::builder(command)
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let client = tokio::time::timeout(Duration::from_secs(15), ().serve(transport))
            .await
            .unwrap()
            .unwrap();
        (client, stderr.unwrap())
    }
}

#[tokio::test]
async fn stdio_process_reads_scoped_memories_and_survives_process_restart() {
    let Some(test) = Fixture::new().await else {
        return;
    };
    test.memory(test.first, "Katzen trinken Kaffee. private-content-secret")
        .await;
    test.memory(test.second, "Katzen trinken Kaffee. foreign-persona-secret")
        .await;
    for _ in 0..2 {
        let (client, mut stderr) = test.subprocess().await;
        let persona = call(&client, "get_persona", json!({})).await;
        assert_eq!(persona.is_error, Some(false));
        assert_eq!(
            persona.structured_content.unwrap()["persona"]["id"],
            test.first
        );
        let result = call(
            &client,
            "search_memories",
            json!({"q":"KATZE Kaffee", "limit":1}),
        )
        .await;
        assert_eq!(result.is_error, Some(false));
        let data = result.structured_content.unwrap();
        let memories = data["memories"].as_array().unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0]["persona_id"], test.first);
        assert_eq!(memories[0]["source"], "synthetic MCP test");
        assert_eq!(memories[0]["confirmation_status"], "inferred");
        assert_eq!(memories[0]["observed_at"], "2026-11-02T08:00:00.000000Z");
        assert!(!data.to_string().contains("foreign-persona-secret"));
        let none = call(&client, "search_memories", json!({"q":"unauffindbar"})).await;
        assert_eq!(none.structured_content.unwrap()["memories"], json!([]));
        client.cancel().await.unwrap();
        let mut logs = String::new();
        tokio::time::timeout(Duration::from_secs(10), stderr.read_to_string(&mut logs))
            .await
            .unwrap()
            .unwrap();
        assert!(logs.contains("mcp_stdio"));
        assert!(!logs.contains("http_listen"));
        for secret in [
            "private-content-secret",
            "foreign-persona-secret",
            "KATZE",
            &test.url,
        ] {
            assert!(!logs.contains(secret), "private data in stderr");
        }
        for line in logs.lines() {
            serde_json::from_str::<Value>(line).expect("stderr is JSON logging");
        }
    }
    // Persona deletion invalidates the next call, rather than switching identity.
    let client = connect(LocalMcp::new(test.pool.clone(), test.first).unwrap()).await;
    test.cleanup().await;
    let result = call(&client, "get_persona", json!({})).await;
    assert_eq!(result.is_error, Some(true));
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn oversized_mcp_results_are_rejected_without_truncating_provenance() {
    let Some(test) = Fixture::new().await else {
        return;
    };
    for _ in 0..15 {
        test.memory(test.first, &format!("Kaffee {}", "x ".repeat(45000)))
            .await;
    }
    let client = connect(LocalMcp::new(test.pool.clone(), test.first).unwrap()).await;
    let result = call(
        &client,
        "search_memories",
        json!({"q":"Kaffee", "limit":20}),
    )
    .await;
    assert_eq!(result.is_error, Some(true));
    let result = call(&client, "search_memories", json!({"q":"Kaffee", "limit":1})).await;
    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content.unwrap()["memories"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    client.cancel().await.unwrap();
    test.cleanup().await;
}

#[test]
fn stdio_startup_rejects_missing_or_invalid_persona_without_stdout_or_secrets() {
    let directory = std::env::temp_dir().join(format!("sara-mcp-config-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    // HTTP credentials must never select a persona for MCP implicitly.
    std::fs::write(
        directory.join(".env"),
        "MCP_PERSONA_ID=42\nDATABASE_URL=postgres://http-config-secret/test\n",
    )
    .unwrap();
    for id in [None, Some("invalid-persona-secret"), Some("0")] {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_the-sara-protocol"));
        command.arg("--mcp").env_clear().current_dir(&directory);
        if let Some(id) = id {
            command.env("MCP_PERSONA_ID", id);
        }
        let output = command.output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("invalid_mcp_persona"));
        assert!(!stderr.contains("invalid-persona-secret"));
        assert!(!stderr.contains("http-config-secret"));
    }
    std::fs::remove_file(directory.join(".env")).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[tokio::test]
async fn stdio_startup_fails_closed_for_a_deleted_persona() {
    let Some(test) = Fixture::new().await else {
        return;
    };
    test.cleanup().await;
    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_the-sara-protocol"))
        .arg("--mcp")
        .env_clear()
        .env("DATABASE_URL", &test.url)
        .env("MCP_PERSONA_ID", test.first.to_string())
        .env(
            "MIGRATIONS_DIR",
            std::fs::canonicalize("migrations").unwrap(),
        )
        .output()
        .await
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let logs = String::from_utf8(output.stderr).unwrap();
    assert!(logs.contains("persona_unavailable"));
    assert!(!logs.contains(&test.url));
}
