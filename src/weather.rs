use crate::config::{City, Config};
use serde::Deserialize;

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
const GEOCODE_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

#[derive(Debug, Clone, PartialEq)]
pub struct Weather {
    pub current_temp: f64,
    pub current_rain: bool,
    pub today_max: f64,
    pub today_min: f64,
    pub today_rain_prob: u32,
    pub tomorrow_max: f64,
    pub tomorrow_min: f64,
    pub tomorrow_rain_prob: u32,
    /// `HH:MM` of the first wet 15-minute slot within the next hour.
    pub rain_starts: Option<String>,
}

#[derive(Deserialize)]
struct ForecastResponse {
    current_weather: CurrentWeather,
    daily: Daily,
    minutely_15: Option<Minutely15>,
}

#[derive(Deserialize)]
struct CurrentWeather {
    time: String,
    temperature: f64,
    weathercode: u32,
}

#[derive(Deserialize)]
struct Minutely15 {
    time: Vec<String>,
    precipitation: Vec<Option<f64>>,
}

#[derive(Deserialize)]
struct Daily {
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    precipitation_probability_max: Vec<Option<f64>>,
}

#[derive(Deserialize)]
struct GeocodeResponse {
    results: Option<Vec<GeocodeResult>>,
}

#[derive(Deserialize)]
struct GeocodeResult {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
    admin1: Option<String>,
}

pub fn is_rainy_code(code: u32) -> bool {
    matches!(code, 51..=67 | 80..=82 | 95..=99)
}

fn parse_forecast(body: &str) -> Result<Weather, String> {
    let r: ForecastResponse =
        serde_json::from_str(body).map_err(|e| format!("bad forecast data: {e}"))?;
    let d = r.daily;
    if d.temperature_2m_max.len() < 2 || d.temperature_2m_min.len() < 2 {
        return Err("forecast has fewer than 2 days".into());
    }
    let prob = |i: usize| {
        d.precipitation_probability_max
            .get(i)
            .copied()
            .flatten()
            .unwrap_or(0.0)
            .round() as u32
    };
    Ok(Weather {
        current_temp: r.current_weather.temperature,
        current_rain: is_rainy_code(r.current_weather.weathercode),
        today_max: d.temperature_2m_max[0],
        today_min: d.temperature_2m_min[0],
        today_rain_prob: prob(0),
        tomorrow_max: d.temperature_2m_max[1],
        tomorrow_min: d.temperature_2m_min[1],
        tomorrow_rain_prob: prob(1),
        rain_starts: r
            .minutely_15
            .as_ref()
            .and_then(|m| rain_start(&r.current_weather.time, m)),
    })
}

/// First 15-minute slot after `now` (and within the hour) with real precipitation.
fn rain_start(now: &str, m: &Minutely15) -> Option<String> {
    const FMT: &str = "%Y-%m-%dT%H:%M";
    let now = chrono::NaiveDateTime::parse_from_str(now, FMT).ok()?;
    m.time.iter().zip(&m.precipitation).find_map(|(t, mm)| {
        let at = chrono::NaiveDateTime::parse_from_str(t, FMT).ok()?;
        let ahead = at.signed_duration_since(now).num_minutes();
        (ahead > 0 && ahead <= 60 && mm.unwrap_or(0.0) >= 0.1)
            .then(|| at.format("%H:%M").to_string())
    })
}

/// Delay before retry number `failures` (1-based): 5s, 10s, 20s, 40s, 80s, then 120s.
/// Starting fast matters because the network is often not ready yet right after boot.
pub fn retry_delay(failures: u32) -> std::time::Duration {
    let secs = 5u64.saturating_mul(1 << failures.saturating_sub(1).min(5));
    std::time::Duration::from_secs(secs.min(120))
}

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent(concat!("Zephyr/", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap_or_default()
}

pub async fn fetch_weather(client: &reqwest::Client, cfg: &Config) -> Result<Weather, String> {
    let unit = if cfg.celsius { "celsius" } else { "fahrenheit" };
    let body = client
        .get(FORECAST_URL)
        .query(&[
            ("latitude", cfg.city.latitude.to_string()),
            ("longitude", cfg.city.longitude.to_string()),
            ("current_weather", "true".to_string()),
            (
                "daily",
                "temperature_2m_max,temperature_2m_min,precipitation_probability_max".to_string(),
            ),
            ("minutely_15", "precipitation".to_string()),
            ("forecast_minutely_15", "8".to_string()),
            ("temperature_unit", unit.to_string()),
            ("timezone", "auto".to_string()),
        ])
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    parse_forecast(&body)
}

pub async fn search_cities(client: &reqwest::Client, query: &str) -> Result<Vec<City>, String> {
    let r: GeocodeResponse = client
        .get(GEOCODE_URL)
        .query(&[("name", query), ("count", "8"), ("language", "en"), ("format", "json")])
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    Ok(r.results
        .unwrap_or_default()
        .into_iter()
        .map(|g| City {
            name: g.name,
            region: g.admin1.unwrap_or_default(),
            country: g.country.unwrap_or_default(),
            latitude: g.latitude,
            longitude: g.longitude,
        })
        .collect())
}

/// Windows cuts tray tooltips at 63 UTF-16 units, so the text is built compactly
/// and whole lines are dropped from the bottom before anything is cut mid-line.
pub const TOOLTIP_MAX: usize = 63;

pub fn format_tooltip(city: &City, celsius: bool, w: &Weather, footer: &str) -> String {
    let u = if celsius { "C" } else { "F" };
    let umbrella = |p: u32| if p >= 30 { format!("\u{2602}{p}%") } else { String::new() };
    let name: String = city.name.chars().take(12).collect();
    let rain = if w.current_rain { "\u{2602}" } else { "" };
    let mut lines = vec![
        format!("{} {:.1}\u{b0}{}{}", name, w.current_temp, u, rain),
        format!("Today {:.0}\u{b0}/{:.0}\u{b0}{}", w.today_max, w.today_min, umbrella(w.today_rain_prob)),
        format!("Tomorrow {:.0}\u{b0}/{:.0}\u{b0}{}", w.tomorrow_max, w.tomorrow_min, umbrella(w.tomorrow_rain_prob)),
        footer.to_string(),
    ];
    let units = |s: &str| s.encode_utf16().count();
    while lines.len() > 1 && units(&lines.join("\n")) > TOOLTIP_MAX {
        lines.pop();
    }
    let mut text = lines.join("\n");
    while units(&text) > TOOLTIP_MAX {
        text.pop();
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "current_weather": {"time": "2026-10-28T14:00", "temperature": 71.6, "weathercode": 61},
        "daily": {
            "temperature_2m_max": [90.1, 88.0],
            "temperature_2m_min": [60.4, 58.9],
            "precipitation_probability_max": [45, null]
        }
    }"#;

    /// Wet slot 30 minutes after `current_weather.time`, dry slots on either side.
    const RAIN_SOON: &str = r#"{
        "current_weather": {"time": "2026-10-28T14:00", "temperature": 60.0, "weathercode": 3},
        "daily": {
            "temperature_2m_max": [70.0, 71.0],
            "temperature_2m_min": [50.0, 51.0],
            "precipitation_probability_max": [10, 20]
        },
        "minutely_15": {
            "time": ["2026-10-28T14:00", "2026-10-28T14:15", "2026-10-28T14:30", "2026-10-28T14:45"],
            "precipitation": [0.0, 0.0, 0.4, 1.2]
        }
    }"#;

    /// First wet slot is 90 minutes ahead, outside the alert window.
    const RAIN_LATER: &str = r#"{
        "current_weather": {"time": "2026-10-28T14:00", "temperature": 60.0, "weathercode": 3},
        "daily": {
            "temperature_2m_max": [70.0, 71.0],
            "temperature_2m_min": [50.0, 51.0],
            "precipitation_probability_max": [10, 20]
        },
        "minutely_15": {
            "time": ["2026-10-28T14:45", "2026-10-28T15:30"],
            "precipitation": [0.0, 0.5]
        }
    }"#;

    #[test]
    fn retry_delay_backs_off_and_caps() {
        let secs: Vec<u64> = (1..=8).map(|n| retry_delay(n).as_secs()).collect();
        assert_eq!(secs, vec![5, 10, 20, 40, 80, 120, 120, 120]);
    }

    #[test]
    fn rain_codes() {
        assert!(is_rainy_code(61));
        assert!(is_rainy_code(95));
        assert!(!is_rainy_code(0));
        assert!(!is_rainy_code(3));
    }

    #[test]
    fn parses_sample_and_tolerates_null_probability() {
        let w = parse_forecast(SAMPLE).unwrap();
        assert_eq!(w.today_rain_prob, 45);
        assert_eq!(w.tomorrow_rain_prob, 0);
        assert!(w.current_rain);
    }

    #[test]
    fn rejects_garbage_and_short_forecasts() {
        assert!(parse_forecast("{}").is_err());
        let short = r#"{"current_weather":{"temperature":1.0,"weathercode":0},
            "daily":{"temperature_2m_max":[1.0],"temperature_2m_min":[0.0],"precipitation_probability_max":[0]}}"#;
        assert!(parse_forecast(short).is_err());
    }

    #[test]
    fn tooltip_typical_case_keeps_every_line() {
        let w = parse_forecast(SAMPLE).unwrap();
        let t = format_tooltip(&City::default(), false, &w, "Upd 14:05");
        assert!(t.starts_with("Sanger 71.6\u{b0}F\u{2602}"));
        assert!(t.contains("\u{2602}45%"));
        assert!(t.contains("Tomorrow 88\u{b0}/59\u{b0}"));
        assert!(t.ends_with("Upd 14:05"));
        assert!(t.encode_utf16().count() <= TOOLTIP_MAX);
    }

    #[test]
    fn tooltip_keeps_footer_with_rain_on_both_days() {
        let mut w = parse_forecast(SAMPLE).unwrap();
        w.tomorrow_rain_prob = 40;
        let t = format_tooltip(&City::default(), false, &w, "Upd 22:52");
        assert!(t.ends_with("Upd 22:52"), "footer was dropped: {t}");
    }

    #[test]
    fn rain_within_the_hour_sets_time() {
        let w = parse_forecast(RAIN_SOON).unwrap();
        assert_eq!(w.rain_starts.as_deref(), Some("14:30"));
    }

    #[test]
    fn rain_beyond_the_hour_is_ignored() {
        let w = parse_forecast(RAIN_LATER).unwrap();
        assert_eq!(w.rain_starts, None);
    }

    #[test]
    fn missing_minutely_data_means_no_rain_alert() {
        let w = parse_forecast(SAMPLE).unwrap();
        assert_eq!(w.rain_starts, None);
    }

    #[test]
    fn tooltip_never_exceeds_limit() {
        let w = Weather {
            current_temp: -123.4,
            current_rain: true,
            today_max: -100.0,
            today_min: -120.0,
            today_rain_prob: 100,
            tomorrow_max: 120.0,
            tomorrow_min: -110.0,
            tomorrow_rain_prob: 100,
            rain_starts: Some("14:30".into()),
        };
        let mut city = City::default();
        city.name = "Llanfairpwllgwyngyll".into();
        let t = format_tooltip(&city, true, &w, "Upd 14:05");
        assert!(t.encode_utf16().count() <= TOOLTIP_MAX);
        assert!(t.contains("Today"));
    }
}
