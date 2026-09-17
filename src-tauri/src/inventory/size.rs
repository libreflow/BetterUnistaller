use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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

/// FILE_ATTRIBUTE_REPARSE_POINT (winnt.h) — marque les jonctions NTFS, points
/// de montage et autres reparse points que `Metadata::is_symlink()` seul ne
/// détecte pas (il ne couvre que les symlinks « classiques »).
#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

#[cfg(windows)]
fn is_reparse_point(meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_meta: &std::fs::Metadata) -> bool {
    false
}

/// Somme itérative (pile explicite, aucune récursion de fonction) donc
/// insensible à la profondeur de l'arborescence — pas de stack overflow sur
/// des arbres pathologiquement profonds. Tolérante aux erreurs (accès
/// refusé, dossier disparu → contribution nulle). Ignore les liens
/// symboliques et tout autre point de reparse NTFS (jonctions, points de
/// montage) pour éviter boucles infinies et double comptage.
pub fn dir_size(path: &Path) -> u64 {
    dir_size_checked(path).unwrap_or(0)
}

/// Comme `dir_size`, mais retourne `None` si la racine elle-même est
/// illisible (`read_dir` échoue immédiatement) — pour distinguer « taille
/// réellement nulle » de « impossible à mesurer » côté appelant. Les
/// sous-dossiers illisibles rencontrés en cours de parcours contribuent
/// toujours pour 0 (comportement tolérant existant), seule la racine est
/// discriminante.
pub fn dir_size_checked(path: &Path) -> Option<u64> {
    std::fs::read_dir(path).ok()?;

    let mut total: u64 = 0;
    let mut stack: Vec<PathBuf> = vec![path.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_symlink() || is_reparse_point(&meta) {
                continue;
            }
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                total += meta.len();
            }
        }
    }

    Some(total)
}

/// Calcule séquentiellement en tâche de fond ; chaque résultat est poussé
/// via l'événement `size-computed` dès qu'il est prêt.
#[tauri::command]
pub fn compute_sizes(app: AppHandle, requests: Vec<SizeRequest>) {
    tauri::async_runtime::spawn_blocking(move || {
        for req in requests {
            // Racine illisible : on n'émet rien, l'UI garde son indicateur
            // "inconnu" plutôt que d'afficher une fausse taille de 0.
            if let Some(size_bytes) = dir_size_checked(Path::new(&req.path)) {
                let _ = app.emit("size-computed", SizeComputed { id: req.id, size_bytes });
            }
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

    #[test]
    fn handles_deeply_nested_directories_without_stack_overflow() {
        let dir = std::env::temp_dir().join("bu_size_deep_test");
        let _ = fs::remove_dir_all(&dir);

        let mut leaf = dir.clone();
        for i in 0..200 {
            leaf = leaf.join(format!("d{i}"));
        }
        fs::create_dir_all(&leaf).unwrap();
        fs::write(leaf.join("leaf.bin"), vec![0u8; 42]).unwrap();

        assert_eq!(dir_size(&dir), 42);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn dir_size_checked_returns_none_for_unreadable_root() {
        assert_eq!(
            dir_size_checked(std::path::Path::new(r"C:\bu_inexistant_xyz")),
            None
        );
    }

    #[test]
    fn dir_size_checked_returns_some_zero_for_empty_readable_dir() {
        let dir = std::env::temp_dir().join("bu_size_checked_empty_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        assert_eq!(dir_size_checked(&dir), Some(0));

        fs::remove_dir_all(&dir).unwrap();
    }
}
