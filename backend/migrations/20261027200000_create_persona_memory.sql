-- Initial schema; persona-scoped records and provenance for future issues.
CREATE TABLE IF NOT EXISTS personas (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS memories (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    persona_id BIGINT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS memories_persona_id_idx ON memories(persona_id);
