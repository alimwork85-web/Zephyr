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
    Some(PathBuf::from(appdata).join("Zephyr").join("config.toml"))
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
