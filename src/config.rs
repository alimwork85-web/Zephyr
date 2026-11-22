use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct City {
    pub name: String,
    pub region: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl City {
    pub fn label(&self) -> String {
        [self.name.as_str(), self.region.as_str(), self.country.as_str()]
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub celsius: bool,
    pub rain_alerts: bool,
    pub air_alerts: bool,
    pub severe_alerts: bool,
    pub city: City,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            celsius: false,
            rain_alerts: true,
            air_alerts: true,
            severe_alerts: true,
            city: City {
                name: "Sanger".into(),
                region: "California".into(),
                country: "United States".into(),
                latitude: 36.7080,
                longitude: -119.5559,
            },
        }
    }
}

impl Default for City {
    fn default() -> Self {
        Config::default().city
    }
}

fn config_path() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    let dir = PathBuf::from(appdata);
    let path = dir.join("HoverSky").join("config.toml");
    adopt_legacy(&path, &dir.join("Zephyr").join("config.toml"));
    Some(path)
}

/// The app was called Zephyr up to 0.3.2: copy (never move) its settings once.
fn adopt_legacy(path: &std::path::Path, legacy: &std::path::Path) {
    if path.exists() || !legacy.exists() {
        return;
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::copy(legacy, path);
}

impl Config {
    /// Loads the saved config. A missing file gives defaults, and an unreadable
    /// one is moved aside to config.toml.bad so it is never silently lost.
    pub fn load() -> Config {
        let Some(path) = config_path() else { return Config::default() };
        let Ok(text) = std::fs::read_to_string(&path) else { return Config::default() };
        match toml::from_str(&text) {
            Ok(cfg) => cfg,
            Err(_) => {
                let _ = std::fs::rename(&path, path.with_extension("toml.bad"));
                Config::default()
            }
        }
    }

    pub fn save(&self) {
        let Some(path) = config_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, text);
        }
    }
}

/// Alert memory that must survive a restart. Kept out of `config.toml` so an
/// alert never rewrites the user's settings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    /// Dates are ISO strings (`YYYY-MM-DD`): chrono here is built without serde.
    pub air_alerted_on: Option<String>,
    pub seen_alerts: Vec<String>,
    pub rain_alerted: bool,
}

fn state_path() -> Option<PathBuf> {
    Some(config_path()?.with_file_name("state.toml"))
}

impl State {
    pub fn load() -> State {
        state_path().map(|p| State::load_from(&p)).unwrap_or_default()
    }

    fn load_from(path: &std::path::Path) -> State {
        let Ok(text) = std::fs::read_to_string(path) else { return State::default() };
        toml::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = state_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        // Only the recent ids matter, so the file stays small forever.
        let mut state = self.clone();
        let extra = state.seen_alerts.len().saturating_sub(50);
        state.seen_alerts.drain(..extra);
        if let Ok(text) = toml::to_string_pretty(&state) {
            let _ = std::fs::write(path, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_config_is_copied_once_and_kept() {
        let dir = std::env::temp_dir().join(format!("hoversky-test-{}", std::process::id()));
        let legacy = dir.join("Zephyr").join("config.toml");
        let path = dir.join("HoverSky").join("config.toml");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, "celsius = true").unwrap();
        adopt_legacy(&path, &legacy);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "celsius = true");
        assert!(legacy.exists(), "old file must stay");
        std::fs::write(&legacy, "celsius = false").unwrap();
        adopt_legacy(&path, &legacy);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "celsius = true", "new file must not be overwritten");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn roundtrip() {
        let mut cfg = Config::default();
        cfg.celsius = true;
        cfg.city.name = "London".into();
        let text = toml::to_string_pretty(&cfg).unwrap();
        assert_eq!(toml::from_str::<Config>(&text).unwrap(), cfg);
    }

    #[test]
    fn partial_file_falls_back_to_defaults() {
        let cfg: Config = toml::from_str("celsius = true").unwrap();
        assert!(cfg.celsius);
        assert_eq!(cfg.city.name, "Sanger");
    }

    #[test]
    fn old_config_without_air_alerts_gets_them_on() {
        let cfg: Config = toml::from_str("celsius = true\nrain_alerts = false").unwrap();
        assert!(!cfg.rain_alerts);
        assert!(cfg.air_alerts);
        assert!(cfg.severe_alerts);
    }

    #[test]
    fn label_skips_empty_parts() {
        let c = City { name: "Paris".into(), region: "".into(), country: "France".into(), latitude: 0.0, longitude: 0.0 };
        assert_eq!(c.label(), "Paris, France");
    }

    #[test]
    fn state_roundtrip() {
        let s = State {
            air_alerted_on: Some("2026-11-22".into()),
            seen_alerts: vec!["urn:a".into(), "urn:b".into()],
            rain_alerted: true,
        };
        let text = toml::to_string_pretty(&s).unwrap();
        assert_eq!(toml::from_str::<State>(&text).unwrap(), s);
    }

    #[test]
    fn garbage_state_file_gives_default() {
        let path = std::env::temp_dir().join(format!("hoversky-state-{}.toml", std::process::id()));
        std::fs::write(&path, "this is = = not toml").unwrap();
        assert_eq!(State::load_from(&path), State::default());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn state_missing_field_uses_default() {
        let s: State = toml::from_str("rain_alerted = true").unwrap();
        assert!(s.rain_alerted);
        assert_eq!(s.air_alerted_on, None);
        assert!(s.seen_alerts.is_empty());
    }
}
