# Zephyr 0.3.0 plan: smarter alerts, still no window

Zephyr stays a tiny tray app: no new windows, no new dependencies, about 20 MB of RAM. Every new feature is a Windows notification plus, at most, one checkable tray menu item. Read [AGENTS.md](AGENTS.md) before starting.

Baseline at the time of writing: commit `c605410`, version 0.2.3, `cargo test` = 11 passed.

Every step below is one commit. After each step: `cargo test` passes, `cargo build --release` passes, commit, stop and report.

---

## Step 0. Baseline check

- Run `cargo test` and `cargo build --release`. Both must pass before any change.
- If either fails, stop and report. Do not fix it.

## Step 1. Publisher name

**Files:** `build.rs`, `Cargo.toml`, `LICENSE`, `README.md`

- `build.rs`: add `res.set("CompanyName", "Ali Alhumedi")`; change `LegalCopyright` from `"MIT License"` to `"© 2026 Ali Alhumedi"`. Keep `ProductName` and `FileDescription`.
- `Cargo.toml`: add `authors = ["Ali Alhumedi"]` under `[package]`.
- `LICENSE`: change the copyright line to `Copyright (c) 2025-2026 Ali Alhumedi`. Rest of the MIT text unchanged.
- `README.md`: in the License section add `Developed by Ali Alhumedi.`

**Done when:** the built EXE's Properties → Details shows Company and Copyright as Ali Alhumedi.

## Step 2. Notifications show "Zephyr", not "Windows PowerShell"

**Why:** `notify-rust` on Windows falls back to `Toast::POWERSHELL_APP_ID` when no `app_id` is set (`notify-rust-4.18.x/src/windows.rs:79`), so the sender reads "Windows PowerShell".

**Files:** `src/autostart.rs` (or a new tiny `src/aumid.rs` in the same style), `src/main.rs`

- At startup, write `HKCU\Software\Classes\AppUserModelId\Zephyr.WeatherTray` with string value `DisplayName = "Zephyr"`, using `winreg` exactly the way `autostart.rs` does. No admin rights needed. No `IconUri` (no new files).
- In `notify_rain` (and every later notification) call `.app_id("Zephyr.WeatherTray")` **only if the registration succeeded**; otherwise keep the default so notifications never disappear.
- Put the id in one `const` in `main.rs`.

**Done when:** a test notification (temporarily trigger it locally, do not commit the trigger) shows "Zephyr" as the sender.

## Step 3. City window no longer freezes while searching

**Why:** `src/city_dialog.rs:51` runs `self.rt.block_on(weather::search_cities(..))` on the UI thread, so the window hangs for up to 15 seconds on a slow network.

**Files:** `src/city_dialog.rs` only

- Start the search with `self.rt.spawn(...)`, put the result in a shared slot (`Arc<Mutex<Option<Result<Vec<City>, String>>>>`), and wake the UI thread with an `nwg::Notice` (feature already enabled via nwg's default `all`).
- Handle `OnNotice` in the existing event handler: read the slot and run the same `match` that exists today (empty / list / error messages unchanged).
- Keep the Search button disabled while a search is running. Layout, sizes and texts unchanged.

**Done when:** while searching, the window can be moved and closed; results and error messages are the same as before.

## Step 4. "Rain within the next hour" alert (replaces the daily rain alert)

**Files:** `src/weather.rs`, `src/main.rs`

- `fetch_weather`: add `("minutely_15", "precipitation")` and `("forecast_minutely_15", "8")` to the existing query. Same single request, no extra network call.
- Response structs: add `minutely_15: Option<Minutely15>` with `time: Vec<String>` and `precipitation: Vec<Option<f64>>`; add `time: String` to `CurrentWeather` (Open-Meteo already returns it, local time because of `timezone=auto`).
- `Weather`: new field `rain_starts: Option<String>` = `HH:MM` of the first slot whose time is within 60 minutes after `current_weather.time` and whose precipitation is ≥ 0.1 mm. Compare as strings parsed with `chrono::NaiveDateTime` (format `%Y-%m-%dT%H:%M`). Missing data → `None`.
- Why 60 minutes is enough without a timer: data refreshes every 30 minutes, so rain is always seen at least ~30 minutes ahead.
- `main.rs`: replace the daily rule (`today_rain_prob >= RAIN_ALERT_THRESHOLD && alerted_on != Some(today)`) with: if `cfg.rain_alerts` and `w.rain_starts` is `Some(t)` and no alert was sent for the current rain episode → notify "Rain expected around {t} in {city}". The episode resets on the first update where `rain_starts` is `None`, and on city change.
- Keep `RAIN_ALERT_THRESHOLD` for the icon's blue strip (`show()` uses it). Keep the "Rain alerts" menu item as the switch.
- Update `tooltip_never_exceeds_limit` and any struct literal for the new field.

**Tests (in `weather.rs`):** fixture with rain 30 minutes after current time → `Some("HH:MM")`; fixture with rain 90 minutes ahead → `None`; fixture without `minutely_15` → `None`.

## Step 5. Air quality alert (only when it is bad)

**Files:** `src/weather.rs`, `src/config.rs`, `src/main.rs`

- `weather.rs`: `const AIR_URL = "https://air-quality-api.open-meteo.com/v1/air-quality"`; `pub async fn fetch_aqi(client, city) -> Result<u32, String>` with `hourly=us_aqi`, `forecast_days=1`, `timezone=auto`; returns the day's maximum (nulls ignored). Separate `parse_aqi(body)` for tests.
- Fetch **once per day**: inside the existing fetcher loop, after a successful weather fetch, if the last AQI date is not today, call `fetch_aqi` and send a new `UserEvent::AirQuality(u32)`. Errors are ignored and retried on the next loop pass. Do not change the loop's timing.
- `main.rs`: on `AirQuality(aqi)`, if `cfg.air_alerts` and `aqi >= 101` and not alerted today → one notification: "Air quality in {city}: {category} (AQI {aqi})". Categories: 101-150 Unhealthy for Sensitive Groups, 151-200 Unhealthy, 201-300 Very Unhealthy, 301+ Hazardous.
- `config.rs`: `pub air_alerts: bool`, default `true` (the struct already has `#[serde(default)]`, so old config files keep loading). Menu: `CheckMenuItem` "Air quality alerts" right after "Rain alerts", toggled the same way.

**Tests:** `parse_aqi` max with nulls; category boundaries 100/101/150/151/300/301; config without `air_alerts` loads as `true`.

## Step 6. Official US severe weather warnings (NWS)

**Files:** `src/weather.rs`, `src/config.rs`, `src/main.rs`

- `weather.rs`: `pub async fn fetch_nws_alerts(client, city) -> Result<Vec<NwsAlert>, String>` calling `https://api.weather.gov/alerts/active?point={lat:.4},{lon:.4}` with header `Accept: application/geo+json`. `NwsAlert { id, event, headline }` from `features[].properties`. Keep only `severity` = `Severe` or `Extreme`. Separate `parse_nws(body)` for tests.
- `http_client()`: User-Agent becomes `Zephyr/<version> (+https://github.com/alimwork85-web/Zephyr)` (NWS asks for contact info).
- Called in the existing fetcher loop after each successful weather fetch (every 30 minutes); result sent as `UserEvent::Alerts(Vec<NwsAlert>)`. Any error (including 404 outside the US) is ignored silently.
- `main.rs`: keep a `HashSet<String>` of seen ids (cleared on city change); notify each new id once: title = `event`, body = `headline`.
- `config.rs`: `pub severe_alerts: bool`, default `true`; menu `CheckMenuItem` "Severe weather alerts" after "Air quality alerts".

**Tests:** `parse_nws` on a small GeoJSON fixture (one Severe, one Moderate → 1 alert); empty `features` → empty vec; garbage → `Err`.

## Step 7. Docs and version

**Files:** `README.md`, `CHANGELOG.md`, `Cargo.toml`

- Version `0.3.0` in `Cargo.toml` (let `cargo build` update `Cargo.lock`).
- `CHANGELOG.md`: a `## 0.3.0` section (Added / Changed / Fixed) covering steps 1-6.
- `README.md`: features list (rain in the next hour, air quality, NWS warnings, US only for NWS), Usage table rows for the new menu items, update the Privacy line to list the three APIs (Open-Meteo forecast and air quality, api.weather.gov), download file name `Zephyr-v0.3.0-windows-x64.exe`.

## Step 8. Distribution (prepare only, external actions need Ali's approval)

**Files:** `.github/workflows/release.yml`, `winget/` manifests

- GitHub Actions workflow: on tag `v*`, `windows-latest`, `cargo test`, `cargo build --release`, upload `zephyr.exe` renamed to `Zephyr-v<ver>-windows-x64.exe` plus its `.sha256` as artifacts (signing step left as a commented placeholder for SignPath).
- winget manifests (portable installer type, Publisher "Ali Alhumedi", PackageIdentifier `AliAlhumedi.Zephyr`) in a `winget/` folder, ready to submit.
- Do NOT push, tag, release, open a PR on `microsoft/winget-pkgs`, or apply to SignPath. Report that these are ready and wait.

---

## Out of scope (noted, not to be done)

- The "already alerted" state is kept in memory only, so restarting Zephyr can repeat an alert the same day.
- Arabic interface, popup window, multiple cities, IP location.
