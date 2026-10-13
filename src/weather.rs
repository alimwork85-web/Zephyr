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
}

#[derive(Deserialize)]
struct ForecastResponse {
    current_weather: CurrentWeather,
    daily: Daily,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature: f64,
    weathercode: u32,
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
    })
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

/// Windows limits tray tooltips to 127 characters, so keep this compact.
pub fn format_tooltip(city: &City, celsius: bool, w: &Weather, updated: &str) -> String {
    let u = if celsius { "C" } else { "F" };
    let umbrella = |p: u32| if p >= 30 { format!(" \u{2602}{p}%") } else { String::new() };
    let name: String = city.name.chars().take(24).collect();
    let rain = if w.current_rain { " (Rain)" } else { "" };
    let text = format!(
        "Zephyr ({})\nNow: {:.1}\u{b0}{}{}\nToday: {:.0}\u{b0}/{:.0}\u{b0}{}{}\nTomorrow: {:.0}\u{b0}/{:.0}\u{b0}{}{}\nUpdated {}",
        name,
        w.current_temp, u, rain,
        w.today_max, w.today_min, u, umbrella(w.today_rain_prob),
        w.tomorrow_max, w.tomorrow_min, u, umbrella(w.tomorrow_rain_prob),
        updated
    );
    text.chars().take(127).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "current_weather": {"temperature": 71.6, "weathercode": 61},
        "daily": {
            "temperature_2m_max": [90.1, 88.0],
            "temperature_2m_min": [60.4, 58.9],
            "precipitation_probability_max": [45, null]
        }
    }"#;

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
    fn tooltip_is_short_and_shows_umbrella() {
        let w = parse_forecast(SAMPLE).unwrap();
        let t = format_tooltip(&City::default(), false, &w, "14:05");
        assert!(t.contains("Zephyr (Sanger)"));
        assert!(t.contains("\u{2602}45%"));
        assert!(t.chars().count() <= 127);
    }
}
