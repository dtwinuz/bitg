# Roadmap

What bitg plans for the next year. Dates are targets, not promises; the issue tracker has the details. The goal
of v1.0 is to be ready before the new alphabet becomes mandatory (April 2027).

## Done

- **v0.1.0** — `bitg-core` (2026 alphabet rules, English protection, Cyrillic → new Latin, reverse conversion),
  the `bitg` CLI, the Windows TSF input method with a settings app and an MSI installer.
- **v0.2.0** — Linux: IBus engine in a `.deb`, Fcitx5 add-on (testing), end-to-end tests with a real `ibus-daemon`.
- **v0.3.0** — macOS beta (InputMethodKit, universal `.pkg`), ambiguous words are no longer guessed (#5), foreign
  names keep their spelling (#6), bitg.uz with the WebAssembly playground, a signed apt repository, Homebrew cask.
- **v0.3.1** — Windows keeps bitg selected when activation fails (#13) and stays off in password, PIN and e-mail
  fields (#26); every release ships SLSA build provenance and an SBOM (`gh attestation verify`); reproducible `.deb`.
- **v0.4.0** — personal dictionary, in-app feedback, verified one-click update (Windows), What's new page, Ubuntu
  panel indicator, 3 419 foreign names.

## Next: v0.4.x (November–December 2026)

- winget package `dtwinuz.bitg` accepted in microsoft/winget-pkgs, and the Chocolatey package through community
  moderation.
- AUR packages once new-account registration reopens.
- Listing in the ChatGPT app directory.
- Fedora/openSUSE packages (COPR/OBS) and a nixpkgs submission.

## v0.5 (Q1 2027)

- Signed Windows installer (code-signing certificate) and a Microsoft Store listing.
- Fcitx5 tested in real applications; Fedora/openSUSE packages published (spec in `packaging/rpm`), Nix in nixpkgs;
  Flatpak.
- macOS: tested in Safari, Chrome, Notes, Word and Telegram; Developer ID signing and notarization.
- A second maintainer with release access (apt key, server handover).

## v1.0 (before April 2027)

- Accuracy on `eval/uz.tsv` above 99 % while `eval/en.tsv` stays above 99.7 %.
- Stable settings format and C ABI (`bitg-core`, `bitg-ime`) with a compatibility promise.
- Long-term support: security fixes for the last minor release for at least 12 months.

## Later

- iOS keyboard extension (the core as a `staticlib`) and Android keyboard.
- Word add-in; the browser extension in Edge Add-ons and addons.mozilla.org (Chrome Web Store: in review).
- An API for organisations converting large document archives.

## How to influence the roadmap

Open an issue or comment on an existing one. Changes to this roadmap are made by pull request
([GOVERNANCE.md](GOVERNANCE.md)).
