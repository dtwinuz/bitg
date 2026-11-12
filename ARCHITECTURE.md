# Architecture

bitg converts Uzbek text to the 2026 Latin alphabet (`o'` → `ö`, `g'` → `ğ`, `sh` → `ş`, `ch` → `ç`) while the user
types, and leaves English and foreign names alone. One Rust core does the conversion; thin platform layers connect
it to each operating system's input-method framework.

## Components

```mermaid
flowchart TB
  subgraph core["bitg-core (Rust, no runtime dependencies)"]
    tok["token.rs<br/>split text into words"] --> lang["lang.rs + lexicon.rs<br/>is this word Uzbek?"]
    lang --> latin["latin.rs / cyrillic.rs<br/>alphabet rules"]
    sess["Session<br/>typing state, one word at a time"]
  end
  cli["bitg-cli<br/>convert, explain, eval, files"] --> core
  wasm["wasm.rs<br/>bitg.uz playground"] --> core
  ime["bitg-ime<br/>platform-independent typing logic"] --> sess
  tsf["bitg-tsf<br/>Windows TSF text service (DLL)"] --> sess
  settings["bitg-settings<br/>Windows tray + settings (WebView2)"] --> core
  ime -- "C ABI" --> ibus["linux/ibus (C)"]
  ime -- "C ABI" --> fcitx["linux/fcitx5 (C++)"]
  ime -- "C ABI" --> mac["macos/ (Swift, InputMethodKit)"]
```

| Path | Role |
|---|---|
| `crates/bitg-core` | Conversion: tokenizer, language evidence (Uzbek lexicon from corpora, SCOWL English list, brand names, foreign names), alphabet rules, reverse and Cyrillic conversion, `Session` for typing, C ABI (`ffi.rs`) and WebAssembly ABI (`wasm.rs`). The lexicons are compiled into the binary by `build.rs`; nothing is loaded at runtime. |
| `crates/bitg-ime` | Typing logic shared by Linux and macOS: word boundaries, replace-in-place when the application exposes surrounding text, preedit when it does not, Backspace undo and learned exceptions, `settings.ini` / `exceptions.txt`. Exposed as a C ABI. |
| `crates/bitg-tsf` | Windows Text Services Framework text input processor. It watches text changes (`ITfTextEditSink`) and replaces the finished word through a TSF edit session. For applications that do not expose text (Qt, e.g. Telegram) it rebuilds the word from key events and sends Backspace + the new word with `SendInput`. |
| `crates/bitg-settings` | Windows tray icon and settings window; also the "convert selected text" hotkey (`Ctrl+Shift+Y`). |
| `crates/bitg-cli` | Command-line tool and the accuracy harness (`bitg eval eval/uz.tsv`). |
| `linux/`, `macos/` | Thin platform shells over `bitg-ime`, packaging (`.deb`, `.pkg`) and end-to-end tests. |
| `installer/` | WiX MSI for Windows. |
| `web/`, `tools/site/` | The bitg.uz static site and its build; the playground runs the real core compiled to WebAssembly. |
| `fuzz/` | libFuzzer targets for the core (`convert`, `session`). |
| `eval/` | Accuracy sets: 3 000 Uzbek and 3 000 English sentences with expected output. |

## Data flow while typing

```mermaid
sequenceDiagram
  participant App as Application
  participant OS as OS input framework (TSF / IBus / IMK)
  participant Shell as Platform shell
  participant Core as bitg-core Session
  App->>OS: user types "o'zbek" + space
  OS->>Shell: key / text-change event
  Shell->>Core: complete_word("o'zbek", options)
  Core-->>Shell: Edit { delete: 6, insert: "özbek" }
  Shell->>OS: replace the word (edit session / commit text)
  OS->>App: "özbek "
```

`Session::complete_word` returns an `Edit` that only touches the word just finished: `delete` is never larger than
the word (checked by the `session` fuzz target). Backspace right after a replacement restores the original and
remembers it as an exception.

## Trust boundaries

- **Input:** the only input is text the user types (and, for files, documents the user opens with the CLI). It is
  untrusted and arbitrary; the core handles any UTF-8 and never panics across an FFI or COM boundary (panics are
  caught at those boundaries and logged).
- **No network:** the input methods and the core never open network connections. The website is static.
- **Privileged surface:** the Windows text service is a DLL loaded into every application where the user types.
  It runs with the application's own rights; it is disabled in password, PIN and e-mail fields and in terminals.
  It uses no global keyboard hook.
- **Local files:** `settings.ini` and `exceptions.txt` in the user's profile; logs only record events and error
  codes, never typed text.

More detail on threats and how they are handled: [docs/ASSURANCE.md](docs/ASSURANCE.md).

## Build and release

`cargo build --workspace` builds everything; platform packages are built by `installer/build.ps1`,
`linux/build-deb.sh` and `macos/build.sh`. Releases are built only by GitHub Actions from a `v*` tag and carry
SHA-256 files, SLSA build provenance and an SBOM.
