use std::sync::Mutex;

pub mod inventory;
pub mod models;
pub mod uninstaller;

use uninstaller::elevated_flow::{
    force_uninstall_or_request_elevation, run_pending_uninstall_if_any, ForceUninstallOutcome,
    PendingUninstallState,
};

#[tauri::command]
fn list_programs() -> Vec<models::Program> {
    inventory::registry::read_installed_programs()
}

/// Désinstallation forcée d'un programme (F6). Réservée aux cas où la
/// désinstallation standard a échoué ou est absente — l'UI frontend porte la
/// responsabilité de n'exposer cette commande qu'après confirmation
/// explicite de l'utilisateur (avertissement clair, cf. cahier des charges).
/// Demande automatiquement l'élévation UAC si le programme est machine-wide
/// et que le process courant ne l'a pas déjà (§5.3) : dans ce cas l'app se
/// ferme pour relancer une instance élevée qui termine l'opération.
#[tauri::command]
fn force_uninstall(app: tauri::AppHandle, program_id: String) -> ForceUninstallOutcome {
    force_uninstall_or_request_elevation(app, &program_id)
}

/// Récupère (et consomme) le résultat d'une désinstallation forcée qui a
/// motivé une relance élevée au démarrage — `None` si l'instance courante
/// n'a pas été lancée pour reprendre une opération en attente.
#[tauri::command]
fn take_pending_uninstall_outcome(
    state: tauri::State<PendingUninstallState>,
) -> Option<ForceUninstallOutcome> {
    state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let pending_outcome = run_pending_uninstall_if_any(&args);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(PendingUninstallState(Mutex::new(pending_outcome)))
        .invoke_handler(tauri::generate_handler![
            list_programs,
            force_uninstall,
            take_pending_uninstall_outcome,
            inventory::size::compute_sizes,
            inventory::icon::load_icons
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
