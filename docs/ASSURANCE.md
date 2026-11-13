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
| Elevated or system process | the text service DLL is loaded into an installer, `consent.exe` or the sign-in screen and trusts that process's environment (`%APPDATA%`) | R1, R4 ([#45](https://github.com/dtwinuz/bitg/issues/45)) |
| Clipboard history / sync | the `Ctrl+Shift+Y` hotkey copies the selection with the host application's own copy, which may reach Win+V history | R1 |
| Spoofed injection tag | a process in the same session forges bitg's `dwExtraInfo` tags to make bitg skip a key | R2 (bitg then leaves the word unchanged) |
| Poisoned word lists | a pull request changes a lexicon so that chosen words are converted wrongly | R6 (review of every change, accuracy sets in CI) |
| Distribution channels | the `bitg.uz` mirror, the apt key, Homebrew and winget manifests | R5 ([#42](https://github.com/dtwinuz/bitg/issues/42)) |
| Shared Linux machine | another local user reads `exceptions.txt` | R1 ([#43](https://github.com/dtwinuz/bitg/issues/43)) |

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
  They do record the process name and the length of each word in the fallback path; that is typing metadata,
  not text.
- The only persisted text is `exceptions.txt`. On Windows it is written only if `undo_remember=1` (default off);
  on Linux and macOS when the user undoes a replacement with Backspace.
- The fallback path keeps at most 40 characters of the current word in memory (`crates/bitg-tsf/src/typed.rs`).

### R2 — edits stay inside the word
- `Session::complete_word` returns an `Edit { delete, insert }` for the finished word only.
- The `session` fuzz target (`fuzz/fuzz_targets/session.rs`) asserts `delete <= word length` for arbitrary input;
  it runs weekly and on every pull request that touches the core.
- Windows: before replacing, the text service re-reads the text before the caret; if it no longer ends with the
  word, nothing is replaced (`word unchanged`, `undo: text changed` paths in `service.rs`).

### R3 — robustness against arbitrary input
- Memory safety: Rust everywhere except the thin IBus (C), Fcitx5 (C++) and macOS (Swift) shells; `unsafe` is
  limited to FFI and COM calls.
- The COM entry points of the Windows text service that do real work run inside `guard()`, which catches errors
  and panics and logs them instead of returning them to TSF (`service.rs`, issue #13). The trivial class-factory
  and focus callbacks and the `bitg-core` C ABI are not wrapped yet ([#44](https://github.com/dtwinuz/bitg/issues/44)).
- libFuzzer targets `convert` and `session` with a corpus from `eval/`; CodeQL (Rust, C/C++, Python,
  JavaScript, Actions) on every pull request; `cargo clippy -D warnings`.
- Office files: only text nodes are rewritten into a new archive; entry names are never used as file-system paths,
  so a crafted archive cannot write outside the output file. Only the text parts that are converted are
  decompressed, each capped at 256 MiB (`ENTRY_LIMIT` in `crates/bitg-cli/src/ooxml.rs`); a larger entry
  (zip bomb) stops the conversion with an error. Other entries are copied still compressed.

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
- Builds are reproducible: the same commit gives byte-identical `.deb`, `.msi` and `.pkg` files in any directory
  (`SOURCE_DATE_EPOCH` = commit time, path prefixes remapped, deterministic linker output, MSI product/package
  codes derived from the version and the payload, xar timestamps normalised by `tools/xar_normalize.py`). CI builds
  each package twice and compares SHA-256 (`linux`, `macos`, `e2e` workflows). Authenticode or Developer ID
  signatures, when used, add time stamps and are outside this guarantee.
- The MSI is not Authenticode-signed and the `.pkg` is ad-hoc signed; integrity comes from the attestations.

### R6 — infrastructure
- `main` is protected by a ruleset: pull request required, squash only, required CI checks, no force-push or
  deletion. Two maintainers from different employers ([GOVERNANCE.md](../GOVERNANCE.md)).
- Workflows run with `contents: read` by default; only the release job gets `id-token` / `attestations` write.
  Release checkouts do not keep the token in `.git/config`, and signing secrets are visible only to a step that does
  not compile code.
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
| Uncaught exception across FFI (CWE-248) | `guard()` at the working COM entry points; `catch_unwind` in the `bitg-ime` C ABI used by Linux and macOS; remaining entry points: [#44](https://github.com/dtwinuz/bitg/issues/44). |
| Uncontrolled search path (CWE-427) | `input.dll` is loaded from System32 only (`LOAD_LIBRARY_SEARCH_SYSTEM32`). |
| Injection (CWE-74/78/79) | No shell, SQL or HTML is built from typed text; the site is static and escapes generated text (`tools/site/og.cjs`). |
| Information exposure through logs (CWE-532) | Logs contain no typed text (R1). |
| Path traversal in archives (CWE-22) | Entry names are not used as paths (R3). |
| Download of code without integrity check (CWE-494) | HTTPS, SHA-256, signed provenance (R5). |

## 7. Known gaps

| # | Gap | Plan |
|---|---|---|
| G2 | Closed: decompressed Office entries are capped at 256 MiB (R3, [#27](https://github.com/dtwinuz/bitg/issues/27)). | — |
| G3 | The apt signing key and the web server are held by one maintainer. | Handover tracked in [GOVERNANCE.md](../GOVERNANCE.md#continuity). |
| G4 | Windows fallback path (apps without TSF text): a field whose type cannot be read is treated as plain text, and Backspace re-typing cannot re-read the text (R2, R4). | [#41](https://github.com/dtwinuz/bitg/issues/41) |
| G5 | `bitg.uz` serves files checked only against their `.sha256`, not the attestation. | [#42](https://github.com/dtwinuz/bitg/issues/42) |
| G6 | `bitg file` output and `exceptions.txt` writes: predictable temp name, permissions not kept. | [#43](https://github.com/dtwinuz/bitg/issues/43) |
| G7 | No CI gate on RustSec advisories. | [#47](https://github.com/dtwinuz/bitg/issues/47) |

## 8. Review

This assurance case is reviewed by both maintainers at every minor release and whenever a new platform or input
path is added. Last review: 2026-11-13 (v0.3.x).

### Security review 2026-11-13

- **Scope:** the code at `063778f` against R1–R6 and the trust boundaries above: Windows text service
  (`bitg-tsf`), the C ABI and the IBus, Fcitx5 and macOS shells, `bitg file` (OOXML), configuration parsing,
  workflows, installers (`Package.wxs`, `build-deb.sh`, `pkg-scripts`), `deploy/` and the bot.
- **Method:** manual code reading (`unsafe`, FFI/COM lengths and frees, panics across boundaries, file writes,
  log calls, `${{ }}` in `run:`), with `grep` and `cargo tree`. Not covered: running fuzzers or `cargo audit`,
  GitHub-side settings.
- **Result:** no critical or high findings; no path sends typed text off the device.

| ID | Severity | Finding | Status |
|---|---|---|---|
| F1 | medium | Windows fallback: unknown field type treated as non-sensitive | open, G4 / #41 |
| F2 | medium | release `msi` job compiled third-party code with the write token in `.git/config` and the signing secrets in the environment | fixed (`persist-credentials: false`, separate signing step) |
| F3 | medium | `bitg.uz` mirror not tied to the attestation | open, G5 / #42 |
| F4 | low | `bitg file` / `exceptions.txt` write safety | open, G6 / #43 |
| F5 | low | panics not caught at a few FFI/COM entry points; R3 wording overstated | wording fixed, code #44 |
| F6 | low | fallback word buffer grew without limit | fixed (capped at 40) |
| F7 | low | `LoadLibraryW("input.dll")` searched the application directory | fixed (System32 only) |
| F8 | low | `%APPDATA%` taken from the host's environment | open, #45 |
| F9 | low | tag name interpolated into release scripts | fixed (passed through `env`) |
| F10 | low | IBus copies the whole surrounding text on every key | open, #46 |
| F11 | info | no advisory gate in CI; `pip install` not hash-pinned | open, G7 / #47 |

Checked and found sound: VARIANT/BSTR clearing and `CoTaskMemFree` pairing, re-validation of the text before
every replacement on all platforms, `catch_unwind` in the `bitg-ime` C ABI, OOXML entry names never used as paths,
no entity expansion in `quick-xml`, read-only workflow defaults and SHA-pinned actions, MSI custom actions limited
to fixed `regsvr32` / `taskkill` commands, the bot container's restrictions.
