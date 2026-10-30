# Versionierung und Releases

## Komponenten und öffentliche Schnittstellen

Backend und künftiges Frontend werden unabhängig versioniert. Die Backend-Version steht in `backend/Cargo.toml`; der lokale Paketeintrag in `backend/Cargo.lock` muss übereinstimmen. Änderungen stehen in [backend/CHANGELOG.md](../backend/CHANGELOG.md). Das spätere Frontend verwendet sein eigenes `frontend/package.json`, seinen Paket-Lockfile und `frontend/CHANGELOG.md`. Diese Dateien werden erst mit dem echten Frontend angelegt.

Tags heißen `backend/vX.Y.Z` bzw. später `frontend/vX.Y.Z`; gleichnamige Komponentenversionen sind unabhängig. Es gibt keine gemeinsame Produktversion und keinen globalen „Latest“-Release. Öffentliche Backend-Schnittstellen umfassen HTTP-Vertrag, dokumentierte Konfiguration und Betriebsanforderungen. Datenbankmigrationen und Kompatibilität bereits gespeicherter Daten werden bei Änderungen ausdrücklich bewertet.

Vor `1.0.0` erhöht eine inkompatible Änderung die Minor-Version; kompatible Ergänzungen und Fehlerkorrekturen erhöhen die Patch-Version. `major` dient dem bewusst gewählten Übergang zur stabilen Schnittstelle `1.0.0`. Danach gelten die regulären Regeln: inkompatibel → Major, kompatible Ergänzung → Minor, Fehlerkorrektur → Patch. Die Auswahl ist eine Review-Entscheidung; die Automation kann API-Kompatibilität nicht zuverlässig selbst beurteilen. Vorabversionen werden zunächst nicht automatisiert unterstützt und als Eingabe abgewiesen.

`0.1.0` in Cargo ist bislang eine Entwicklungsversion, kein bereits veröffentlichter Release. Der erste Release verwendet `initial`, behält die Paketversion bei und veröffentlicht erst nach der abgeschlossenen Abnahme in [#43](https://github.com/Skanky0815/the-sara-protocol/issues/43).

## Einrichtung

Die Workflows müssen zunächst per normalem PR nach `main` gelangen. Unter **Settings → Actions → General → Workflow permissions** muss **Allow GitHub Actions to create and approve pull requests** aktiviert sein, damit der eingebaute `GITHUB_TOKEN` Versions-PRs erstellen darf. Diese Repository-Einstellung wird durch die Implementierung nicht verändert. Die Automation erstellt keine Reviews und genehmigt oder mergt keine PRs. Die Standardberechtigung kann weiterhin lesend bleiben; Schreibrechte werden nur den jeweiligen Jobs erteilt.

`Prepare component release` benötigt `contents: write`, `pull-requests: write` und `actions: write` für Release-Branch, PR und ausdrücklichen QA-Start. `Publish component release` erhält `contents: write` für Tag/Release und Leserechte für Actions, PRs und Issues. Kein persönlicher Token und keine weiteren Secrets sind nötig. Tag-Regelsätze müssen die reguläre Erstellung des vorgesehenen Tags zulassen; die Automation umgeht keine Regeln. Eine fehlende Berechtigung führt zum Abbruch.

## Bedienung

1. Änderungen unter `## [Unveröffentlicht]` im Backend-Changelog sammeln. Der Abschnitt muss aussagekräftige Release-Notizen enthalten.
2. In Actions **Prepare component release → Run workflow**, Branch **main**, Komponente **backend**, Sprung **initial/patch/minor/major** wählen. Der erste Release benutzt **initial**. Weitere Sprünge setzen den veröffentlichten Vorgänger-Release in der main-Historie voraus; `patch` kann die erste Abnahme nicht umgehen. Diese Handlung ist der bewusste Release-Auftrag.
3. Die Action erzeugt ausschließlich Manifest-/Lockfile-Version und Changelog auf `release/backend/vX.Y.Z` und erstellt einen PR mit Bezug zu #44. Für `initial` ändert sich nur das Changelog. Es gibt keinen direkten Push auf main und keinen automatischen Merge.
4. Rust-QA wird auf dem Release-Branch ausdrücklich per `workflow_dispatch` gestartet. Auf diesen Lauf und die richtige Head-SHA achten. Bei zusätzlich angezeigten, genehmigungspflichtigen PR-Workflows diese über die normale GitHub-Oberfläche freigeben. Review und Branch-Schutz bleiben verbindlich.
5. Versions-PR prüfen und regulär squashen oder rebasen. Nach dem Merge läuft Rust-QA erneut auf main. Erst ein erfolgreicher Lauf für genau diesen Merge-Commit löst die Veröffentlichung aus.
6. Der veröffentlichte Tag zeigt auf den tatsächlich gemergten Commit, nicht auf den vorherigen Feature-Branch oder den inzwischen weitergelaufenen main. Der GitHub Release enthält die Notizen der Komponente. Es werden keine Pakete, Images oder Deployments veröffentlicht.

Frontend-Auswahl wird vor jeglicher Änderung zurückgewiesen. Erst mit Implementierung der Frontend-Versionierung, Lockfile-Behandlung und eigenen QA darf diese Komponente freigeschaltet werden.

## Prüfungen und Wiederholung

Die Veröffentlichung akzeptiert ausschließlich erfolgreiche `push`-Läufe des echten Rust-QA-Workflows auf main aus diesem Repository. Sie prüft die vier Jobs **Format**, **Clippy**, **Tests (PostgreSQL)** und **Dependency audit**, einen zu diesem Commit gemergten Versions-PR, dessen Release-Markierung und Dateiumfang sowie die exakte Versions-/Changelog-Änderung gegenüber dem direkten Elterncommit. Die Privilegien des `workflow_run`-Jobs werden nie für PR-Checkouts, fremde Repositorys, heruntergeladene Artefakte oder Caches verwendet.

Ein vorhandener Release-Branch oder Tag blockiert einen weiteren Vorbereitungsauftrag; es wird weder überschrieben noch ein zweiter Versions-PR erzeugt. Bei einem Abbruch nach Branch-Erstellung den existierenden Branch prüfen und gegebenenfalls den fehlenden PR manuell anlegen. Release-Markierung und Standardbeschreibung müssen erhalten bleiben; keine zusätzlichen Dateien in einen Versions-PR aufnehmen. Wenn der QA-Dispatch fehlschlägt, Rust QA über **Run workflow** auf diesem Branch starten. Beispiel:

```sh
gh workflow run rust-qa.yml --ref release/backend/v0.1.0
```

Wird der Versions-PR durch main-Änderungen veraltet, ihn normal aktualisieren, erneut prüfen und nötigenfalls erneut approven. Hat sich dabei die Komponentenversion geändert, den überholten Auftrag schließen und einen neuen vorbereiten. Bei einem reinen Rebase bleibt der bewusste Release-Auftrag bestehen.

Nach erfolgreicher Veröffentlichung ist eine Wiederholung ohne zusätzliche Schreiboperationen möglich. Existiert nach einem Teilfehler bereits der korrekte Tag, wird nur der fehlende Release ergänzt. Ein Tag auf einem anderen Commit wird niemals verschoben. Bei offenen Abnahmekriterien für `0.1.0` wird vor der Tag-Erstellung abgebrochen. Nach Schließen von #43 oder Beheben vorübergehender Berechtigungs-/API-Fehler den ursprünglichen Rust-QA-Lauf des Merge-Commits erneut ausführen; dadurch wird die Veröffentlichung erneut geprüft. Keine Tags manuell auf main „nachziehen“.

## Validierung und noch notwendiger Betriebsnachweis

Lokal und in Workflow validation werden Versions-/Changelog-Planung, Ablehnungen, simulierte GitHub-API-Abläufe, QA-Dispatch, Veröffentlichung und Wiederholung getestet. Die echte Backend-Manifest-/Lockfile-Struktur ist einbezogen. `actionlint` prüft die Workflows.

Die erstmalige tatsächliche Erstellung eines Versions-PRs und der QA-Start mit `GITHUB_TOKEN` können erst nach Merge der Workflows und Freigabe der Repository-Einstellung nachgewiesen werden. Diesen Betriebsnachweis in #44 festhalten; lokale Simulationen ersetzen ihn nicht. Bis zum bewussten ersten Release-Auftrag werden weder Tag noch Release erstellt. Die Abnahme #43 muss vor der ersten Veröffentlichung abgeschlossen sein.

## Quellen

- [Semantic Versioning 2.0.0](https://semver.org/lang/de/)
- [GitHub: Workflows aus anderen Workflows auslösen](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow) — `workflow_dispatch` startet auch mit `GITHUB_TOKEN` einen Lauf; bot-erzeugte PR-Ereignisse können zusätzliche Freigabe erfordern.
- [GitHub: workflow_run und dessen Sicherheitsgrenzen](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run)
