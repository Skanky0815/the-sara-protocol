# ADR 0009: Native parallele Backend-Builds und Runtime-Packaging

- Datum: 2026-10-30
- Status: Vorgeschlagen
- An der Entscheidung beteiligt: Rico und Sara (KI-Assistentin)
- Dokumentiert von: Sara (KI-Assistentin)
- Bezug: #48, ergänzt ADR 0008

## Kontext
Der erste Delivery-Lauf nach PR #46 kompiliert zunächst ein natives Prüfimage und anschließend ein Multiarch-Image einschließlich emuliertem ARM-Build. Rico schlägt getrennte parallele Rust-Builds und nachfolgendes Binary-Packaging vor, wie in einem anderen Projekt erfolgreich eingesetzt.

## Entscheidung
Ein wiederverwendbarer Workflow trennt native Cargo-Release-Jobs für amd64/arm64 von nativen Runtime-Image-Jobs. Die Runner verwenden passende CPU-Architekturen; Cargo läuft in Rust 1.98 auf Debian Bookworm, passend zum Runtime-Image. Cargo-Abhängigkeiten werden pro Architektur gecacht. Binary-/Image-Artefakte bleiben im selben Workflow-Lauf und enthalten überprüfte SHA-, Architektur- und Prüfsummenmetadaten. Binary-Artefakte werden zusätzlich auf ELF-Architektur geprüft.

Der Runtime-Dockerfile enthält ausschließlich die Paketierung. Beide Images werden vor dem Upload nativ per Compose geprüft. Ein eigener Publisher nach main-QA übernimmt die fertigen Images, pusht sie und kombiniert die Architektur-Digests zum Multiarch-Index. Kein QEMU-Build und keine erneute Kompilierung im Publisher oder Release-Job. PRs erhalten keine Registry-Schreibrechte. Der bisherige Multi-Stage-Dockerfile bleibt als lokaler Quellcode-Build erhalten.

## Alternativen
Docker-Layer-Caching mit cargo-chef würde Abhängigkeiten besser wiederverwenden, beseitigt aber allein weder emulierte Kompilierung noch den doppelten Prüf-/Multiarch-Build. Docker dokumentiert ebenfalls die Verteilung von Plattform-Builds auf native Runner. Ein Cargo-Build direkt auf Ubuntu könnte eine neuere glibc verlangen als das Bookworm-Runtime-Image; deshalb die kompatible Buildumgebung. Weitere eigene Build- oder Cross-Compile-Werkzeuge sind nicht erforderlich.

## Konsequenzen
Kaltstarts kompilieren weiterhin je Architektur einmal; erwartete Vorteile sind Parallelität, native Ausführung und Cargo-Caching. Image-Artefakte verursachen Transferaufwand und werden nur einen Tag aufbewahrt. Veröffentlichung konsumiert keine Artefakte aus fremden Workflow-Läufen. Laufzeiten werden anhand CI-Ergebnissen beurteilt, nicht vorab versprochen. Digest-basierte Abnahme und Release-Grenzen aus ADR 0008 bleiben erhalten.

## Primärquellen
- [GitHub: native amd64-/arm64-Runner](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
- [Docker: Plattform-Builds auf mehrere Runner verteilen](https://docs.docker.com/build/ci/github-actions/multi-platform/#distribute-build-across-multiple-runners)
