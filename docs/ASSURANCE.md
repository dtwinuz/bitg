# Security assurance case

This document argues why bitg meets its security requirements, and lists where it does not yet. It is the
"assurance case" of the OpenSSF Best Practices badge (silver `assurance_case`, gold `security_review`).
Architecture: [ARCHITECTURE.md](../ARCHITECTURE.md). Reporting vulnerabilities: [SECURITY.md](../SECURITY.md).

## 1. Security requirements

| # | Requirement |
|---|---|
| R1 | Text the user types is never sent off the device and never written to disk, except words the user explicitly makes exceptions (`exceptions.txt`). |
| R2 | bitg never changes text outside the word that was just finished. |
| R3 | Arbitrary input (any UTF-8, any length, any application behaviour) cannot crash the host application or execute code. |
| R4 | bitg does not act where it could corrupt or expose secrets: password, PIN and e-mail fields, terminals. |
| R5 | A user can verify that an installer was built by this repository's CI from a tagged commit. |
| R6 | The project's infrastructure (repository, CI, releases) cannot be changed by one compromised outside account. |

## 2. Threat model

| Actor / threat | Example | Requirement |
|---|---|---|
| Malicious or buggy application | sends huge or malformed text, changes the text while bitg edits it | R2, R3 |
| Malicious document | crafted `.docx` / `.xlsx` given to `bitg file` | R3 |
| Network attacker | intercepts a download | R5 |
| Supply-chain attacker | compromised dependency or GitHub Action, tampered release | R5, R6 |
| Local reader of logs | looks for typed text in `tsf.log` | R1 |

Out of scope: an attacker who already runs code as the user (they can read keystrokes without bitg) and a
compromised operating system.

## 3. Trust boundaries

1. **Application ↔ input method.** The Windows text service is a DLL inside every application where the user
   types; on Linux and macOS the engine runs in the input-method process. Text from the application is untrusted.
2. **Platform shell ↔ core (C ABI / COM).** Rust panics must not cross it.
3. **User files ↔ CLI.** Office documents are untrusted ZIP/XML.
4. **Repository ↔ CI ↔ release.** Only GitHub Actions builds releases.

## 4. Arguments

### R1 — no leaks of typed text
- No networking code in the input methods: `bitg-core` has no runtime dependencies; `bitg-ime` depends only on
  `bitg-core`; `bitg-tsf` adds only the `windows` crates (see the `Cargo.toml` files and the SBOM of each release).
- Logs record events, lengths and error codes, not text: `crates/bitg-tsf/src/service.rs` logs e.g.
  `fallback: word unchanged (len N)` and `… failed hr=0x…`; the edit session logs a static status string.
- The only persisted text is `exceptions.txt`, written when the user undoes a replacement with Backspace.

### R2 — edits stay inside the word
- `Session::complete_word` returns an `Edit { delete, insert }` for the finished word only.
- The `session` fuzz target (`fuzz/fuzz_targets/session.rs`) asserts `delete <= word length` for arbitrary input;
  it runs weekly and on every pull request that touches the core.
- Windows: before replacing, the text service re-reads the text before the caret; if it no longer ends with the
  word, nothing is replaced (`word unchanged`, `undo: text changed` paths in `service.rs`).

### R3 — robustness against arbitrary input
- Memory safety: Rust everywhere except the thin IBus (C), Fcitx5 (C++) and macOS (Swift) shells; `unsafe` is
  limited to FFI and COM calls.
- Every COM entry point of the Windows text service runs inside `guard()`, which catches errors and panics and
  logs them instead of returning them to TSF (`service.rs`, issue #13).
- libFuzzer targets `convert` and `session` with a corpus from `eval/`; CodeQL (Rust, C/C++, Python,
  JavaScript, Actions) on every pull request; `cargo clippy -D warnings`.
- Office files: only text nodes are rewritten into a new archive; entry names are never used as file-system paths,
  so a crafted archive cannot write outside the output file.

### R4 — no action in sensitive fields
- Linux: IBus (`linux/ibus/engine.c`) turns bitg off for password and PIN input purposes and Fcitx5
  (`linux/fcitx5/bitg.cpp`) for `Password` / `Sensitive` fields; covered by `linux/test/ibus_e2e.py`
  (`password_field_untouched`).
- macOS: the system disables input methods while secure text entry is active.
- Windows: terminal processes (`windowsterminal.exe`, `conhost.exe`, …) are disabled by default
  (`crates/bitg-tsf/src/config.rs`). The text service reads the field's input scope (`GUID_PROP_INPUTSCOPE`)
  and the keyboard-disabled compartment and stays off for password, PIN and e-mail fields
  (`crates/bitg-tsf/src/plan.rs` `SENSITIVE_INPUT_SCOPES`); covered by `tools/e2e/tsf-e2e.ps1`
  (`sensitive: email field untouched`).

### R5 — verifiable releases
- Releases are built only by `.github/workflows/release.yml` from a `v*` tag.
- From v0.3.1 each file has a SHA-256 file, SLSA build provenance and an SBOM attestation signed through Sigstore
  by GitHub Actions (`gh attestation verify <file> -R dtwinuz/bitg`); the bundles are attached to the release for
  offline verification. The apt repository is signed with a dedicated OpenPGP key (`deploy/apt/bitg.asc`).
- Downloads are offered only over HTTPS (GitHub Releases, `bitg.uz` with HSTS).

### R6 — infrastructure
- `main` is protected by a ruleset: pull request required, squash only, required CI checks, no force-push or
  deletion. Two maintainers from different employers ([GOVERNANCE.md](../GOVERNANCE.md)).
- Workflows run with `contents: read` by default; only the release job gets `id-token` / `attestations` write.
- Every GitHub Action and Docker base image is pinned by commit SHA or digest; Dependabot updates them monthly.
- Secret scanning and push protection are enabled; OpenSSF Scorecard runs weekly.

## 5. Secure design principles applied

| Principle | How |
|---|---|
| Economy of mechanism | One small core; platform shells only translate events. |
| Fail-safe defaults | When unsure (ambiguous word, text changed under us), bitg leaves the text as typed. |
| Least privilege | No keyboard hook, no network, no admin rights at runtime; CI tokens are read-only by default. |
| Complete mediation | Every edit goes through `Session` and is checked against the current text first. |
| Open design | All code and the accuracy sets are public. |
| Psychological acceptability | Backspace undoes a replacement and remembers the exception. |

## 6. Common weaknesses

| Weakness (CWE) | Status |
|---|---|
| Memory corruption (CWE-119/787/416) | Rust; the C/C++ shells are small and covered by CodeQL. |
| Uncaught exception across FFI (CWE-248) | `guard()` at every COM entry point; `catch_unwind` in the `bitg-ime` C ABI used by Linux and macOS. |
| Injection (CWE-74/78/79) | No shell, SQL or HTML is built from typed text; the site is static and escapes generated text (`tools/site/og.cjs`). |
| Information exposure through logs (CWE-532) | Logs contain no typed text (R1). |
| Path traversal in archives (CWE-22) | Entry names are not used as paths (R3). |
| Download of code without integrity check (CWE-494) | HTTPS, SHA-256, signed provenance (R5). |

## 7. Known gaps

| # | Gap | Plan |
|---|---|---|
| G2 | `bitg file` has no size limit for decompressed Office entries (a "zip bomb" can use a lot of memory). Only affects files the user chooses to convert. | Cap the decompressed size per entry ([#27](https://github.com/dtwinuz/bitg/issues/27)). |
| G3 | The apt signing key and the web server are held by one maintainer. | Handover tracked in [GOVERNANCE.md](../GOVERNANCE.md#continuity). |

## 8. Review

This assurance case is reviewed by both maintainers at every minor release and whenever a new platform or input
path is added. Last review: 2026-11-12 (v0.3.x).
