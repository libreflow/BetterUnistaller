pub mod inventory;
pub mod models;

#[tauri::command]
fn list_programs() -> Vec<models::Program> {
    inventory::registry::read_installed_programs()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_programs, inventory::size::compute_sizes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
