# Instructions for the coding agent (Hermes)

You are implementing [PLAN.md](PLAN.md) in this repository. Nothing else.

## What Zephyr is

A Windows system tray weather app in Rust. Temperature drawn in the tray icon, details in the tooltip, settings in `%APPDATA%\Zephyr\config.toml`. It must stay tiny: one EXE around 2 MB, about 20 MB of RAM, no window except the existing city picker.

Layout (do not change it):

| File | Role |
| --- | --- |
| `src/main.rs` | Tray, menu, event loop, background fetcher loop, notifications |
| `src/weather.rs` | HTTP calls, JSON parsing, tooltip text, unit tests |
| `src/config.rs` | `Config` / `City`, load and save TOML |
| `src/icon.rs` | Draws the temperature icon pixel by pixel |
| `src/autostart.rs` | "Start with Windows" registry entry |
| `src/city_dialog.rs` | The only window: city search |
| `build.rs` | EXE icon, manifest and version info |

## How to work

1. Do the steps of PLAN.md **in order, one step at a time**. Never combine steps.
2. Before each step, read only the files that step lists.
3. After each step: `cargo test` and `cargo build --release` must pass, then make **one commit** for that step, then **stop** and report:
   - what changed (files and a one-line summary each),
   - test results,
   - how to undo it (`git revert <hash>`).
4. Wait for Ali to say "continue" before the next step.
5. If a step cannot be done as written, or you find that the plan is wrong, stop and explain. Do not invent a different solution.

## Hard rules

- **Do not leave the plan.** Anything you notice that is outside it (a bug, an idea, a cleanup) goes into your report. You do not fix it.
- **Do not touch the core structure:** no moving, renaming, splitting or merging modules; no redesign of the event loop or the fetcher loop in `main.rs`. Adding calls inside the existing fetcher loop and new `UserEvent` variants is allowed when a step says so.
- **No new dependencies.** Do not edit `[dependencies]` in `Cargo.toml`. Everything needed (`reqwest`, `serde`, `serde_json`, `chrono`, `winreg`, `notify-rust`, `native-windows-gui`, `tokio`) is already there.
- **No new windows or UI** beyond what a step names (only checkable tray menu items).
- Do not change the 30-minute refresh interval, the retry timing, the tooltip limit logic, or `icon.rs`.
- Do not edit `Zephyr.ico`, `dist/`, or `Cargo.lock` by hand (`cargo build` may update `Cargo.lock` only in Step 7).
- Never delete user data: old `config.toml` files must keep loading (new fields get defaults).
- **No outward actions without Ali's explicit approval:** no `git push`, no tags, no GitHub releases, no PRs to other repositories, no winget submission, no SignPath application.

## Code style

- Match the surrounding code: small functions, `Result<_, String>` errors, `let _ =` for best-effort calls, `#[serde(default)]` for config.
- Comments only for the "why", one line, in English like the existing ones.
- Every new parser gets a unit test with a small JSON fixture in the same file's `mod tests`.
- UI text stays in English. Numbers always use Western digits (0-9); never format with an Arabic locale.
- Publisher everywhere is **Ali Alhumedi**.

## Commands

```bash
cargo test
```

```bash
cargo build --release
```

Keep reports short. Filter long command output (for example only the `test result` lines and errors).

Commit messages: short imperative summary, e.g. `Add air quality alert (AQI >= 101)`.
