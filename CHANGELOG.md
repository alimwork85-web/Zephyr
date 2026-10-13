# Changelog

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
