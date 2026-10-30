# Generierte API-Referenz

Die verbindliche HTTP-Referenz wird aus dem Rust-Code als OpenAPI 3.1 erzeugt. Nach dem lokalen Start ist sie unter [`http://127.0.0.1:3000/openapi.json`](http://127.0.0.1:3000/openapi.json) erreichbar:

```sh
curl --fail http://127.0.0.1:3000/openapi.json
# Optional als Datei für einen OpenAPI-Client speichern:
curl --fail http://127.0.0.1:3000/openapi.json -o /tmp/sara-openapi.json
```

Die normale Backend-Instanz dokumentiert die fünf fachlichen Operationen für Health, Personas und Memories. Die JSON-Referenz selbst ist ein Metadaten-Endpunkt und wird nicht rekursiv in ihren eigenen Vertrag aufgenommen. Die nur in Tests verwendete Health-App dokumentiert entsprechend nur Health. Server-URLs sind relativ (`/`), damit die Referenz keine lokale Host-Adresse festschreibt. Die Referenz führt weder API-Aufrufe aus noch stellt sie eine ChatGPT-Verbindung her.

## Pflege am Code

- `utoipa::ToSchema` beschreibt Request-/Response-Datentypen direkt neben ihren Rust-Definitionen. Serde-Feldregeln, Pflichtfelder und Enum-Werte werden übernommen; Zusatzangaben erklären fachliche Regeln und Zeitformate.
- `utoipa::path` dokumentiert Operation, Parameter, Statuscodes, Antwortmedien und Header bei jedem Handler. Deutsche Rust-Dokumentationskommentare werden als Beschreibung verwendet.
- `utoipa-axum` registriert Handler und sammelt deren OpenAPI-Metadaten gemeinsam. Es gibt keine zweite, unabhängig gepflegte Liste von Endpunkten.
- Unerwartete Datenbankfehler bleiben leere HTTP-Fehlerantworten. Axum-Validierungsfehler können Text enthalten; bei gemischten Validierungsantworten ist der optionale Text-Body beschrieben. Headerfehler, Routing-Fallbacks und implizite HEAD-Unterstützung bleiben Verhalten des HTTP-Frameworks außerhalb der fünf fachlichen Operationen.

Neue Endpunkte werden mit `routes!(handler)` in einem `OpenApiRouter` registriert. Geänderte DTOs und Fehlerpfade müssen zusammen mit den Annotationen angepasst werden. Das Schema ersetzt keine Laufzeitvalidierung und führt keine neue Authentifizierung ein.

## Vertragstests

Die Tests unter `backend/tests/openapi/` holen die Referenz über den tatsächlichen JSON-Endpunkt ab und prüfen:

- registrierte fachliche Pfade, eindeutige Operation-IDs und vorhandene Schemas;
- echte Health-, Persona- und Memory-Antworten gegen die generierten JSON-Schemas, inklusive Zeitformaten, Listen und Enum-Werten;
- Erfolg, Namenskonflikte, nicht vorhandene Personas, ID-/JSON-Validierung, Content-Type, Body-Limit und leere Datenbankfehlerantworten.

Der JSON-Schema-Validator ist nur eine Testabhängigkeit; HTTP-/Datei-Resolver sind deaktiviert. Die Validierung braucht keine externe Netzwerkverbindung. PostgreSQL-Tests benötigen eine dedizierte Datenbank; ohne `DATABASE_URL` werden diese Fälle übersprungen.

```sh
# Aus backend/ mit einer vorhandenen Testdatenbank:
export DATABASE_URL=postgres://sara:sara_local_dev@127.0.0.1:5432/sara_test
cargo test --locked --test openapi_contract
# Gesamte Suite und Qualitätsprüfungen:
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo audit
```

Die Einrichtung der Testdatenbank steht in der [Entwicklungsanleitung](getting-started.md). Fachliche Beispiele stehen in der [Persona-Anleitung](persona-api.md) und im [IntelliJ HTTP Client](../backend/dev/persona.http). Die vollständige Referenz wird nicht als separat gepflegte JSON-/YAML-Datei eingecheckt.

## Zeitangaben für Memories

`observed_at` muss RFC3339 mit expliziter Zeitzone erfüllen. Der normalisierte UTC-Zeitpunkt muss im Jahr 0001 bis 9999 liegen. Datum ohne Uhrzeit, fehlende Zeitzone, relative PostgreSQL-Ausdrücke und `infinity`/`-infinity` werden vor dem Schreiben mit HTTP 400 abgelehnt. Die Ausgabe erfolgt in UTC mit sechs Nachkommastellen; zusätzliche Nachkommastellen werden bei der Eingabe abgeschnitten.

Bereits vorhandene Zeitwerte außerhalb dieses Vertrags führen beim Lesen der betroffenen Memory-Liste zu einer kontrollierten, leeren HTTP-500-Antwort. Die Daten bleiben erhalten; es werden weder Einträge still ausgeblendet noch automatisch gelöscht. Andere Personas und Health bleiben erreichbar. Eine Bereinigung solcher Altdaten braucht eine gesonderte Entscheidung.
