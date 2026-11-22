# Changelog

## 0.4.0

### Added
- **Frost / heat alerts.** One notification a day when tomorrow's low reaches 0 °C (32 °F) or its high reaches 40 °C (104 °F), e.g. "Frost tomorrow in Riyadh: low -2°C". Off by default; turn it on from the menu.
- **Left click opens the forecast.** Clicking the tray icon opens [Windy](https://www.windy.com/) at your city. Right click still opens the menu.

### Changed
- **New name: HoverSky** (was Zephyr). Settings in `%APPDATA%\Zephyr\config.toml` are copied once to `%APPDATA%\HoverSky\config.toml` (the old file is kept), and an old "Start with Windows" entry is moved to the new name so the app never starts twice.
- **New app icon:** a degree ring, cold blue on the left and hot red on the right.
- **Alerts are remembered.** Rain, air quality and severe weather notifications are not repeated after a restart; the memory lives in `%APPDATA%\HoverSky\state.toml`.

### Fixed
- **A fresh number after sleep.** The 30-minute wait used a monotonic timer, so after the laptop woke from sleep the tooltip could show an old temperature for hours. The wall clock is now checked every minute.

## 0.3.2

### Changed
- **Umbrella icon.** While it is raining (now, or rain starting within the hour) the tray icon is a white umbrella instead of the temperature; the temperature stays one hover away in the tooltip. When it is dry the icon shows the temperature as before.
- The blue rain strip at the bottom of the icon is gone: the umbrella replaces it.

## 0.3.1

### Changed
- The README is now bilingual: English first, then Arabic the same content.
- No code change; the EXE is identical to 0.3.0.

## 0.3.0

### Added
- **Rain in the next hour.** A notification when rain starts within the next 60 minutes, with the time it begins. Data refreshes every 30 minutes, so the warning comes at least about half an hour ahead.
- **Air quality alerts.** One notification a day when the US AQI reaches 101 (unhealthy for sensitive groups) or worse, with the EPA category.
- **Official US warnings.** Active severe or extreme National Weather Service warnings for your city, each one notified once. US only.
- Menu switches for the air quality and severe weather alerts.

### Changed
- Notifications show "Zephyr" as the sender instead of "Windows PowerShell".
- Publisher, author and copyright are Ali Alhumedi in the EXE, the Cargo package and the license.
- The HTTP user agent now carries the project URL, as api.weather.gov asks for.

### Fixed
- The *Change City* window froze for up to 15 seconds while searching. The search now runs in the background: the window can be moved and closed, and the Search button stays disabled until the results arrive.

## 0.2.3

### Fixed
- Right after Windows starts the network is often not ready, and Zephyr waited 2 minutes before trying again. It now retries after 5, 10, 20 and 40 seconds (then every 2 minutes at most), so the weather appears within seconds of the connection coming up.
- The tooltip said "Tmrw"; it now spells out "Tomorrow".

## 0.2.2

### Fixed
- "Start with Windows" stopped working if the EXE was moved, renamed or replaced by a newer version. Zephyr now refreshes the startup entry to its own location every time it runs.

## 0.2.1

### Fixed
- The tooltip was cut off after about 63 characters by Windows, so the "Updated" line and the rain details were lost. It is now compact (city name first, then Today, Tmrw and the update time) and always fits.
- While offline the tooltip shows "Offline" next to the last known weather.

## 0.2.0

### Added
- Change city from the tray menu (search any city worldwide).
- Celsius / Fahrenheit switch.
- Temperature drawn into the tray icon, colored by how warm it is.
- Daily rain notification (50% or more), can be turned off.
- "Start with Windows" option.
- "Updated HH:MM" line in the tooltip.
- Settings saved in `%APPDATA%\Zephyr\config.toml`.

### Fixed
- The tooltip never refreshed on its own; it now updates every 30 minutes.
- "Refresh Now" could freeze the app; it now runs in the background with a 15 second timeout.
- A failed request showed a fake 0.0 F reading; it now says the weather is unavailable and retries.
- The build only worked on the author's machine because of an absolute icon path.
- Exit now removes the tray icon cleanly.

### Changed
- Typed JSON parsing, unit tests, smaller dependency set (no `image` crate).
- `Cargo.lock` is now tracked.

## 0.1.0
- First version: Sanger, CA weather in the tray tooltip.
