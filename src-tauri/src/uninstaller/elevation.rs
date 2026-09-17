use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Argument CLI utilisé pour la reprise de contexte après relance élevée
/// (cahier des charges §5.3 : "relance élevée de l'application sur demande,
/// avec reprise du contexte — opération en attente sérialisée"). L'instance
/// élevée relit cet argument au démarrage et exécute l'opération en attente
/// avant d'afficher l'interface normale.
pub const PENDING_FORCE_UNINSTALL_FLAG: &str = "--force-uninstall-id";

/// `true` si le process courant tourne avec un jeton élevé (UAC). Les
/// suppressions HKLM/Program Files et la création de point de restauration
/// exigent l'élévation (cahier des charges §5.3) ; `false` en cas d'échec de
/// lecture du jeton — on suppose alors le cas le plus restrictif (non élevé)
/// plutôt que de risquer un faux positif qui laisserait croire à tort qu'une
/// opération privilégiée va réussir.
pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        );
        let _ = CloseHandle(token);
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// Construit la ligne d'arguments à passer à l'instance relancée pour
/// qu'elle reprenne directement la désinstallation forcée en attente.
/// Fonction pure séparée de `relaunch_elevated` pour rester testable sans
/// déclencher une vraie invite UAC.
pub fn pending_uninstall_args(program_id: &str) -> Vec<String> {
    vec![PENDING_FORCE_UNINSTALL_FLAG.to_string(), program_id.to_string()]
}

/// Cherche `--force-uninstall-id <id>` dans les arguments de lancement du
/// process (repris après une relance élevée). `None` si absent ou mal formé.
pub fn extract_pending_uninstall_id(args: &[String]) -> Option<String> {
    let pos = args.iter().position(|a| a == PENDING_FORCE_UNINSTALL_FLAG)?;
    args.get(pos + 1).cloned()
}

fn encode_wide_nul(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Relance l'exécutable courant avec élévation UAC (verbe `runas`), en lui
/// passant les arguments de reprise de contexte, puis termine le process non
/// élevé courant. C'est à l'appelant (commande Tauri) de quitter l'app juste
/// après un `Ok` — cette fonction ne le fait pas elle-même pour rester
/// testable sans effet de bord fatal.
///
/// Erreurs attendues et normales, pas des bugs : l'utilisateur clique
/// « Non » sur l'invite UAC (ERROR_CANCELLED), ou refuse l'élévation d'une
/// autre façon.
pub fn relaunch_elevated(args: &[String]) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("chemin de l'exécutable introuvable : {e}"))?;
    let exe_wide = encode_wide_nul(exe.to_string_lossy().as_ref());
    let params = args.join(" ");
    let params_wide = encode_wide_nul(&params);
    let verb_wide = encode_wide_nul("runas");

    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb_wide.as_ptr()),
        lpFile: PCWSTR(exe_wide.as_ptr()),
        lpParameters: PCWSTR(params_wide.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    unsafe {
        ShellExecuteExW(&mut info)
            .map_err(|e| format!("relance élevée refusée ou annulée : {e}"))?;
        if !info.hProcess.is_invalid() {
            let _ = CloseHandle(info.hProcess);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_uninstall_args_round_trip_through_extract() {
        let args = pending_uninstall_args(r"HKLM64\SomeApp");
        // Simule argv complet : [exe_path, ...args].
        let mut argv = vec!["betterunistaller.exe".to_string()];
        argv.extend(args);
        assert_eq!(
            extract_pending_uninstall_id(&argv),
            Some(r"HKLM64\SomeApp".to_string())
        );
    }

    #[test]
    fn extract_pending_uninstall_id_returns_none_when_flag_absent() {
        let argv = vec!["betterunistaller.exe".to_string()];
        assert_eq!(extract_pending_uninstall_id(&argv), None);
    }

    #[test]
    fn extract_pending_uninstall_id_returns_none_when_value_missing() {
        let argv = vec![
            "betterunistaller.exe".to_string(),
            PENDING_FORCE_UNINSTALL_FLAG.to_string(),
        ];
        assert_eq!(extract_pending_uninstall_id(&argv), None);
    }

    /// Test d'intégration : lit le vrai jeton du process de test. N'affirme
    /// pas une valeur précise (dépend de la machine CI/dev), seulement que
    /// l'appel ne panique pas et renvoie un booléen exploitable.
    #[test]
    fn is_elevated_does_not_panic_on_real_token() {
        let _ = is_elevated();
    }
}
