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
Run the service from the repository root: migrations are loaded from `./migrations`
at runtime. When distributing the binary, include that directory in its working
directory. Missing or unreadable migrations prevent startup.

The service uses sqlx-core and sqlx-postgres directly, without SQLx macros,
to keep optional MySQL dependencies (including `rsa`,
affected by RUSTSEC-2023-0071) out of the lockfile. See
[ADR 0004](adr/0004-runtime-migrations.md).

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

## Persona memories (Issue #11)

Use `dev/persona.http` in the IntelliJ HTTP Client. Set `@personaId` to the ID of an existing persona in your local database. Persona creation is tracked separately; this issue only adds memory endpoints.

- `POST /personas/{id}/memories` expects JSON with `content`, `source`, `observed_at` (ISO-8601 timestamp with timezone), and `confirmation_status` (`confirmed` or `inferred`).
- `GET /personas/{id}/memories` lists only memories owned by that persona, ordered by ID.
- Unknown persona: HTTP 404. Empty content/source, invalid timestamps and nonpositive persona IDs: HTTP 400. An unsupported `confirmation_status` is rejected by JSON deserialization with HTTP 422. Successful creation: HTTP 201.
- The initial schema's `created_at` remains the database insertion time; `observed_at` records the source event time.

For persistence and persona-isolation integration tests, start PostgreSQL and run `DATABASE_URL=postgres://sara:sara_local_dev@127.0.0.1:5432/sara cargo test --test memory_api`. The test creates and removes temporary persona rows. Without `DATABASE_URL`, the integration test skips itself.

To run the complete CI test suite locally, including migration and memory API
checks, export the test database URL for all targets:

```sh
DATABASE_URL=postgres://sara:sara_local_dev@127.0.0.1:5432/sara cargo test --all-targets --all-features
cargo audit
```

Use a dedicated test database. `cargo audit` requires the separately installed
`cargo-audit` tool and access to the current RustSec advisory database.

The memory API suite uses separate named tests for persistence, persona isolation
and rejection cases. Each test creates its own persona so the tests can run in
parallel. To run just one case, pass its name, for example
`cargo test --test memory_api unknown_persona_returns_not_found` with `DATABASE_URL`
set as above.
