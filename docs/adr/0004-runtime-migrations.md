# ADR 0004: Runtime migration loading without SQLx macros

- Date: 2026-10-27
- Status: Proposed
- Documented by: Codex (AI assistant)

## Context
The SQLx 0.8 facade introduces optional database dependencies into Cargo.lock,
including sqlx-mysql and rsa. The active PostgreSQL build does not use them,
but cargo audit reports rsa under RUSTSEC-2023-0071, which has no patched version.

## Decision
Depend directly on sqlx-core and sqlx-postgres without macros and load
`./migrations` with the runtime Migrator API. Keep PostgreSQL, Tokio, rustls
with ring and WebPKI roots, and migration support enabled. Pin both components
to 0.8.6 because the runtime/TLS feature flags are internal SQLx interfaces;
review those flags together when upgrading. Do not suppress the advisory.

## Alternatives
Ignore the advisory for the unused driver. Disabling macros on the facade
was also tested, but does not remove the optional drivers from the lockfile.

## Consequences
The lockfile excludes the unused drivers and rsa. Migration files are no longer
embedded in the binary: deployments must ship the migrations directory and
start the process from its parent. Loading or applying migrations must succeed
before the service binds its HTTP port. A test checks loading the initial schema.
