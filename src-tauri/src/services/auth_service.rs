use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const SCOPE: &str = "XboxLive.signin offline_access";
const USER_AGENT: &str = "TaurineLauncher/0.1.0 (github.com/nhnh-jp/Taurinelauncher)";

static BROWSER_LOGIN_RESULTS: OnceLock<Mutex<HashMap<String, BrowserLoginState>>> = OnceLock::new();

#[derive(Debug, Clone)]
enum BrowserLoginState {
    Pending,
    Code(String),
    Error(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct MicrosoftBrowserLogin {
    pub state: String,
    pub auth_url: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MicrosoftAuthCodeResult {
    pub code: String,
    pub state: String,
}

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

pub fn begin_microsoft_browser_login() -> Result<MicrosoftBrowserLogin, String> {
    let client_id = client_id()?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{}/auth/microsoft/callback", port);
    let state = create_state();
    let auth_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&prompt=select_account",
        AUTHORIZE_URL,
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(SCOPE),
        urlencoding::encode(&state),
    );

    results()
        .lock()
        .map_err(|error| error.to_string())?
        .insert(state.clone(), BrowserLoginState::Pending);
    spawn_callback_listener(listener, state.clone());
    open_browser(&auth_url)?;

    Ok(MicrosoftBrowserLogin {
        state,
        auth_url,
        redirect_uri,
    })
}

pub fn poll_microsoft_browser_login(
    state: String,
) -> Result<Option<MicrosoftAuthCodeResult>, String> {
    let mut guard = results().lock().map_err(|error| error.to_string())?;
    match guard.get(&state).cloned() {
        Some(BrowserLoginState::Pending) => Ok(None),
        Some(BrowserLoginState::Code(code)) => {
            guard.remove(&state);
            Ok(Some(MicrosoftAuthCodeResult { code, state }))
        }
        Some(BrowserLoginState::Error(error)) => {
            guard.remove(&state);
            Err(error)
        }
        None => {
            Err("Microsoft browser login session was not found or already completed".to_string())
        }
    }
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

fn spawn_callback_listener(listener: TcpListener, expected_state: String) {
    thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(300);
        while Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _addr)) => {
                    let mut buffer = [0_u8; 4096];
                    let read = stream.read(&mut buffer).unwrap_or(0);
                    let request = String::from_utf8_lossy(&buffer[..read]);
                    let result = parse_callback_request(&request, &expected_state);
                    let body = match &result {
                        Ok(_) => {
                            "Microsoft login code received. You can return to Taurine Launcher."
                        }
                        Err(_) => "Microsoft login failed. You can return to Taurine Launcher.",
                    };
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let mut guard = match results().lock() {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };
                    match result {
                        Ok(code) => {
                            guard.insert(expected_state, BrowserLoginState::Code(code));
                        }
                        Err(error) => {
                            guard.insert(expected_state, BrowserLoginState::Error(error));
                        }
                    }
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(150));
                }
                Err(error) => {
                    if let Ok(mut guard) = results().lock() {
                        guard.insert(expected_state, BrowserLoginState::Error(error.to_string()));
                    }
                    return;
                }
            }
        }
        if let Ok(mut guard) = results().lock() {
            guard.insert(
                expected_state,
                BrowserLoginState::Error("Microsoft login timed out".to_string()),
            );
        }
    });
}

fn parse_callback_request(request: &str, expected_state: &str) -> Result<String, String> {
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| "empty callback request".to_string())?;
    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "callback request did not include a path".to_string())?;
    let query = path
        .split_once('?')
        .map(|(_, query)| query)
        .ok_or_else(|| "callback request did not include query parameters".to_string())?;
    let values = parse_query(query)?;
    if let Some(error) = values.get("error") {
        return Err(values
            .get("error_description")
            .cloned()
            .unwrap_or_else(|| error.clone()));
    }
    let state = values
        .get("state")
        .ok_or_else(|| "callback did not include state".to_string())?;
    if state != expected_state {
        return Err("callback state did not match login session".to_string());
    }
    values
        .get("code")
        .cloned()
        .ok_or_else(|| "callback did not include authorization code".to_string())
}

fn parse_query(query: &str) -> Result<HashMap<String, String>, String> {
    let mut values = HashMap::new();
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        values.insert(
            urlencoding::decode(key)
                .map_err(|error| error.to_string())?
                .to_string(),
            urlencoding::decode(value)
                .map_err(|error| error.to_string())?
                .to_string(),
        );
    }
    Ok(values)
}

fn open_browser(url: &str) -> Result<(), String> {
    let status = if cfg!(windows) {
        Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(url).status()
    } else {
        Command::new("xdg-open").arg(url).status()
    }
    .map_err(|error| error.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("failed to open browser for Microsoft login".to_string())
    }
}

fn create_state() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("taurine-{}-{}", std::process::id(), nanos)
}

fn results() -> &'static Mutex<HashMap<String, BrowserLoginState>> {
    BROWSER_LOGIN_RESULTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn client_id() -> Result<String, String> {
    env::var("TAURINE_MICROSOFT_CLIENT_ID").map_err(|_| {
        "TAURINE_MICROSOFT_CLIENT_ID is not set. Register an Azure app with a localhost redirect URI before Microsoft login.".to_string()
    })
}

fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|error| error.to_string())
}
