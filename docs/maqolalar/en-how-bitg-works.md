---
title: "Rewriting a country's alphabet as you type: how bitg works"
published: false
description: "Uzbekistan changed four letters in October 2026. bitg converts text to the new alphabet in every app, without touching English, in about 2 µs per word."
tags: rust, opensource, i18n, windows
canonical_url: https://github.com/dtwinuz/bitg/blob/main/docs/maqolalar/en-how-bitg-works.md
---

<!-- Draft for dev.to / Hashnode / Medium. Every number below comes from README.md ("Sifat") or ARCHITECTURE.md;
     re-run `bitg eval` before publishing and update the figures and the date. -->

On 21 October 2026 Uzbekistan adopted law O'RQ-1184. It replaces four letters of the Latin alphabet:
**Oʻ → Ö, Gʻ → Ğ, Sh → Ş, Ch → Ç**. The law takes effect in April 2027. Keyboards do not have these letters, and
most people who write Uzbek also write English every day.

[bitg](https://github.com/dtwinuz/bitg) is an open-source input method that does the conversion while you type,
in any application: you keep typing `o'zbek shahar`, and when you press space the word becomes `özbek şahar`.

```text
$ bitg "Please check the schedule, keyin choy ichamiz"
Please check the schedule, keyin çoy içamiz
```

This post covers the three problems that made it harder than a find-and-replace.

## 1. `sh` is not always `ş`

The naive rule (`sh → ş`, `ch → ç`) breaks every English word: *check*, *shell*, *push*. bitg only changes a word
when there is **evidence that it is Uzbek**:

- an Uzbek lexicon built from corpora (Leipzig Corpora Collection, Mozilla Common Voice, Tatoeba) with
  suffix-aware stemming, so inflected forms like *shaharlarimizdan* are recognised;
- the SCOWL English word list, brand names and a list of foreign names that keep their spelling;
- the shape of the token: URLs, e-mail addresses, `@user`, `#tag`, `snake_case`, `camelCase`, `push()` and
  password-like strings such as `shahar123` are never touched;
- sentence context: in `Menga chat yozing`, *chat* is English; in `Menga choy yozing`, *choy* becomes *çoy*.

The lexicons are compiled into the binary by `build.rs`, so loading takes 0 ms and the core has no runtime
dependencies. Words that are genuinely ambiguous without apostrophes (`ochmaydi`, `otdim`) are left as typed rather
than guessed.

## 2. Twelve apostrophes

Uzbek text in the wild uses twelve different characters for the old `ʻ`, among them `'`, `’`, `‘`, `ʻ`, `ʼ` and
`` ` ``. bitg treats them as one, and then has to tell an apostrophe from a quotation mark: in `'Yaxshi tong'` the
marks are quotes, in `tog'` it is part of the letter `ğ`, and in `Is'hoq` it separates `s` from `h` and disappears
(`Ishoq`).

## 3. Every operating system types differently

One Rust core does the conversion; each platform gets a thin shell:

| Platform | How the word is replaced |
|---|---|
| Windows | A Text Services Framework text service written in Rust. It does not hook the keyboard: it watches text changes (`ITfTextEditSink`) and replaces the finished word in a TSF edit session. Qt apps such as Telegram Desktop do not expose their text, so there bitg rebuilds the word from key events and sends Backspace + the new word. |
| Linux | IBus engine (C) and Fcitx5 add-on (C++) over a shared C ABI (`bitg-ime`): replace in place when the app exposes surrounding text, preedit when it does not. |
| macOS | InputMethodKit (Swift) over the same C ABI. |
| Web | The same core compiled to WebAssembly runs the [bitg.uz](https://bitg.uz) playground. |

The contract between core and shell is small. `Session::complete_word` returns an `Edit { delete, insert }` that
only touches the word just finished; a fuzz target checks that `delete` is never larger than that word. Backspace
right after a replacement restores the original and remembers it as an exception.

In password, PIN and e-mail fields and in terminals bitg stays off.

## Measured, not guessed

`bitg eval` runs the core over fixed sets that are excluded from building the lexicon (in the repo under `eval/`,
CC BY 4.0):

| Set | Result |
|---|---|
| 3 000 Uzbek sentences, old Latin → new Latin, mixed apostrophes | **98.87 %** |
| 3 000 Uzbek sentences written without apostrophes (`ozbek`, `togri`), whole sentence correct | **81.20 %** |
| 3 000 English sentences containing sh/ch/apostrophes, must stay unchanged | **99.73 %** |
| Speed (release, x64) | 2.7 µs/word Uzbek, 1.4 µs/word English, 1.1 MB memory |

Measured on 2026-11-12. Rule changes are judged by these numbers, not by examples.

## Trust

An input method is loaded into every application where you type, so it has to earn trust:

- it never opens a network connection, and logs record events and error codes, never typed text;
- releases are built only by GitHub Actions from a tag, with SHA-256 files, SLSA build provenance and an SBOM
  (`gh attestation verify`);
- the code is MIT OR Apache-2.0, REUSE-compliant, fuzzed, and tracked by OpenSSF Scorecard and Best Practices.

## Try it

- Windows 10/11: MSI from [Releases](https://github.com/dtwinuz/bitg/releases) (winget package under review)
- Ubuntu/Debian: `.deb` or the signed apt repository; Arch: PKGBUILD in `packaging/aur`
- macOS (beta): `brew install --cask dtwinuz/tap/bitg`
- In the browser: [bitg.uz](https://bitg.uz)

Wrong conversion? Open an [issue](https://github.com/dtwinuz/bitg/issues/new/choose) with the word and the
sentence it appeared in.
