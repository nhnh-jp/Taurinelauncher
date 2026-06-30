use crate::services::auth_service::{
    self, MicrosoftAuthCodeResult, MicrosoftBrowserLogin, MicrosoftDeviceCode, MicrosoftTokenResult,
};

#[tauri::command]
pub fn begin_microsoft_browser_login() -> Result<MicrosoftBrowserLogin, String> {
    auth_service::begin_microsoft_browser_login()
}

#[tauri::command]
pub fn poll_microsoft_browser_login(
    state: String,
) -> Result<Option<MicrosoftAuthCodeResult>, String> {
    auth_service::poll_microsoft_browser_login(state)
}

#[tauri::command]
pub fn begin_microsoft_device_login() -> Result<MicrosoftDeviceCode, String> {
    auth_service::begin_microsoft_device_login()
}

#[tauri::command]
pub fn poll_microsoft_device_login(
    device_code: String,
) -> Result<Option<MicrosoftTokenResult>, String> {
    auth_service::poll_microsoft_device_login(device_code)
}
