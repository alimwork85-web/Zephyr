Zephyr Weather Tray
Zephyr Weather Tray is a lightweight, zero-footprint Windows system tray application written in Rust. It provides real-time local weather updates for Sanger, CA directly from your system tray without clogging your taskbar or requiring heavy web browsers.

🌟 Key Features
System Tray Integration: Operates discreetly in the Windows notification area with a custom native icon.

Real-Time Forecasts: Fetches live weather data from Open-Meteo, including current temperature, daily highs/lows, and tomorrow's forecast.

Rich Tooltip Overlay: Hover over the tray icon to instantly view formatted temperature metrics at a glance.

Context Menu Control: Right-click menu featuring Refresh Now for manual updates and a clean Exit option.

Low Resource Footprint: Built with asynchronous Rust (tokio and tray-icon) to run with minimal CPU and memory usage.

Embedded Assets: Self-contained executable featuring a bundled Windows icon resource via winres.

🚀 Tech Stack
Language: Rust

Async Runtime: Tokio

HTTP Client: reqwest

System Tray & Events: tray-icon & winit

API: Open-Meteo API

🛠️ Build & Run
Prerequisites
Rust toolchain (cargo, rustc)

Windows OS

Building from Source
DOS
# Clone the repository
git clone https://github.com/your-username/zephyr.git
cd zephyr

# Build optimized release binary
cargo build --release
The compiled binary will be available at target/release/zephyr.exe.

📜 License
This project is open-source and available under the MIT License.
