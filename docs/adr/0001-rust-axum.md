# ADR 0001: Rust web service with Axum

- Date: 2026-10-27 (retrospective record)
- Status: Accepted
- Decision participants: Rico and Sara (AI assistant)
- Documented by: Sara (AI assistant)

## Context
The Sara Protocol needs a small, performant local-first REST service for personas and memories, initially developed on macOS.

## Decision
Use Rust with Axum, Tokio, Serde and SQLx for the initial service.

## Alternatives
Kotlin/Spring Boot and TypeScript/Fastify were considered.

## Consequences
Strong type safety and low runtime overhead, with added Rust learning/build complexity. LLM latency and retrieval quality may dominate end-to-end performance.
