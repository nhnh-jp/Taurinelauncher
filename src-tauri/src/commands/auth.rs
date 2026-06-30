use crate::services::auth_service::{self, MicrosoftDeviceCode, MicrosoftTokenResult};

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
