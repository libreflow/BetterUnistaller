use std::path::Path;

/// Envoie `path` à la corbeille (F6/F7 — les suppressions sont réversibles
/// par défaut). `Ok(())` si le chemin n'existe déjà plus : la désinstallation
/// forcée est idempotente, un dossier absent n'est pas un échec.
pub fn trash_path(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if !p.exists() {
        return Ok(());
    }
    trash::delete(p).map_err(|e| format!("envoi à la corbeille impossible pour {path} : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn trash_path_removes_existing_directory() {
        let dir = std::env::temp_dir().join("bu_trash_test_dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("f.txt"), b"x").unwrap();

        trash_path(dir.to_str().unwrap()).expect("doit réussir");
        assert!(!dir.exists());
    }

    #[test]
    fn trash_path_on_missing_path_is_ok() {
        assert!(trash_path(r"C:\bu_inexistant_xyz_trash_test").is_ok());
    }
}
