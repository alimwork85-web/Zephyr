#![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

const API_URL: &str = "https://api.open-meteo.com/v1/forecast";
const LATITUDE: f64 = 36.7080;
const LONGITUDE: f64 = -119.5559;

const ICON_BYTES: &[u8] = include_bytes!(r"C:\Users\suailali\Desktop\المستودع\Zephyr\Zephyr.ico");

#[derive(Debug, Clone)]
struct ZephyrState {
    current_temp: f64,
    max_temp: f64,
    min_temp: f64,
    tomorrow_max: f64,
    tomorrow_min: f64,
}

async fn fetch_weather() -> Result<ZephyrState, Box<dyn std::error::Error>> {
    let params = format!(
        "latitude={}&longitude={}&current_weather=true&daily=temperature_2m_max,temperature_2m_min&temperature_unit=fahrenheit&windspeed_unit=mph&timezone=auto",
        LATITUDE, LONGITUDE
    );

    let client = reqwest::Client::new();
    let url = format!("{}?{}", API_URL, params);
    let response = client.get(&url).send().await?;
    let json_body: serde_json::Value = response.json().await?;

    Ok(ZephyrState {
        current_temp: json_body["current_weather"]["temperature"].as_f64().unwrap_or(0.0),
        max_temp: json_body["daily"]["temperature_2m_max"][0].as_f64().unwrap_or(0.0),
        min_temp: json_body["daily"]["temperature_2m_min"][0].as_f64().unwrap_or(0.0),
        tomorrow_max: json_body["daily"]["temperature_2m_max"][1].as_f64().unwrap_or(0.0),
        tomorrow_min: json_body["daily"]["temperature_2m_min"][1].as_f64().unwrap_or(0.0),
    })
}

fn format_tooltip(state: &ZephyrState) -> String {
    format!(
        "Zephyr (Sanger, CA)\nNow: {:.1}°F\nToday: {:.0}° / {:.0}°F\nTomorrow: {:.0}° / {:.0}°F",
        state.current_temp, state.max_temp, state.min_temp, state.tomorrow_max, state.tomorrow_min
    )
}

fn load_icon() -> Icon {
    let image = image::load_from_memory(ICON_BYTES)
        .expect("Failed to load icon bytes")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoopBuilder::new().build()?;
    let rt = tokio::runtime::Runtime::new()?;

    let initial_state = rt.block_on(async {
        fetch_weather().await.unwrap_or(ZephyrState {
            current_temp: 0.0,
            max_temp: 0.0,
            min_temp: 0.0,
            tomorrow_max: 0.0,
            tomorrow_min: 0.0,
        })
    });

    let state_arc = Arc::new(Mutex::new(initial_state.clone()));

    let menu = Menu::new();
    let refresh_item = MenuItem::new("Refresh Now", true, None);
    let exit_item = MenuItem::new("Exit", true, None);
    let _ = menu.append(&refresh_item);
    let _ = menu.append(&exit_item);

    let icon = load_icon();
    let tooltip_text = format_tooltip(&initial_state);

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip_text)
        .with_icon(icon)
        .build()?;

    let refresh_id = refresh_item.id().clone();
    let exit_id = exit_item.id().clone();

    let state_ref_loop = Arc::clone(&state_arc);
    rt.spawn(async move {
        loop {
            sleep(Duration::from_secs(12 * 3600)).await;
            if let Ok(new_state) = fetch_weather().await {
                let mut state = state_ref_loop.lock().unwrap();
                *state = new_state;
            }
        }
    });

    let menu_channel = MenuEvent::receiver();
    let state_ref_menu = Arc::clone(&state_arc);

    event_loop.run(move |_event, _target| {
        _target.set_control_flow(ControlFlow::Wait);

        if let Ok(event) = menu_channel.try_recv() {
            if event.id() == &refresh_id {
                let state_clone = Arc::clone(&state_ref_menu);
                let new_state = rt.block_on(async move {
                    fetch_weather().await.ok()
                });

                if let Some(state_data) = new_state {
                    let mut state = state_clone.lock().unwrap();
                    *state = state_data.clone();
                    let tooltip = format_tooltip(&state_data);
                    let _ = tray_icon.set_tooltip(Some(tooltip));
                }
            } else if event.id() == &exit_id {
                std::process::exit(0);
            }
        }
    })?;

    Ok(())
}