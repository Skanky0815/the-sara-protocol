# ADR 0010: Persona-spezifische PostgreSQL-Volltextsuche

- Datum: 2026-11-02
- Status: Vorgeschlagen
- An der Entscheidung beteiligt: Rico (Auftrag), Sara (Umsetzung)
- Dokumentiert von: Sara (KI-Assistentin)

## Kontext

[Issue #13](https://github.com/Skanky0815/the-sara-protocol/issues/13) benötigt gezielten Abruf relevanter Erinnerungen bei erhaltener Persona-Isolation und Provenienz. Der Dienst verwendet bereits PostgreSQL 17; ein zusätzlicher Suchdienst ist nicht vorgesehen.

## Entscheidung

`GET /personas/{id}/memories/search?q=...&limit=20` durchsucht ausschließlich `memories.content` der angefragten Persona. Das bestehende Listing bleibt unverändert. `plainto_tsquery('pg_catalog.german', q)` behandelt die Eingabe als Text und verbindet relevante Begriffe mit UND. Eine feste deutsche Konfiguration passt zur Projektsprache und verhindert Abhängigkeit von Datenbank-Defaults. Ein GIN-Ausdrucksindex auf `to_tsvector('pg_catalog.german', content)` erfasst vorhandene und neue Datensätze sowie spätere Inhaltsänderungen ohne zusätzliche Spalte oder Trigger.

Parameter werden gebunden. Ergebnisse enthalten das unveränderte Memory-DTO einschließlich Provenienz und sind nach `ts_rank_cd` absteigend, dann ID aufsteigend geordnet. Standardlimit 20, maximal 100. Suchtext muss 1–500 UTF-8-Bytes umfassen und darf weder nur Whitespace noch NUL enthalten. Unbekannte Parameter werden abgelehnt. Stoppwörter oder nicht indexierbare Zeichen allein liefern ein leeres Array. Die Persona-Existenzprüfung unterscheidet 404 von einer erfolgreichen leeren Suche.

## Alternativen

`ILIKE` liefert Teilstringtreffer, aber keine sprachliche Normalisierung oder Volltextrelevanz. Ein gespeicherter Suchvektor wäre bei umfangreicheren Suchdokumenten erwägenswert, benötigt hier jedoch zusätzlichen Speicher und Schemafelder. Ein externer Suchdienst vergrößert Betrieb und Synchronisationsaufwand. Eine mehrsprachige Konfiguration benötigt eine explizite Sprachmodellierung und eigene Tests; die erste Version ist bewusst deutsch.

### Bewertung der semantischen Suche als Folgeschritt

[pgvector](https://github.com/pgvector/pgvector) kann Vektoren in PostgreSQL speichern und Ähnlichkeit mit exakter Suche sowie HNSW-/IVFFlat-Indizes abfragen. Das ergänzt lexikalische Treffer um inhaltliche Nähe, verlangt aber vorab Entscheidungen über Embedding-Modell, Dimensionen, Versionierung, Kosten, Datenschutz und Aktualisierung bei geänderten Memories. Ohne definierten Provider und synthetischen Relevanzdatensatz ist die Qualität derzeit nicht belastbar bewertbar.

Ein separater Folgeschritt sollte zunächst exakte persona-gefilterte Vektorsuche gegen einen kleinen synthetischen deutschen Testkorpus mit Paraphrasen prüfen. Messen: Trefferqualität gegenüber Volltext, Latenz, Speicher und Embedding-Kosten; dabei Fremdpersona-Treffer strikt ausschließen. Erst danach ANN-Indizes bzw. hybride Rangfusion bewerten. Bei ANN können nachgelagerte Persona-Filter die Trefferzahl reduzieren; ein globales Top-k mit bloßem Filtern im Client ist daher keine geeignete Isolation. Embeddings müssen an Memory-ID, Persona-ID und Modellversion gebunden bleiben; Provenienz bleibt am Memory erhalten. Dieses Issue installiert keine Erweiterung, erzeugt keine Embeddings und sendet keine Inhalte an externe Anbieter.

## Konsequenzen

Die additive Migration baut den Index auch über Bestandsdaten auf und kann währenddessen Schreibzugriffe blockieren. Das bestehende Laufzeit-Migrationsmodell ist für die aktuelle lokale Datenmenge ausreichend; vor größeren produktiven Datenmengen wäre ein separat geplanter Online-Indexaufbau nötig. Die Suche ist weder Teilstring-, Tippfehler- noch semantische Suche. Es gibt vorerst keine Pagination über die begrenzte Ergebnismenge und keine Suche über Persona-Namen. Persona-Isolation ist Datenbereichstrennung, weiterhin keine Benutzer-Authentifizierung.

Quellen: [PostgreSQL: Steuerung der Volltextsuche](https://www.postgresql.org/docs/17/textsearch-controls.html), [PostgreSQL: Tabellen und Indizes](https://www.postgresql.org/docs/17/textsearch-tables.html), [pgvector: Filtering und Indexing](https://github.com/pgvector/pgvector#filtering). Tests prüfen echte HTTP-Suche, Normalisierung, UND-Verknüpfung, Rangfolge, Limits, Isolation, Provenienz, Migration und OpenAPI-Vertrag.
