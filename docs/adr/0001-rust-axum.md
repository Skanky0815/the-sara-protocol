# ADR 0001: Rust-Webdienst mit Axum

- Datum: 2026-10-27 (nachträglich dokumentiert)
- Status: Akzeptiert
- An der Entscheidung beteiligt: Rico und Sara (KI-Assistentin)
- Dokumentiert von: Sara (KI-Assistentin)

## Kontext
The Sara Protocol benötigt einen kleinen, leistungsfähigen, primär lokal betriebenen REST-Dienst für Personas und Erinnerungen. Die erste Entwicklung erfolgt unter macOS.

## Entscheidung
Rust mit Axum, Tokio, Serde und SQLx für den ersten Dienst verwenden.

## Alternativen
Kotlin/Spring Boot und TypeScript/Fastify wurden erwogen.

## Konsequenzen
Hohe Typsicherheit und geringer Laufzeitaufwand gehen mit zusätzlichem Lern- und Build-Aufwand für Rust einher. LLM-Latenz und die Qualität des Abrufs können die Gesamtleistung stärker bestimmen.
