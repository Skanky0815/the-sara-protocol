use axum::{
    body::{to_bytes, Body},
    http::Request,
    response::Response,
    Router,
};
use serde_json::{json, Value};
use sqlx_postgres::PgPool;
use tower::ServiceExt;

pub struct Contract {
    pub router: Router,
    pub doc: Value,
}

impl Contract {
    pub async fn new(router: Router) -> Self {
        let response = send(&router, "GET", "/openapi.json", None, None).await;
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["content-type"], "application/json");
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        Self {
            router,
            doc: serde_json::from_slice(&bytes).unwrap(),
        }
    }

    pub async fn offline() -> Self {
        let pool = PgPool::connect_lazy("postgres://local:secret@localhost/test").unwrap();
        pool.close().await;
        Self::new(the_sara_protocol::app_with_pool(pool)).await
    }

    pub fn validate(&self, schema: &Value, value: &Value) {
        let root = json!({"$schema":"https://json-schema.org/draft/2020-12/schema",
            "components": self.doc["components"], "allOf":[schema]});
        let validator = jsonschema::draft202012::options()
            .should_validate_formats(true)
            .build(&root)
            .unwrap();
        validator.validate(value).unwrap();
    }

    pub async fn response(&self, path: &str, method: &str, response: Response) -> Option<Value> {
        let status = response.status();
        let described = &self.doc["paths"][path][method]["responses"][status.as_str()];
        assert!(
            described.is_object(),
            "undocumented response: {method} {path} {status}"
        );
        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned());
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        if described.get("content").is_none() {
            assert!(bytes.is_empty(), "response documented without body");
            return None;
        }
        if bytes.is_empty() {
            assert_eq!(
                status.as_u16(),
                400,
                "only mixed validation responses may be empty"
            );
            return None;
        }
        let content_type = content_type.expect("response content type");
        let schema = &described["content"][&content_type]["schema"];
        assert!(
            schema.is_object(),
            "undocumented media type: {content_type}"
        );
        let value = if content_type == "application/json" {
            serde_json::from_slice(&bytes).unwrap()
        } else {
            Value::String(String::from_utf8(bytes.to_vec()).unwrap())
        };
        self.validate(schema, &value);
        Some(value)
    }
}

pub async fn send(
    router: &Router,
    method: &str,
    path: &str,
    body: Option<String>,
    content_type: Option<&str>,
) -> Response {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(kind) = content_type {
        request = request.header("content-type", kind);
    }
    router
        .clone()
        .oneshot(
            request
                .body(body.map_or_else(Body::empty, Body::from))
                .unwrap(),
        )
        .await
        .unwrap()
}

pub async fn database() -> Option<PgPool> {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("Skipping OpenAPI database test: DATABASE_URL is unset");
        return None;
    };
    let pool = PgPool::connect(&url).await.unwrap();
    sqlx_core::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();
    Some(pool)
}

pub struct PersonaCase {
    pub pool: PgPool,
    pub contract: Contract,
    pub persona: Value,
    pub location: String,
}

impl PersonaCase {
    pub async fn new() -> Option<Self> {
        let pool = database().await?;
        let contract = Contract::new(the_sara_protocol::app_with_pool(pool.clone())).await;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let name = format!(
            "openapi_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        let response = send(
            &contract.router,
            "POST",
            "/personas",
            Some(json!({"name":name}).to_string()),
            Some("application/json"),
        )
        .await;
        assert_eq!(response.status(), 201);
        let location = response.headers()["location"].to_str().unwrap().to_owned();
        let persona = contract
            .response("/personas", "post", response)
            .await
            .unwrap();
        Some(Self {
            pool,
            contract,
            persona,
            location,
        })
    }

    pub async fn remove(&self) {
        sqlx_core::query::query("DELETE FROM personas WHERE id = $1")
            .bind(self.persona["id"].as_i64().unwrap())
            .execute(&self.pool)
            .await
            .unwrap();
    }
}
