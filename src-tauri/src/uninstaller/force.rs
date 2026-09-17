use crate::inventory::registry::delete_uninstall_key;
use crate::models::{Program, Scope};
use crate::uninstaller::{cleanup, elevation, process, protection, restore_point};

/// Étape du déroulement de la désinstallation forcée, reportée au frontend
/// pour affichage progressif (F6 : "avertissement clair" avant l'action, ici
/// on donne aussi une trace de ce qui a réellement été fait).
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "step")]
pub enum ForceUninstallStep {
    RestorePointCreated,
    RestorePointFailed { message: String },
    ProcessesTerminated { count: u32 },
    InstallLocationRemoved,
    InstallLocationRemovalFailed { message: String },
    RegistryKeyRemoved,
    RegistryKeyRemovalFailed { message: String },
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceUninstallResult {
    pub steps: Vec<ForceUninstallStep>,
    /// `true` si la clé de registre a bien été supprimée à la fin — c'est le
    /// seul signal fiable de "le programme n'apparaît plus dans la liste".
    /// Les autres étapes (kill process, suppression fichiers) sont au mieux
    /// tolérantes aux erreurs (cf. F4 : le scan de résidus rattrapera le
    /// reste), mais un échec de suppression de la clé Uninstall est
    /// remonté comme échec global.
    pub succeeded: bool,
}

/// Erreur distincte (plutôt qu'un `String` générique) pour le cas
/// "élévation requise" : le frontend en a besoin pour proposer explicitement
/// une relance élevée plutôt que d'afficher un message d'échec sec (cahier
/// des charges §5.3 : "relance élevée de l'application sur demande, avec
/// reprise du contexte").
#[derive(Debug)]
pub enum ForceUninstallError {
    ElevationRequired,
    Other(String),
}

impl std::fmt::Display for ForceUninstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ElevationRequired => write!(
                f,
                "cette désinstallation forcée nécessite l'élévation des privilèges"
            ),
            Self::Other(message) => write!(f, "{message}"),
        }
    }
}

/// `true` si `program` exige l'élévation UAC pour être désinstallé de force
/// (cahier des charges §5.3 : "les suppressions machine (HKLM, Program
/// Files) [...] exigent l'élévation"). Les entrées `Scope::User` (HKCU)
/// n'en ont jamais besoin.
pub fn requires_elevation(program: &Program) -> bool {
    program.scope == Scope::Machine
}

/// Désinstallation forcée d'un programme (F6). Séquence, dans l'ordre imposé
/// par le cahier des charges : point de restauration obligatoire → arrêt des
/// processus → suppression de l'InstallLocation (corbeille) → suppression de
/// la clé Uninstall. Refuse d'agir sur un composant protégé (F7), et refuse
/// de démarrer sur un programme machine-wide si le process courant n'est pas
/// élevé — mieux vaut échouer immédiatement et clairement que d'exécuter une
/// suppression partielle (ex : processus tués mais clé Uninstall HKLM
/// inaccessible, laissant le programme dans un état incohérent).
///
/// Chaque étape individuelle est tolérante aux erreurs sauf la suppression
/// finale de la clé de registre : c'est elle qui détermine si le programme
/// disparaît de la liste, donc son échec fait échouer l'opération globale
/// même si les étapes précédentes ont réussi.
pub fn force_uninstall(program: &Program) -> Result<ForceUninstallResult, ForceUninstallError> {
    if protection::is_protected(&program.name) {
        return Err(ForceUninstallError::Other(format!(
            "« {} » fait partie des composants protégés et ne peut pas être désinstallé de force",
            program.name
        )));
    }

    if requires_elevation(program) && !elevation::is_elevated() {
        return Err(ForceUninstallError::ElevationRequired);
    }

    let mut steps = Vec::new();

    match restore_point::create_restore_point(&format!(
        "BetterUnistaller — avant désinstallation forcée de {}",
        program.name
    )) {
        Ok(()) => steps.push(ForceUninstallStep::RestorePointCreated),
        Err(message) => steps.push(ForceUninstallStep::RestorePointFailed { message }),
    }

    if let Some(location) = &program.install_location {
        let killed = process::kill_processes_under(location);
        steps.push(ForceUninstallStep::ProcessesTerminated { count: killed });

        match cleanup::trash_path(location) {
            Ok(()) => steps.push(ForceUninstallStep::InstallLocationRemoved),
            Err(message) => {
                steps.push(ForceUninstallStep::InstallLocationRemovalFailed { message })
            }
        }
    }

    let succeeded = match delete_uninstall_key(&program.id) {
        Ok(()) => {
            steps.push(ForceUninstallStep::RegistryKeyRemoved);
            true
        }
        Err(message) => {
            steps.push(ForceUninstallStep::RegistryKeyRemovalFailed { message });
            false
        }
    };

    Ok(ForceUninstallResult { steps, succeeded })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Scope;

    fn program(name: &str, id: &str, install_location: Option<&str>) -> Program {
        Program {
            id: id.to_string(),
            name: name.to_string(),
            publisher: None,
            version: None,
            install_date: None,
            estimated_size_bytes: None,
            install_location: install_location.map(str::to_string),
            uninstall_string: None,
            quiet_uninstall_string: None,
            display_icon: None,
            scope: Scope::User,
            is_system_entry: false,
        }
    }

    #[test]
    fn refuses_to_force_uninstall_a_protected_component() {
        let p = program("Microsoft Edge WebView2 Runtime", r"HKCU\Whatever", None);
        let result = force_uninstall(&p);
        assert!(matches!(result, Err(ForceUninstallError::Other(_))));
    }

    #[test]
    fn requires_elevation_only_for_machine_scope() {
        let mut p = program("Foo", r"HKLM64\Foo", None);
        p.scope = Scope::Machine;
        assert!(requires_elevation(&p));

        p.scope = Scope::User;
        assert!(!requires_elevation(&p));
    }

    /// Test d'intégration : programme de test réel dans HKCU (aucun droit
    /// admin requis), avec un vrai dossier temporaire comme InstallLocation.
    /// Vérifie l'orchestration complète sans toucher au vrai registre/disque
    /// de l'utilisateur.
    #[test]
    fn force_uninstall_removes_test_registry_entry_and_temp_folder() {
        use std::fs;
        use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS};
        use winreg::RegKey;

        let key_name = "BU_force_uninstall_test_entry";
        let dir = std::env::temp_dir().join("bu_force_uninstall_test_dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("f.txt"), b"x").unwrap();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let uninstall = hkcu
            .open_subkey_with_flags(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                KEY_ALL_ACCESS,
            )
            .unwrap();
        let (test_key, _) = uninstall.create_subkey(key_name).unwrap();
        test_key.set_value("DisplayName", &"BU Force Test").unwrap();
        drop(test_key);

        let p = program(
            "BU Force Test",
            &format!(r"HKCU\{key_name}"),
            dir.to_str(),
        );

        let result = force_uninstall(&p).expect("l'orchestration ne doit pas paniquer");
        assert!(result.succeeded, "la clé de registre doit être supprimée");
        assert!(uninstall.open_subkey(key_name).is_err());
        assert!(!dir.exists(), "le dossier doit être envoyé à la corbeille");
    }
}
