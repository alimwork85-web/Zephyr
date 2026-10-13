# Zephyr Weather Tray

**The weather, one hover away.** Zephyr is a tiny Windows system tray app that shows the current temperature right in your tray icon and the full picture (now, today, tomorrow, rain chance) when you point at it. No browser, no window, no account.

Built in Rust. About 2 MB, around 20 MB of RAM, and free forever (MIT).

## Features

- **Temperature in the icon.** The tray icon itself shows the current temperature, colored from cold blue to hot red, with a blue strip when rain is likely.
- **Hover for details.** Now, today's high/low, tomorrow's high/low, and an umbrella with the rain chance when it is 30% or more.
- **Any city in the world.** Right-click, choose *Change City...*, type a name and pick from the results. Your choice is remembered.
- **Celsius or Fahrenheit.** One click in the menu.
- **Rain alerts.** One Windows notification per day when rain is 50% or more likely. Turn it off from the menu.
- **Start with Windows.** Optional, from the menu. No admin rights needed.
- **Always fresh.** Updates every 30 minutes, retries after 2 minutes if the connection drops, and says so in the tooltip instead of showing wrong numbers.
- **Private.** No accounts, no tracking, no analytics. The only network calls are to the free [Open-Meteo](https://open-meteo.com/) weather and geocoding APIs.

## Download

1. Open the [latest release](https://github.com/alimwork85-web/Zephyr/releases/latest).
2. Download `Zephyr-v0.2.1-windows-x64.exe` and run it. There is no installer.
3. Look for the new icon in the notification area (click the `^` arrow if Windows hides it, and drag the icon out to keep it visible).

> **SmartScreen note:** the EXE is not code-signed yet, so Windows may show "Windows protected your PC". Click *More info*, then *Run anyway*. You can compare the file against the SHA256 checksum published in the release, or build it yourself from source (below).

Requires Windows 10 or 11 (64-bit) and an internet connection.

## Usage

| Action | How |
| --- | --- |
| See the forecast | Hover over the tray icon |
| Refresh now | Right-click, *Refresh Now* |
| Change city | Right-click, *Change City...*, search, select, *Use this city* |
| Switch units | Right-click, *Use Celsius* |
| Rain notifications | Right-click, *Rain alerts* |
| Launch at sign-in | Right-click, *Start with Windows* |
| Quit | Right-click, *Exit* |

Settings are stored in `%APPDATA%\Zephyr\config.toml`.

## Build from source

You need the [Rust toolchain](https://rustup.rs/) on Windows (MSVC).

```bash
git clone https://github.com/alimwork85-web/Zephyr.git
cd Zephyr
cargo test
cargo build --release
```

The binary is `target/release/zephyr.exe`.

## How it works

Rust, [tokio](https://tokio.rs/) and [reqwest](https://github.com/seanmonstar/reqwest) for networking, [tray-icon](https://github.com/tauri-apps/tray-icon) and [winit](https://github.com/rust-windowing/winit) for the tray, [native-windows-gui](https://github.com/gabdube/native-windows-gui) for the city picker, and [Open-Meteo](https://open-meteo.com/) for data. The temperature icon is drawn pixel by pixel in code, so there is no font or image dependency.

## Contributing

Issues and pull requests are welcome. Ideas: more languages, hourly view, wind and humidity, code signing.

## License

[MIT](LICENSE). Weather data by [Open-Meteo.com](https://open-meteo.com/) (CC BY 4.0).
