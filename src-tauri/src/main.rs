mod commands;
mod models;
mod services;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::profile::create_profile,
            commands::profile::list_profiles,
            commands::profile::read_profile,
            commands::profile::update_profile,
            commands::profile::delete_profile,
            commands::memory::calculate_memory,
            commands::modrinth::search_modrinth,
            commands::modrinth::get_modrinth_versions,
            commands::modrinth::install_mod,
            commands::modrinth::remove_mod,
            commands::modrinth::enable_mod,
            commands::modrinth::disable_mod,
            commands::modrinth::check_mod_updates,
            commands::server::list_servers,
            commands::server::read_server_config,
            commands::server::create_server_profile,
            commands::java::detect_java,
            commands::launcher::launch_minecraft,
            commands::launcher::stream_logs,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Taurine Launcher");
}
