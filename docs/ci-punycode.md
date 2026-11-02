# CI-Wartung: punycode im Cleanup (#54)

## Befund

Der [Lauf 41193423442](https://github.com/Skanky0815/the-sara-protocol/actions/runs/41193423442) („Prepare component release“, 31.10.2026) enthält im Job `prepare` um 10:39:23 UTC exakt die gemeldete Warnung `(node:2369) [DEP0040]`. Betroffen ist der Post-Schritt von `actions/setup-node@v4`, aufgelöst zu `49933ea5288caeca8642d1e84afbd3f7d6820020`. Der Lauf war erfolgreich. Auch der [Backend-delivery-Lauf 41194814514](https://github.com/Skanky0815/the-sara-protocol/actions/runs/41194814514) zeigt die Warnung im setup-node-Cleanup.

Der offizielle [v4-Lockfile](https://github.com/actions/setup-node/blob/49933ea5288caeca8642d1e84afbd3f7d6820020/package-lock.json) enthält `@actions/cache 4.0.3`, `@azure/ms-rest-js 2.7.0` sowie `@azure/storage-blob 12.16.0` → `@azure/core-http 3.0.3`. Diese laden `node-fetch 2.7.0` → `whatwg-url 5.0.0` → `tr46 0.0.3`. Das gebündelte `dist/cache-save/index.js` referenziert daraus das eingebaute `require("punycode")` (ncc-Modul 4876, Zeile 88599). Schon das Laden des Post-Bundles erzeugt die Warnung, auch ohne aktiviertes Paketmanager-Caching.

## Entscheidung und fokussierter Nachweis

Die gepflegte offizielle [Version v7.1.0](https://github.com/actions/setup-node/releases/tag/v7.1.0) ersetzt v4 an allen drei setup-node-Einsatzstellen. Der SHA-Pin lautet `949feb2413d6458794dcd2491c4babbbce0c15c1`. Die Action läuft mit Node.js 24; die gewählte Node-Version bleibt ebenfalls 24. Die GitHub-gehosteten Runner erfüllen die erforderliche Runner-Version. Bestehende andere Pins und Berechtigungen bleiben erhalten.

Ein eigener Bundle-Patch wäre wartungsintensiv und unnötig. Globale Warnungsunterdrückung oder ein Downgrade beheben die Ursache nicht. Ein breites Upgrade anderer Actions gehört nicht zu dieser Änderung; Warnungen von Artefakt-Actions im Hauptschritt sind separat zu behandeln.

Lokaler Vergleich mit Node.js 24.22.1 und `node --trace-deprecation dist/cache-save/index.js`, ohne Token oder GitHub-Umgebungsdaten: v4 erzeugt DEP0040 mit Stack zu Modul 4876, v7.1.0 beendet denselben Aufruf erfolgreich ohne Deprecation-Warnung. Beide melden lediglich, dass kein Cache konfiguriert ist. Für den echten Cleanup-Nachweis dient der Workflow „Workflow validation“ auf dem PR-Commit, der dieselbe gepinnte Action verwendet. Release-Workflows werden dafür nicht manuell ausgelöst, da sie Veröffentlichungen verändern können.

## GitHub-Actions-Nachweis

[Workflow validation 41403925779](https://github.com/Skanky0815/the-sara-protocol/actions/runs/41403925779/job/125141579204) auf Commit `1024b123cfd6ee31daff414553896e2fb1f6af16` ist erfolgreich: actionlint, 18 Python-Tests und 7 Release-Tests bestanden. Der setup-node-Post-Schritt um 07:04:44 UTC am 02.11.2026 enthält keine DEP0040-Warnung. Die unabhängige Action `actions/setup-python@v5` meldet im selben Lauf weiterhin DEP0040 (Haupt- und Post-Schritt); das ist nicht die Ursache des ursprünglichen Release-Cleanup-Befunds und wird hier nicht durch ein weiteres ungeprüftes Upgrade erweitert.

## Erweiterte Workflow-Prüfung am 02.11.2026

Auf Ricos ausdrücklichen Folgeauftrag wurden alle sieben Workflow-Dateien geprüft. Die offiziellen Release-Tags wurden über die GitHub-API auf Commit-SHAs aufgelöst und die README-/Action-Verträge auf verwendete Inputs und Laufzeiten geprüft.

| Bestandteil | Vorher | Ergebnis |
| --- | --- | --- |
| checkout | v4 | v7.0.1, SHA-gepinnt in allen Workflows |
| setup-python | v5 | v7.1.0, SHA-gepinnt; behebt den weiteren alten Node-/Cache-Pfad |
| upload-artifact | v4 | v7.0.2, SHA-gepinnt |
| download-artifact | v4 | v8.0.2, SHA-gepinnt |
| rust-cache | bewegliches v2 | aktuelles v2.9.3, SHA-gepinnt |
| install-action | älterer v2-SHA | v2.89.0, SHA-gepinnt |
| actionlint | 1.7.7 | 1.7.12 |
| pytest | 8.3.5 | 9.2.0 |
| setup-node | v4 | v7.1.0 wie oben |
| Docker-Actions | gepinnte v4/v7/v4 | bereits aktuelle Releases: setup-buildx v4.4.1, build-push v7.4.0, login v4.7.0 |
| cargo-audit | 0.22.2 | aktuellstes cargo-audit-Release, unverändert |

Die aktualisierten JavaScript-Actions verwenden Node.js 24. Die gehosteten Runner erfüllen die Mindestversion 2.327.1 (checkout-Container-Credentials: 2.329.0). Der Delivery-Workflow prüft weiterhin Herkunft, Push-Ereignis und main-Branch; checkout v7 benötigt hier kein unsicheres Fork-Opt-in. Upload behält das standardmäßige ZIP-Format, Namen, Pfade und Metadaten; download v8 unterstützt diese benannten Artefakte. Die Prüfung umfasst den echten Upload-/Download-/Image-Smoke-Pfad für amd64 und arm64 in der PR-CI.

Python 3.12, Node.js 24, PostgreSQL 17 und die Ubuntu-24.04-Runner sind weiterhin unterstützte Laufzeiten und bleiben erhalten. Der native Build ist bewusst auf Rust 1.98/Bookworm festgelegt; das verfügbare Rust 1.99.1 wäre ein separater Toolchain-Wechsel und gehört nicht zur hier bestätigten Action-/Prüfwerkzeug-Aktualisierung. Rust-QA verwendet bereits `stable`. Debian Bookworm bleibt die gemeinsame glibc-Basis von Build und Runtime. Keine Berechtigungen, Release-Trigger oder Veröffentlichungsregeln wurden erweitert.

Quellen: offizielle Release-Seiten von [checkout](https://github.com/actions/checkout/releases/tag/v7.0.1), [setup-python](https://github.com/actions/setup-python/releases/tag/v7.1.0), [upload-artifact](https://github.com/actions/upload-artifact/releases/tag/v7.0.2), [download-artifact](https://github.com/actions/download-artifact/releases/tag/v8.0.2), [rust-cache](https://github.com/Swatinem/rust-cache/releases/tag/v2.9.3), [install-action](https://github.com/taiki-e/install-action/releases/tag/v2.89.0), [actionlint](https://github.com/rhysd/actionlint/releases/tag/v1.7.12) und [pytest](https://github.com/pytest-dev/pytest/releases/tag/9.2.0). Alternative: individuelle Bundle-Patches würden eigene Wartung erfordern; die gepflegten offiziellen Actions decken die bestehenden Anforderungen ab.
