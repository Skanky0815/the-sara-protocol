# Memories durchsuchen

`GET /personas/{id}/memories/search?q=Katze%20Kaffee&limit=20` liefert ein JSON-Array vollständiger Memories der angegebenen Persona. Beispiel:

```sh
curl --get 'http://127.0.0.1:3000/personas/1/memories/search' \
  --data-urlencode 'q=Katze Kaffee' --data-urlencode 'limit=20'
```

Nur `content` wird durchsucht. PostgreSQL normalisiert deutsche Wortformen und Groß-/Kleinschreibung: `Katze` findet auch `Katzen`. Alle relevanten Suchbegriffe müssen vorkommen; Satzzeichen sind keine Suchoperatoren. Quellen, Persona-Namen und andere Provenienzfelder werden nicht durchsucht, aber unverändert mit den Treffern zurückgegeben. Die Relevanz (`ts_rank_cd`) bestimmt die Reihenfolge, bei Gleichstand die ID aufsteigend.

`q` ist erforderlich, nicht nur Whitespace, maximal 500 UTF-8-Bytes, ohne NUL. `limit` ist optional (Standard 20), ganzzahlig zwischen 1 und 100. Fehlerhafte, doppelte oder unbekannte Query-Parameter ergeben HTTP 400. Eine existierende Persona ohne Treffer, eine reine Stoppwortsuche (z. B. `der und die`) oder eine Suche ohne indexierbare Wörter liefern HTTP 200 mit `[]`. Unbekannte Persona: 404; nichtpositive/ungültige ID: 400; Datenbankfehler: 500. Fachliche Fehler haben einen leeren Body, Extraktionsfehler einen `text/plain`-Body. Der generierte Vertrag steht unter `/openapi.json`.

Das bestehende `GET /personas/{id}/memories` bleibt das vollständige Listing nach ID. Die Suche begrenzt ihre Ergebnismenge und bietet vorerst keine weitere Pagination. Die Datenbereichstrennung ersetzt keine Benutzer-Authentifizierung. Suchtext und Ergebnisse werden nicht durch das Request-Logging ausgegeben.

Die additive Migration erstellt einen GIN-Ausdrucksindex mit festem `pg_catalog.german`. Bestehende Erinnerungen sind ohne erneutes Speichern suchbar; Inhaltsänderungen aktualisieren den Index automatisch. Keine neue PostgreSQL-Erweiterung oder Rust-Abhängigkeit ist erforderlich. Der reguläre Indexaufbau kann während der Migration Schreibzugriffe blockieren; große Deployments benötigen eine gesonderte Online-Migrationsplanung.

Semantische Suche mit pgvector ist als separater Folgeschritt in [ADR 0010](adr/0010-persona-memory-full-text-search.md) bewertet. Sie ist noch nicht implementiert.

Prüfen aus `backend/` mit dedizierter Testdatenbank und gesetzter `DATABASE_URL`:

```sh
cargo test --locked --test memory_api full_text
cargo test --locked --test openapi_contract
```
