# Local development (Issues #8 and #9)

Requires a recent stable Rust toolchain (`cargo`, `rustc`) and Docker Compose.

Start PostgreSQL:

```sh
docker compose up -d postgres
```

Copy the example configuration:

```sh
cp .env.example .env
cargo run
```

The service loads `.env` for local development; exported environment variables take precedence.
`HOST` defaults to `127.0.0.1` and `PORT` defaults to `3000`.
`DATABASE_URL` is required. Invalid values cause startup to fail with an error.
Database connectivity and SQLx migrations are checked at startup, before binding the HTTP port.

In another terminal:

```sh
curl -i http://127.0.0.1:3000/health
```

Expected HTTP status: `200 OK`, JSON: `{"status":"ok"}`.

Run unit tests:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

For an isolated migration smoke test, start with an empty Compose volume (`docker compose down -v` **deletes local database data**), then run `cargo run` and verify the process starts. Re-running `cargo run` should leave the applied migration intact.

Do not commit `.env` or real database credentials. In CI and deployment, inject configuration as environment variables rather than copying a local `.env` file.
