# Local development (Issue #8)

Requires a recent stable Rust toolchain (`cargo`, `rustc`).

```sh
cargo run
```

In another terminal:

```sh
curl -i http://127.0.0.1:3000/health
```

Expected HTTP status: `200 OK`, JSON: `{"status":"ok"}`.

Run tests:

```sh
cargo test
```

This first service has no database dependency; PostgreSQL and SQLx follow in Issue #9.
