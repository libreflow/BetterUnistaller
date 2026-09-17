pub mod inventory;
pub mod models;
pub mod uninstaller;

#[tauri::command]
fn list_programs() -> Vec<models::Program> {
    inventory::registry::read_installed_programs()
}

/// Désinstallation forcée d'un programme (F6). Réservée aux cas où la
/// désinstallation standard a échoué ou est absente — l'UI frontend porte la
/// responsabilité de n'exposer cette commande qu'après confirmation
/// explicite de l'utilisateur (avertissement clair, cf. cahier des charges).
#[tauri::command]
fn force_uninstall(program: models::Program) -> Result<uninstaller::force::ForceUninstallResult, String> {
    uninstaller::force::force_uninstall(&program)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_programs,
            force_uninstall,
            inventory::size::compute_sizes,
            inventory::icon::load_icons
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
