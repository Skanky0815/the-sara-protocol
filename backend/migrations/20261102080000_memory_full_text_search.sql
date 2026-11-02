-- Fixed configuration matches the search query, independent of the DB default.
-- An expression index covers existing memories and subsequent inserts/updates.
CREATE INDEX memories_content_search_idx
    ON memories USING GIN (to_tsvector('pg_catalog.german', content));
