# ADR 0006: OpenAPI aus Handlern und Datentypen generieren

- Datum: 2026-10-29
- Status: Vorgeschlagen
- Dokumentiert von: Sara
- Bezug: Issue #12 und Ricos Review von PR #38

## Kontext

Die REST-Endpunkte brauchen einen maschinenlesbaren Vertrag für lokale Clients und eine spätere ChatGPT-Anbindung. Eine zusätzlich gepflegte Referenz in Markdown oder einer statischen OpenAPI-Datei kann vom Code abweichen.

## Entscheidung

`utoipa` generiert OpenAPI 3.1 aus Datentypen und Handler-Annotationen. `utoipa-axum` verbindet deren Registrierung im Axum-Router mit der Sammlung der Metadaten. Das laufende Backend liefert das Dokument unter `/openapi.json`; eine separate Referenzdatei wird nicht eingecheckt. Markdown dient der Einrichtung und fachlichen Erklärung und verweist auf die generierte Referenz.

Vertragstests validieren echte HTTP-Antworten mit `jsonschema` gegen die generierten Schemas. Der Validator wird nur in Tests genutzt, ohne Netzwerk-/Datei-Resolver. Die vorhandenen Funktions- und Isolationstests bleiben erhalten.

## Alternativen

Eine handgeschriebene OpenAPI-Datei ist unabhängig vom Rust-Stack, verdoppelt aber die Pflege von Feldern und Routen. Eine separate Liste von Handlern nur für die Dokumentation bleibt ebenfalls anfällig für ausgelassene oder umbenannte Endpunkte.

## Konsequenzen

Neue Handler werden annotiert und über den OpenAPI-Router registriert. Nicht aus Typen ableitbare Fehlerantworten und fachliche Regeln bleiben bewusst dokumentierte Angaben am Code; Tests prüfen sie gegen das reale Verhalten. Die bestehenden API-Antworten und das Datenbankschema bleiben erhalten. Dies schafft einen Vertrag, aber noch keine Authentifizierung oder Client-Verbindung.
