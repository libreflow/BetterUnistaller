use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeRequest {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeComputed {
    pub id: String,
    pub size_bytes: u64,
}

/// Somme récursive, tolérante aux erreurs (accès refusé, dossier disparu → ignoré).
/// Ne suit pas les liens symboliques pour éviter boucles et double comptage.
pub fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else { return 0 };
    entries
        .flatten()
        .map(|entry| {
            let Ok(meta) = entry.metadata() else { return 0 };
            if meta.is_symlink() {
                0
            } else if meta.is_dir() {
                dir_size(&entry.path())
            } else {
                meta.len()
            }
        })
        .sum()
}

/// Calcule séquentiellement en tâche de fond ; chaque résultat est poussé
/// via l'événement `size-computed` dès qu'il est prêt.
#[tauri::command]
pub fn compute_sizes(app: AppHandle, requests: Vec<SizeRequest>) {
    tauri::async_runtime::spawn_blocking(move || {
        for req in requests {
            let size_bytes = dir_size(Path::new(&req.path));
            let _ = app.emit("size-computed", SizeComputed { id: req.id, size_bytes });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sums_nested_files_and_survives_missing_dir() {
        let dir = std::env::temp_dir().join("bu_size_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("a.bin"), vec![0u8; 1000]).unwrap();
        fs::write(dir.join("sub").join("b.bin"), vec![0u8; 500]).unwrap();

        assert_eq!(dir_size(&dir), 1500);
        assert_eq!(dir_size(std::path::Path::new(r"C:\bu_inexistant_xyz")), 0);

        fs::remove_dir_all(&dir).unwrap();
    }
}
