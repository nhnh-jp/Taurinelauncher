use crate::models::launch_config::LaunchResult;

pub fn launch_minecraft(_profile_path: String) -> Result<LaunchResult, String> {
    Err("Minecraft起動処理はPhase 4で実装します".to_string())
}
pub fn stream_logs(_profile_path: String) -> Result<Vec<String>, String> {
    Ok(vec!["ログ表示はPhase 4で実装します".to_string()])
}
