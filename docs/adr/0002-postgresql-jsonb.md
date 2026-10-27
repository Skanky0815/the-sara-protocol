# ADR 0002: PostgreSQL with JSONB

- Date: 2026-10-27 (retrospective record)
- Status: Accepted
- Decision participants: Rico and Sara (AI assistant)
- Documented by: Sara (AI assistant)

## Context
Persona and memory records need flexible document-shaped fields alongside relationships, transactions and provenance.

## Decision
Use PostgreSQL with JSONB for the MVP, leaving pgvector as an optional later addition. No separate search service initially.

## Alternatives
MongoDB and CouchDB were discussed.

## Consequences
One database can handle relational and flexible data. Schema and migrations still need deliberate design; vector search is not yet implemented.
