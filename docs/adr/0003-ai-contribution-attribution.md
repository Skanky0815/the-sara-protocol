# ADR 0003: Attribution of AI contributions

- Date: 2026-10-27
- Status: Proposed
- Proposed by: Rico and Sara (AI assistant)
- Documented by: Sara (AI assistant)

## Context
We want to distinguish human-authored and AI-assisted changes, including bugs and successful contributions. Git author, committer, GitHub account and actual contributor are not necessarily identical.

## Proposed decision
- Mark AI-authored changes in commit messages with `AI-assisted-by: Sara (AI)` when applicable.
- For collaborative work, describe human review and AI contribution accurately in PR descriptions and ADRs.
- Keep Git author/committer identities truthful; do not pretend a GitHub connector commit was made by a separate Sara account.
- Consider a dedicated bot identity later if technically supported and approved.

## Alternatives
A separate GitHub bot account; Git `Co-authored-by` trailers with a verified identity; relying only on PR labels.

## Consequences
Clearer attribution without rewriting history. Commit trailers alone cannot guarantee line-by-line responsibility or reliable GitHub profile attribution.

## Open question
Confirm exact commit identity and trailer convention in local Codex and GitHub connector workflows before accepting this ADR.
