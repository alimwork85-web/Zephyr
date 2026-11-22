# HoverSky 0.4.0 plan: trust the number

The 0.3.0 plan is done and lives in git history. This plan makes the number in the tray and the alerts trustworthy, plus one small new alert.

**Ali's instruction for this plan:** do all five steps in one run, without stopping between steps. Each step still gets its own commit, and `cargo test` plus `cargo build --release` must pass before each commit. Send one report at the end. Everything in AGENTS.md "Hard rules" still applies (no new dependencies, no push, no tags, no releases, `icon.rs` untouched).

---

## Step 1. Alert memory survives a restart

**Files:** `src/config.rs`, `src/main.rs`

Today `air_alerted_on`, `seen_alerts` and `rain_alerted` live only in memory (`main.rs` before `event_loop.run`), so restarting the app repeats the same alert.

- Add a `State` struct in `config.rs` (serde, every field `#[serde(default)]`): `air_alerted_on: Option<NaiveDate>` (or a `String` date if simpler), `seen_alerts: Vec<String>`, `rain_alerted: bool`.
- File: `state.toml` in the same folder as `config.toml` (reuse the folder from `config_path`). Separate file so `config.toml` is never rewritten by alerts.
- `State::load()` returns default on a missing or unreadable/garbage file. `State::save()` is best-effort (`let _ =`).
- In `main.rs`, initialise the three variables from `State::load()` and call save only where they change (after a notification is sent, when the rain episode ends, on city change). No save on every fetch.
- Keep `seen_alerts` bounded: when saving, keep only the last 50 ids.
- Tests in `config.rs`: round trip, garbage file gives default, file missing a field gives default for that field.

## Step 2. Fresh number after sleep

**Files:** `src/main.rs` (fetcher loop only)

The fetcher waits with `tokio::time::sleep(wait)`, a monotonic timer. After the laptop sleeps for hours it may keep showing the old temperature until the timer finishes.

- Before the `tokio::select!`, compute a wall-clock deadline: `chrono::Local::now() + wait`.
- Add one branch to the existing `select!`: an async block that loops `sleep(60s)` and breaks once `Local::now() >= deadline`. The other two branches stay as they are.
- Do not change `REFRESH_EVERY`, `retry_delay` or the loop structure. The tooltip already shows "Upd HH:MM", so no tooltip change.
- Add a one-line "why" comment.

## Step 3. Left click opens the forecast

**Files:** `src/main.rs`

Left-clicking the tray icon does nothing now, which feels broken.

- Register a `TrayIconEvent` handler (check the `tray-icon` 0.14 API in `~/.cargo/registry`), forward a left-button click (one per click, on button up if the API exposes it) as a new `UserEvent::TrayClick`.
- On `TrayClick`, open `https://www.windy.com/?{lat:.3},{lon:.3},10` with the current city, the same way the about item opens `PROJECT_URL` (`explorer`). Use `.` as decimal separator (plain `format!`).
- The right-click menu must keep working. If a left click also opens the menu, turn that off with the builder option if 0.14 has one; if it doesn't, stop and report it.

## Step 4. Frost and heat alert

**Files:** `src/weather.rs`, `src/config.rs`, `src/main.rs`

- `config.rs`: new field `extreme_alerts: bool`, default **false** (old config files keep loading, test it like `old_config_without_air_alerts_gets_them_on`).
- `weather.rs`: pure function `extreme_tomorrow(w: &Weather, celsius: bool) -> Option<&'static str>`: returns `Some("Frost")` if tomorrow's low is at or below 0 °C (32 °F when not celsius), `Some("Heat")` if tomorrow's high is at or above 40 °C (104 °F), else `None`. Unit tests for both units and both edges.
- `main.rs`: new `CheckMenuItem "Frost / heat alerts"` next to the other alert switches. On a successful weather update, if enabled and `extreme_tomorrow` is `Some` and not already alerted today, send one notification, e.g. "Frost tomorrow in Riyadh: low -2°C". Store the day in `State` (Step 1) as `extreme_alerted_on`, so it survives restarts.

## Step 5. Docs and version

**Files:** `Cargo.toml` (`version` only), `README.md`, `CHANGELOG.md`, `dist/`, new `SIGNPATH.md`

- Version 0.4.0. `cargo build` may update `Cargo.lock` for the version bump only.
- CHANGELOG 0.4.0 entry; README features updated in both the English and the Arabic sections.
- `dist/`: add the new EXE and `.sha256`, delete the previous ones, same naming as now.
- `SIGNPATH.md`: a ready-to-paste application text for SignPath Foundation (project, URL, license MIT, maintainer Ali Alhumedi, build via GitHub Actions). Do NOT submit it.

---

## Out of scope (noted, not to be done)

- Popup window, multiple cities, self-update, IP location, hourly chart, Arabic UI.
