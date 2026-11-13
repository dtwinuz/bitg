# Governance

bitg is an open-source project (MIT OR Apache-2.0) hosted by the [`dtwinuz`](https://github.com/dtwinuz) GitHub
organization. This document says who decides what and how the project keeps going if someone steps away.

## Roles

| Role | Who | Can do |
|---|---|---|
| **Maintainer** | [@XakimovNosirbek](https://github.com/XakimovNosirbek) (lead), [@leverow](https://github.com/leverow) | Review and merge pull requests, triage issues, cut releases, handle security reports, change repository settings. Both are owners of the `dtwinuz` organization. |
| **Reviewer** | [@wonmors](https://github.com/wonmors) | Review pull requests; owner of the `dtwinuz` organization. Becomes a maintainer through the process below. |
| **Contributor** | anyone | Open issues and pull requests, review others' pull requests, suggest words for the lexicon. |
| **Reporter** | anyone | Report bugs and wrong conversions (issue templates, [bitg.uz/fikr](https://bitg.uz/fikr/), Telegram [@bitguz](https://t.me/bitguz)) and vulnerabilities ([SECURITY.md](SECURITY.md)). |

The two maintainers work for different employers; neither is paid by the other.

### Responsibilities

- **Lead maintainer:** release planning ([ROADMAP.md](ROADMAP.md)), the conversion rules in `bitg-core`, the
  Windows and macOS input methods, the website and package repositories.
- **Maintainer:** code review, the Linux input method, CI and supply-chain checks, security triage.
- **Every maintainer:** answers new issues within 14 days, acknowledges security reports within 7 days
  ([SECURITY.md](SECURITY.md)), and keeps 2FA enabled on their GitHub account.

## Decisions

- Day-to-day changes are decided in the pull request: a change is merged when CI is green and the review
  questions are resolved. Merges are squash-only and go through the `main` ruleset (no direct pushes, no
  force-push, no branch deletion).
- Every change is reviewed and approved by a maintainer or reviewer other than its author before it is merged;
  the reviewer reads the diff and leaves at least one comment or question when something is unclear. Exceptions: urgent security fixes and
  documentation-only changes, which are reviewed after the merge.
- Bigger decisions (a new platform, a change of the conversion rules, dropping a supported OS, license questions)
  are discussed in a GitHub issue and need the agreement of both maintainers. If they disagree, the lead
  maintainer decides and records the reason in the issue.
- Conversion rules follow the official 2026 Uzbek Latin alphabet. Where the law leaves a choice, accuracy on the
  `eval/` sets decides: a rule change must not make `eval/uz.tsv` or `eval/en.tsv` worse.

## Becoming a maintainer

A contributor who has made sustained, good-quality contributions (code or reviews over at least three months) can
be invited by an existing maintainer; both maintainers must agree. Maintainers who are inactive for 12 months
move to an "emeritus" list in this file.

## Continuity

The project must be able to continue if any one person disappears.

| Asset | Where it lives | Who can use it |
|---|---|---|
| Repository, settings, rulesets | GitHub `dtwinuz/bitg` | both maintainers (organization owners) |
| Release build and publishing | GitHub Actions (`.github/workflows/release.yml`, started by a `v*` tag) — no personal machine needed | both maintainers |
| Release secrets (code-signing certificate when present) | GitHub Actions secrets | both maintainers can rotate them |
| apt repository signing key | encrypted offline backup | lead maintainer; handover to @leverow is pending |
| `bitg.uz` web server and DNS | server configuration in `deploy/` (Caddy, deploy scripts) | lead maintainer; handover to @leverow is pending |

Step-by-step handover guide (review, merge, release, packages, website): [docs/MAINTAINING.md](docs/MAINTAINING.md).

Anything marked "pending" is a known gap: until it is closed, a new apt key or server can be set up from `deploy/`
and the documented scripts, at the cost of users re-importing the apt key.

## Code of conduct

Everyone in the project follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Changes to this document

Changes to this file are made by pull request and need the approval of both maintainers.
