#![windows_subsystem = "windows"]

mod autostart;
mod city_dialog;
mod config;
mod icon;
mod weather;

use chrono::Local;
use config::{City, Config};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::Notify;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};
use weather::Weather;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoopBuilder};

const REFRESH_EVERY: Duration = Duration::from_secs(30 * 60);
const RAIN_ALERT_THRESHOLD: u32 = 50;
const PROJECT_URL: &str = "https://github.com/alimwork85-web/Zephyr";

enum UserEvent {
    Menu(MenuId),
    Weather(Result<Weather, String>),
    CityChosen(City),
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Updates the tray tooltip and icon. `failed` only matters when there is no
/// weather to show yet.
fn show(tray: &TrayIcon, cfg: &Config, w: Option<&Weather>, footer: &str, failed: bool) {
    let (tip, icon) = match w {
        Some(w) => (
            weather::format_tooltip(&cfg.city, cfg.celsius, w, footer),
            icon::temperature_icon(w.current_temp, cfg.celsius, w.today_rain_prob >= RAIN_ALERT_THRESHOLD),
        ),
        None => {
            let line = if failed { "Weather unavailable\nRetrying soon..." } else { "Loading weather..." };
            (format!("{}\n{}", cfg.city.name.chars().take(12).collect::<String>(), line), None)
        }
    };
    let _ = tray.set_tooltip(Some(tip));
    if let Some(icon) = icon.or_else(icon::app_icon) {
        let _ = tray.set_icon(Some(icon));
    }
}

fn notify_rain(city: &City, prob: u32) {
    let _ = notify_rust::Notification::new()
        .summary(&format!("Rain expected in {}", city.name))
        .body(&format!("{prob}% chance of rain today. Take an umbrella."))
        .show();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    let client = weather::http_client();
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    let mut cfg = Config::load();
    let shared = Arc::new(Mutex::new(cfg.clone()));
    let refresh = Arc::new(Notify::new());

    let refresh_item = MenuItem::new("Refresh Now", true, None);
    let city_item = MenuItem::new("Change City...", true, None);
    let celsius_item = CheckMenuItem::new("Use Celsius (\u{b0}C)", true, cfg.celsius, None);
    let alerts_item = CheckMenuItem::new("Rain alerts", true, cfg.rain_alerts, None);
    // If "Start with Windows" is on, re-register this EXE's current path so the
    // entry never points at an old, moved or deleted file.
    let start_with_windows = autostart::is_enabled();
    if start_with_windows {
        let _ = autostart::set(true);
    }
    let startup_item = CheckMenuItem::new("Start with Windows", true, start_with_windows, None);
    let about_item = MenuItem::new(format!("Zephyr v{} - project page", env!("CARGO_PKG_VERSION")), true, None);
    let exit_item = MenuItem::new("Exit", true, None);

    let menu = Menu::new();
    menu.append_items(&[
        &refresh_item,
        &city_item,
        &PredefinedMenuItem::separator(),
        &celsius_item,
        &alerts_item,
        &startup_item,
        &PredefinedMenuItem::separator(),
        &about_item,
        &exit_item,
    ])?;

    let start_icon = icon::app_icon()
        .or_else(|| icon::temperature_icon(20.0, true, false))
        .ok_or("could not create the tray icon")?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(format!("{}\nLoading weather...", cfg.city.name.chars().take(12).collect::<String>()))
        .with_icon(start_icon)
        .build()?;

    let menu_proxy = proxy.clone();
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        let _ = menu_proxy.send_event(UserEvent::Menu(e.id));
    }));

    // Background fetcher: fetches now, then every 30 minutes (after a failure it retries
    // quickly, 5s then doubling up to 2 minutes, since the network may not be up yet at boot),
    // or immediately whenever `refresh` is notified.
    {
        let (client, shared, refresh, proxy) = (client.clone(), shared.clone(), refresh.clone(), proxy.clone());
        rt.spawn(async move {
            let mut failures = 0u32;
            loop {
                let current = lock(&shared).clone();
                let result = weather::fetch_weather(&client, &current).await;
                let wait = if result.is_ok() {
                    failures = 0;
                    REFRESH_EVERY
                } else {
                    failures += 1;
                    weather::retry_delay(failures)
                };
                if proxy.send_event(UserEvent::Weather(result)).is_err() {
                    break;
                }
                tokio::select! {
                    _ = tokio::time::sleep(wait) => {}
                    _ = refresh.notified() => {}
                }
            }
        });
    }

    let rt_handle = rt.handle().clone();
    let mut latest: Option<Weather> = None;
    let mut updated = String::new();
    let mut alerted_on: Option<chrono::NaiveDate> = None;

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);
        let Event::UserEvent(event) = event else { return };

        // Saves settings and makes the next fetch use them.
        let apply = |cfg: &Config| {
            cfg.save();
            *lock(&shared) = cfg.clone();
            refresh.notify_one();
        };

        match event {
            UserEvent::Weather(Ok(w)) => {
                updated = Local::now().format("%H:%M").to_string();
                show(&tray, &cfg, Some(&w), &format!("Upd {updated}"), false);
                let today = Local::now().date_naive();
                if cfg.rain_alerts && w.today_rain_prob >= RAIN_ALERT_THRESHOLD && alerted_on != Some(today) {
                    notify_rain(&cfg.city, w.today_rain_prob);
                    alerted_on = Some(today);
                }
                latest = Some(w);
            }
            UserEvent::Weather(Err(_)) => match &latest {
                Some(w) => show(&tray, &cfg, Some(w), "Offline", true),
                None => show(&tray, &cfg, None, "", true),
            },
            UserEvent::CityChosen(city) => {
                cfg.city = city;
                latest = None;
                alerted_on = None;
                show(&tray, &cfg, None, "", false);
                apply(&cfg);
            }
            UserEvent::Menu(id) => {
                if id == *refresh_item.id() {
                    refresh.notify_one();
                } else if id == *city_item.id() {
                    let p = proxy.clone();
                    city_dialog::open(rt_handle.clone(), client.clone(), move |c| {
                        let _ = p.send_event(UserEvent::CityChosen(c));
                    });
                } else if id == *celsius_item.id() {
                    cfg.celsius = !cfg.celsius;
                    celsius_item.set_checked(cfg.celsius);
                    latest = None;
                    show(&tray, &cfg, None, "", false);
                    apply(&cfg);
                } else if id == *alerts_item.id() {
                    cfg.rain_alerts = !cfg.rain_alerts;
                    alerts_item.set_checked(cfg.rain_alerts);
                    cfg.save();
                    *lock(&shared) = cfg.clone();
                } else if id == *startup_item.id() {
                    let _ = autostart::set(!autostart::is_enabled());
                    startup_item.set_checked(autostart::is_enabled());
                } else if id == *about_item.id() {
                    let _ = std::process::Command::new("explorer").arg(PROJECT_URL).spawn();
                } else if id == *exit_item.id() {
                    elwt.exit();
                }
            }
        }
    })?;

    Ok(())
}
