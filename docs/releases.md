# Versionierung, Abnahmeimages und Releases

## Werkzeug und Versionsregeln

Wir verwenden **Release Please 17.6.0**, fest im [npm-Lockfile](../.github/release/package-lock.json) gebunden. Das Werkzeug erstellt Versions-PRs, aktualisiert Rust-Manifeste und Lockfiles, erzeugt Changelogs und übernimmt die GitHub-Release-Erstellung. Die kleine [Integration](../.github/release/release.cjs) ergänzt nur unsere QA-/Abnahmegrenzen und Zuordnungen; sie enthält keine eigene Versionsarithmetik oder Datei-Updater. Der [Werkzeugvergleich](release-tooling-evaluation.md) begründet die Auswahl.

Backend und künftiges Frontend haben eigene Versionen, Changelogs und Tags. Backend: `backend/Cargo.toml`, `backend/Cargo.lock`, `backend/CHANGELOG.md`; später Frontend: `frontend/package.json`, Paket-Lockfile und `frontend/CHANGELOG.md`. Die [Konfiguration](../release-please-config.json) registriert derzeit ausschließlich `backend` mit der Rust-Strategie. Das [Versionsmanifest](../.release-please-manifest.json) beginnt leer, weil noch kein Release veröffentlicht wurde; Release Please aktualisiert es im Versions-PR. Frontend wird erst ergänzt, wenn dessen echte Quellen und QA vorliegen.

Die Versionsvorschläge folgen **Conventional Commits**. Vor `1.0.0` erhöht eine inkompatible Änderung (`feat!:`/`BREAKING CHANGE:`) die Minor-Version; kompatible Funktionen (`feat:`) und Fehlerkorrekturen (`fix:`) erhöhen die Patch-Version. Ab `1.0.0` gelten die regulären SemVer-Regeln. Die öffentliche Backend-Schnittstelle umfasst HTTP-Vertrag, dokumentierte Konfiguration und Betriebsanforderungen. Kompatibilität bestehender Daten muss im Review beurteilt werden; kein Versionswerkzeug kann das vollständig automatisch erkennen.

Tags heißen `backend/vX.Y.Z`, später `frontend/vX.Y.Z`. Der erste Release ist `backend/v0.1.0`. Vorabversionen sind derzeit nicht freigeschaltet. Es gibt keine gemeinsame Backend-/Frontend-Version. Für bewusst abweichende Versionsvorschläge die [Release-As-Konvention von Release Please](https://github.com/googleapis/release-please#how-do-i-change-the-version-number) verwenden und im Versions-PR prüfen; keinen zweiten Versionsrechner einführen.

## Einrichtung und Rechte

Die Workflows müssen regulär nach main gemergt sein. **Settings → Actions → General → Workflow permissions → Allow GitHub Actions to create and approve pull requests** muss bewusst freigegeben werden, damit GITHUB_TOKEN Versions-PRs erstellen darf. Diese Einstellung wurde nicht automatisch geändert. Die Integration gibt keine Reviews ab und mergt keine PRs. Standard-Tokenrechte können lesend bleiben.

Die Vorbereitungs-Action erhält Schreibrechte auf Inhalte, Issues und PRs sowie Actions-Rechte für einen ausdrücklichen QA-Dispatch. Der Image-Job erhält nur Inhaltslese- und Paket-Schreibrechte. Der separate Release-Job benötigt Inhalts-, Issue-, PR- und Paket-Schreibrechte sowie Actions-Leserechte. Nur erfolgreiche main-Push-QA aus diesem Repository darf ihn auslösen. Es werden keine PR-Artefakte geladen, keine PR-Caches im privilegierten Job verwendet und keine Zugangsdaten im Checkout gespeichert. Docker-/npm-Downloads stammen aus den konfigurierten Werkzeugen; npm-Lifecycle-Skripte sind bei der Installation deaktiviert.

GHCR muss Actions die Erstellung bzw. Aktualisierung von `ghcr.io/skanky0815/the-sara-protocol-backend` erlauben. Bestehende Pakete benötigen gegebenenfalls expliziten Repository-Zugriff. Paket-Sichtbarkeit separat prüfen: ein öffentliches Repository macht ein neu angelegtes GHCR-Paket nicht zwangsläufig öffentlich. Keine Sichtbarkeits- oder Schutzregel wird automatisch geändert.

## Bewusster Versions-PR

1. In **Actions → Prepare component release → Run workflow** den Branch **main** und **backend** wählen. Das ist der bewusste Vorbereitungsauftrag; normale Pushes erstellen keinen Versions-PR.
2. Release Please berechnet den Vorschlag aus Komponenten-Commits und erstellt bzw. aktualisiert seinen Versions-PR. Manifest, Lockfile, Changelog und Versionsmanifest gemeinsam prüfen. Ein identischer Auftrag erzeugt keine unabhängigen doppelten Versions-PRs.
3. Die Integration weist den PR `Sara-Pixel42` zu, fordert `Skanky0815` zum Review auf, ergänzt den direkten Branch-Link in #44 und startet Rust-QA per `workflow_dispatch` auf dem Release-Branch. Release Please erzeugt den Branch selbst; eine native Issue-Branch-Verknüpfung wird dabei nicht behauptet.
4. Falls GitHub zusätzlich genehmigungspflichtige bot-erzeugte PR-Workflows anzeigt, diese normal freigeben. Vor dem Merge den QA-Lauf für die tatsächliche Head-SHA kontrollieren. Die Repo-Schutzregeln bleiben verbindlich; kein automatischer Merge.
5. Nach regulärem Merge durchläuft genau der main-Commit Rust-QA, Containerprüfung, Image-Bereitstellung und erst dann gegebenenfalls die Release-Freigabe.

Frontend wird nicht vorbereitet; die Workflow-Auswahl führt ohne vorhandene Implementierung zu keinem Job. Bei fehlgeschlagenem Review-/QA-Handoff den bestehenden PR prüfen und `Prepare component release` erneut starten. Bei unverändertem PR meldet Release Please gegebenenfalls keine Aktualisierung; dann die Review-Anforderung und **Rust QA → Run workflow** auf dem bestehenden Release-Branch manuell nachholen. Ein erneuter Vorbereitungsauftrag kann den PR aktualisieren und damit ein neues Approval notwendig machen.

## Abnahme ohne lokalen Build

Nach erfolgreicher main-QA baut **Backend delivery** die Release-Binaries in parallelen Jobs auf nativen `ubuntu-24.04`- und `ubuntu-24.04-arm`-Runnern. Der Cargo-Build läuft jeweils in `rust:1.98-bookworm`, sodass die glibc-Umgebung zum Debian-Bookworm-Runtime-Image passt. Ein architekturspezifischer Cargo-Cache hält Abhängigkeiten für spätere Läufe vor; main speichert keine von PRs erzeugten Cargo-Caches. Es wird kein Rust unter QEMU kompiliert.

Die nachfolgenden Image-Jobs laufen ebenfalls nativ. Sie übernehmen ausschließlich Binary-Artefakte aus demselben Workflow-Lauf und prüfen Commit-SHA, Architektur, SHA256 und ELF-Header. `backend/Dockerfile.runtime` kopiert die fertige Binary und Migrationen; er enthält keinen Rust-Compiler oder Cargo-Aufruf. Beide Architekturimages werden per Compose gestartet und neu gestartet. Danach werden die geprüften Images als unkomprimierte Docker-Artefakte mit SHA-/Architektur-/Prüfsummen-Metadaten für einen Tag gespeichert.

Nur der main-Publisher lädt diese gleichlaufenden Image-Artefakte, prüft sie erneut, pusht die beiden nativen Images und setzt mit Docker Buildx den Multiarch-Index zusammen. Er kompiliert und baut keine Images erneut. Zwischen-Tags `build-<run>-<attempt>-<arch>` dienen der Manifest-Zusammenstellung; für die Abnahme wird ausschließlich die unten beschriebene Commit-/Digest-Referenz verwendet. Bestehende Commit-Tags bleiben erhalten. Die Release-Übernahme verwendet weiterhin denselben Digest. Der bisherige `backend/Dockerfile` bleibt für einen lokalen Build aus Quellcode verfügbar.

PRs verwenden dieselben getrennten Build-/Image-Prüfungen mit reinen Leserechten, aber keinen Registry-Publisher. Der zusätzliche Artefakttransfer ist Teil der Pipeline-Laufzeit. Ein Kaltstart muss beide Architekturen zunächst vollständig kompilieren; eine konkrete Beschleunigung wird erst anhand tatsächlicher Läufe beurteilt. Siehe [ADR 0009](adr/0009-native-backend-builds.md).

Commit-Images heißen `ghcr.io/skanky0815/the-sara-protocol-backend:sha-<vollständige Commit-SHA>`. Ein vorhandenes Commit-Image wird wiederverwendet. Der Workflow gibt im Job-Summary die genaue Referenz `ghcr.io/...@sha256:...` aus. **Für die Abnahme diesen Digest verwenden**, da Tags grundsätzlich Registry-Referenzen sind. Commit-Images sind noch keine stabilen Releases und benötigen nicht die bereits abgeschlossene Abnahme; sonst könnte die erste Abnahme nicht stattfinden.

Das CI-Image muss einmal erzeugt sein, bevor die folgenden Befehle funktionieren. Die Datei [compose.acceptance.yaml](../compose.acceptance.yaml) enthält ausdrücklich keinen Build-Kontext und mountet keinen Quellcode. Sie benötigt auf dem Abnahmerechner nur Docker Compose, keinen Rust-Compiler:

```sh
# Aus einem Checkout der passenden Dokumentation/Compose-Datei:
export BACKEND_IMAGE=ghcr.io/skanky0815/the-sara-protocol-backend@sha256:DEIN_DIGEST
# Lokal einen eigenen URL-sicheren Wert verwenden (z. B. zufällige Hex-Zeichen).
export POSTGRES_PASSWORD=DEIN_LOKALER_HEX_WERT
export BACKEND_PORT=53043

docker compose -p sara-acceptance -f compose.acceptance.yaml pull
docker compose -p sara-acceptance -f compose.acceptance.yaml up --no-build -d --wait
curl --fail http://127.0.0.1:53043/health
curl --fail http://127.0.0.1:53043/openapi.json
```

Bei privatem Paket zuvor mit einem für Paketlesen berechtigten Konto bei `ghcr.io` anmelden, Zugangsdaten ausschließlich über `docker login --password-stdin` oder den lokalen Credential-Store verwenden. Keine Tokens in Befehlsargumente, Compose-Dateien oder Git eintragen. Im Job-Token sind Paket-Schreibrechte ausreichend; auf dem lokalen Abnahmerechner genügen Leserechte. Die Beispielpasswörter und Loopback-Ports sind für die lokale Abnahme, kein öffentliches Produktionsdeployment mit Authentifizierung/TLS.

PostgreSQL ist nur im Compose-Netz erreichbar und erhält ein eigenes Volume unter dem Projekt `sara-acceptance`. Für einen Neustart und den Persistenznachweis:

```sh
docker compose -p sara-acceptance -f compose.acceptance.yaml restart backend
docker compose -p sara-acceptance -f compose.acceptance.yaml up --no-build -d --wait
# Zuvor angelegte Test-Persona/Memories erneut per API abrufen.
docker compose -p sara-acceptance -f compose.acceptance.yaml down
```

`down` behält die Daten. Nur für ausdrücklich entbehrliche Abnahmedaten löscht `down --volumes` das eigene Abnahmevolume. Der vollständige fachliche Nachweis (Persona, Provenienz, Isolation und Persistenz) gehört zu #43; die automatisierte Image-Prüfung ersetzt diese Abnahme nicht. Der absolute Migrationspfad im Image funktioniert unabhängig vom lokalen Arbeitsverzeichnis.

## Versionierte Veröffentlichung

Eine Veröffentlichung erfordert einen von Release Please erkannten gemergten Backend-Versions-PR für **genau den erfolgreichen QA-Commit**. Die Integration prüft den echten QA-Workflow, dessen vier erfolgreichen Jobs, main-Historie, PR-Dateiumfang und Versionskonsistenz zwischen Cargo und Release-Manifest. #43 muss geschlossen sein. Ohne Kandidaten entsteht nur das Commit-Abnahmeimage.

Vor dem GitHub Release wird der bereits bereitgestellte Digest als Image-Tag `:X.Y.Z` übernommen. Es erfolgt kein erneuter Release-Build. Ein vorhandener Versionstag muss denselben Digest haben, andernfalls bricht der Job ab. Erst danach erstellt Release Please über seine Standardbibliothek den GitHub Release samt Git-Tag für diesen Commit und markiert den PR als veröffentlicht. Existierende Git-Tags auf anderen Commits werden nicht überschrieben. Es gibt kein `latest`-Image und kein Cargo-/npm-Publishing.

Bei einem Teilfehler denselben ursprünglichen Rust-QA-main-Lauf erneut ausführen. Dadurch werden vorhandene Commit-Images wiederverwendet und die Release-Prüfung erneut gestartet. Ein bereits erzeugter korrekter Release wird nicht dupliziert. Später weitergelaufenes main ersetzt niemals die SHA des ursprünglichen Auftrags. Fehlende Berechtigungen, GHCR-Fehler oder offene Abnahme führen zum Abbruch, nicht zu einer Umgehung.

## Validierung und verbleibender Betriebsnachweis

Workflow validation führt `actionlint`, bestehende Python-Tests sowie Node-Tests für unsere QA-Grenzen und die tatsächlichen Release-Please-Updater mit den echten Cargo-Dateien aus. **Backend image check** baut native Release-Binaries und paketiert PR-Images für beide Architekturen; Containerstart/Neustart erfolgen ohne Registry-Schreibrechte. Die Release-Please-Konfiguration wurde mit der vereinbarten 0.x-Regel geprüft. Manifest-Tests decken sowohl den leeren Bootstrap-Zustand als auch befüllte Komponenten-Versionen ab und vergleichen das Manifest des jeweiligen Checkouts mit dem Ergebnis des echten Release-Please-Loaders.

Der echte GITHUB_TOKEN-Versions-PR, dessen QA-Dispatch, die erste GHCR-Multiarch-Veröffentlichung und der Pull auf dem Abnahmerechner können erst nach Workflow-Merge und den erforderlichen Berechtigungen nachgewiesen werden. #44 und #47 bleiben bis zu diesen Nachweisen offen. Dieser Implementierungs-PR erzeugt weder einen Release noch einen Tag oder ein GHCR-Image.

## Primärquellen

- [Release Please: Manifest-Modus](https://github.com/googleapis/release-please/blob/main/docs/manifest-releaser.md)
- [Rust-Strategie und Cargo-Lockfile-Updater](https://github.com/googleapis/release-please/blob/main/src/strategies/rust.ts)
- [GitHub: GITHUB_TOKEN und Workflow-Auslöser](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow)
- [Docker: Multiarch-Builds in GitHub Actions](https://docs.docker.com/build/ci/github-actions/multi-platform/)
- [GitHub: Images in GHCR veröffentlichen](https://docs.github.com/en/actions/tutorials/publish-packages/publish-docker-images)
