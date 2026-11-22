# SignPath Foundation application (draft)

Paste the text below into https://signpath.org/apply (or the SignPath Foundation
form). This file is the draft only: nothing has been submitted.

---

**Project name:** HoverSky

**Project URL:** https://github.com/alimwork85-web/Zephyr

**Download page:** https://github.com/alimwork85-web/Zephyr/releases/latest

**License:** MIT (OSI-approved). License text: https://github.com/alimwork85-web/Zephyr/blob/main/LICENSE

**Maintainer:** Ali Alhumedi

**What the software does:** HoverSky is a Windows system tray weather app written
in Rust. It shows the current temperature in the tray icon and today's and
tomorrow's high/low, rain chance and active alerts in the tooltip, with optional
Windows notifications for rain, air quality, official US weather warnings, and
frost or heat. It has no window beyond a city picker, no account, no installer,
no telemetry, and no auto-update.

**Why we need signing:** the releases are unsigned, so Windows SmartScreen shows
"Windows protected your PC" for every download and the publisher is unknown. A
free code signing certificate from the SignPath Foundation removes that warning
and lets users verify that the EXE comes from this repository.

**Build process:** fully automated in GitHub Actions on a `v*` tag push:
https://github.com/alimwork85-web/Zephyr/blob/main/.github/workflows/release.yml
The workflow runs `cargo test`, then `cargo build --release` on `windows-latest`,
names the binary `HoverSky-v<version>-windows-x64.exe`, writes its SHA256
checksum and uploads it as a build artifact. Sources are public and the
dependencies are ordinary crates.io packages declared in `Cargo.toml`.

**Reproducibility:** the build needs only the Rust MSVC toolchain (`rustup`,
stable) on Windows; `Cargo.lock` pins every dependency.

**No bundled third-party binaries:** the EXE is a single Rust binary built from
this repository; the icon is generated in code and from an ICO file in the repo.
Weather data comes at runtime from the free Open-Meteo API.

**Contact:** Ali Alhumedi, project owner, via GitHub issues on the project URL.

---

## Notes for Ali (do not paste)

- SignPath Foundation asks for a project that is free and open source with an
  OSI license, a public repository, and an active maintainer; all of the above
  is stated in the text.
- They also ask that the build runs in CI and that signing is a step of that
  workflow. `release.yml` currently has a signing placeholder comment where the
  SignPath step goes; add it after approval, not before.
- Do not submit anything until you decide to: this plan asked for the draft only.
