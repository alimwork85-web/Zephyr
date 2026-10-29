//! "Start with Windows" via the per-user Run registry key (no admin needed).

use winreg::{enums::HKEY_CURRENT_USER, RegKey};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "Zephyr";

pub fn is_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(RUN_KEY)
        .and_then(|k| k.get_value::<String, _>(VALUE))
        .is_ok()
}

pub fn set(enabled: bool) -> std::io::Result<()> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(RUN_KEY)?;
    if enabled {
        let exe = std::env::current_exe()?;
        key.set_value(VALUE, &format!("\"{}\"", exe.display()))
    } else {
        match key.delete_value(VALUE) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

/// Names the toast sender: without this entry Windows shows "Windows PowerShell".
pub fn register_app_id(app_id: &str) -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(format!(r"Software\Classes\AppUserModelId\{app_id}"))
        .and_then(|(key, _)| key.set_value("DisplayName", &"Zephyr"))
        .is_ok()
}
