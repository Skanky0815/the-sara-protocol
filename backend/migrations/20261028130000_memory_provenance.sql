ALTER TABLE memories
    ADD COLUMN IF NOT EXISTS observed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS confirmation_status TEXT NOT NULL DEFAULT 'inferred';
ALTER TABLE memories
    ADD CONSTRAINT memories_confirmation_status_check
    CHECK (confirmation_status IN ('confirmed', 'inferred'));
CREATE INDEX IF NOT EXISTS memories_persona_created_idx ON memories(persona_id, id);
