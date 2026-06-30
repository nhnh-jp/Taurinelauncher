use crate::{
    models::launch_config::LaunchResult,
    services::{java_service, profile_service},
};

pub fn launch_minecraft(profile_path: String) -> Result<LaunchResult, String> {
    let profile = profile_service::read_profile(profile_path.clone())?;
    let java = java_service::detect_java()?;
    if !java.found {
        return Err(
            "Java was not found. Install Java or set JAVA_HOME before launching Minecraft."
                .to_string(),
        );
    }

    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    Ok(LaunchResult {
        command_preview: format!(
            "{} -Xmx{}M -Djava.library.path=<natives> -cp <minecraft classpath> net.minecraft.client.main.Main --version {} --gameDir {}",
            java.path,
            profile.launch.memory_max_mb,
            profile.minecraft_version,
            profile_dir.to_string_lossy()
        ),
        log_path: profile_dir.join("logs").join("latest.log").to_string_lossy().to_string(),
    })
}

pub fn stream_logs(_profile_path: String) -> Result<Vec<String>, String> {
    Ok(vec![
        "Minecraft log streaming is not connected yet.".to_string()
    ])
}
