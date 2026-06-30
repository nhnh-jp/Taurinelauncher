use std::env;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPE: &str = "XboxLive.signin offline_access";
const USER_AGENT: &str = "TaurineLauncher/0.1.0 (github.com/nhnh-jp/Taurinelauncher)";

#[derive(Debug, Clone, Serialize)]
pub struct MicrosoftDeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MicrosoftTokenResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub scope: String,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: Option<u64>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

pub fn begin_microsoft_device_login() -> Result<MicrosoftDeviceCode, String> {
    let client_id = client_id()?;
    let response: DeviceCodeResponse = client()?
        .post(DEVICE_CODE_URL)
        .form(&[("client_id", client_id.as_str()), ("scope", SCOPE)])
        .send()
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .map_err(|error| error.to_string())?;

    Ok(MicrosoftDeviceCode {
        device_code: response.device_code,
        user_code: response.user_code,
        verification_uri: response.verification_uri,
        expires_in: response.expires_in,
        interval: response.interval.unwrap_or(5),
        message: response.message,
    })
}

pub fn poll_microsoft_device_login(
    device_code: String,
) -> Result<Option<MicrosoftTokenResult>, String> {
    let client_id = client_id()?;
    let response: TokenResponse = client()?
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", client_id.as_str()),
            ("device_code", device_code.as_str()),
        ])
        .send()
        .map_err(|error| error.to_string())?
        .json()
        .map_err(|error| error.to_string())?;

    if let Some(error) = response.error {
        return match error.as_str() {
            "authorization_pending" | "slow_down" => Ok(None),
            _ => Err(response.error_description.unwrap_or(error)),
        };
    }

    let access_token = response
        .access_token
        .ok_or_else(|| "Microsoft token response did not include access_token".to_string())?;
    Ok(Some(MicrosoftTokenResult {
        access_token,
        refresh_token: response.refresh_token.unwrap_or_default(),
        expires_in: response.expires_in.unwrap_or_default(),
        scope: response.scope.unwrap_or_default(),
    }))
}

fn client_id() -> Result<String, String> {
    env::var("TAURINE_MICROSOFT_CLIENT_ID").map_err(|_| {
        "TAURINE_MICROSOFT_CLIENT_ID is not set. Register an Azure app and set its client id before Microsoft login.".to_string()
    })
}

fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|error| error.to_string())
}
