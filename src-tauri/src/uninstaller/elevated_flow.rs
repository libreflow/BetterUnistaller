use std::sync::Mutex;

use serde::Serialize;

use crate::models::Program;
use crate::uninstaller::elevation;
use crate::uninstaller::force::{self, ForceUninstallError, ForceUninstallResult};

/// Résultat exposé au frontend pour une désinstallation forcée déclenchée
/// depuis l'UI. Distinct de `ForceUninstallResult` : ajoute le cas
/// "élévation demandée" (l'app va se fermer pour laisser place à l'instance
/// élevée) qui n'est pas un échec à afficher comme tel.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "outcome")]
pub enum ForceUninstallOutcome {
    Completed { result: ForceUninstallResult },
    ElevationRequested,
    Failed { message: String },
}

/// État partagé entre l'instance non élevée (qui relance) et l'instance
/// élevée (qui exécute la désinstallation en attente au démarrage, avant que
/// l'utilisateur ne revoie l'interface). Le frontend de l'instance élevée
/// interroge `take_pending_uninstall_outcome` une fois chargé pour afficher
/// le résultat de l'opération qui a motivé la relance.
#[derive(Default)]
pub struct PendingUninstallState(pub Mutex<Option<ForceUninstallOutcome>>);

/// Point d'entrée de la commande Tauri `force_uninstall` : exécute la
/// désinstallation forcée si le process courant a déjà les privilèges
/// requis, sinon demande l'élévation (relance + fermeture de l'instance
/// courante) plutôt que d'échouer sèchement — cahier des charges §5.3.
pub fn force_uninstall_or_request_elevation(
    app: &tauri::AppHandle,
    program_id: &str,
) -> ForceUninstallOutcome {
    let program = match lookup_program(program_id) {
        Ok(program) => program,
        Err(message) => return ForceUninstallOutcome::Failed { message },
    };
    match force::force_uninstall(&program) {
        Ok(result) => ForceUninstallOutcome::Completed { result },
        Err(ForceUninstallError::ElevationRequired) => {
            match elevation::relaunch_elevated(&elevation::pending_uninstall_args(&program.id)) {
                Ok(()) => {
                    // Diffère la fermeture de l'app : la réponse IPC doit
                    // avoir le temps d'atteindre le frontend avant que le
                    // process ne disparaisse, sinon la promesse `invoke`
                    // reste pendante et l'UI reste bloquée sur « en cours ».
                    // Un court délai suffit, et si le process se ferme
                    // avant la fin du délai, le résultat est le même que
                    // le `app.exit(0)` immédiat d'origine.
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        app.exit(0);
                    });
                    ForceUninstallOutcome::ElevationRequested
                }
                Err(message) => ForceUninstallOutcome::Failed { message },
            }
        }
        Err(ForceUninstallError::Other(message)) => ForceUninstallOutcome::Failed { message },
    }
}

/// Exécuté au tout début du démarrage (avant l'affichage de la fenêtre) si
/// l'instance a été relancée avec élévation pour terminer une désinstallation
/// forcée en attente. Cherche le programme correspondant dans le registre
/// (l'id seul a traversé la relance, pas l'objet `Program` complet) et
/// exécute l'opération, dont le résultat est stocké pour être récupéré par
/// le frontend une fois l'interface chargée.
pub fn run_pending_uninstall_if_any(args: &[String]) -> Option<ForceUninstallOutcome> {
    let id = elevation::extract_pending_uninstall_id(args)?;
    let program = match lookup_program(&id) {
        Ok(program) => program,
        Err(message) => return Some(ForceUninstallOutcome::Failed { message }),
    };
    Some(match force::force_uninstall(&program) {
        Ok(result) => ForceUninstallOutcome::Completed { result },
        Err(err) => ForceUninstallOutcome::Failed {
            message: err.to_string(),
        },
    })
}

/// Récupère le programme à partir de son id registre uniquement — les
/// autres champs (nom, InstallLocation, scope) sont relus depuis le registre,
/// jamais pris depuis le frontend : celui-ci ne peut ainsi pas faire
/// exécuter une suppression arbitraire en forgéant un objet `Program`.
pub fn lookup_program(program_id: &str) -> Result<Program, String> {
    crate::inventory::registry::read_installed_programs()
        .into_iter()
        .find(|p| p.id == program_id)
        .ok_or_else(|| format!("programme introuvable dans le registre : {program_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_pending_uninstall_if_any_returns_none_without_flag() {
        let args = vec!["betterunistaller.exe".to_string()];
        assert!(run_pending_uninstall_if_any(&args).is_none());
    }

    #[test]
    fn run_pending_uninstall_if_any_reports_missing_program() {
        let args = vec![
            "betterunistaller.exe".to_string(),
            elevation::PENDING_FORCE_UNINSTALL_FLAG.to_string(),
            r"HKLM64\BU_definitely_not_installed_xyz".to_string(),
        ];
        let outcome = run_pending_uninstall_if_any(&args).expect("un id était présent");
        assert!(matches!(outcome, ForceUninstallOutcome::Failed { .. }));
    }
}
