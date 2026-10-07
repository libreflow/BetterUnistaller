use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, TerminateProcess, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};

/// Un processus en cours d'exécution, tel qu'énuméré par Toolhelp.
struct RunningProcess {
    pid: u32,
    exe_path: Option<String>,
}


/// Chemin complet de l'exécutable d'un processus, via son PID. `None` si le
/// processus n'existe plus ou si l'accès est refusé (processus système,
/// process d'un autre utilisateur) — cas tolérés, pas des erreurs bloquantes.
fn query_full_path(pid: u32) -> Option<String> {
    unsafe {
        let handle: HANDLE =
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        ok.ok()?;
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

/// Énumère tous les processus du système via l'API Toolhelp. Ne retourne
/// jamais d'erreur bloquante : un échec d'ouverture du snapshot donne une
/// liste vide plutôt que de faire échouer toute la désinstallation forcée
/// pour une cause annexe.
fn list_running_processes() -> Vec<RunningProcess> {
    let mut out = Vec::new();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return out;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                out.push(RunningProcess {
                    pid: entry.th32ProcessID,
                    exe_path: query_full_path(entry.th32ProcessID),
                });
                entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    out
}

/// Termine tous les processus dont l'exécutable se trouve sous
/// `install_location` (comparaison insensible à la casse, avec séparateur —
/// évite qu'`InstallLocation = C:\Apps\Foo` matche à tort `C:\Apps\FooBar`).
/// Retourne le nombre de processus effectivement terminés. Les échecs
/// individuels (accès refusé, processus déjà terminé) sont ignorés : ce
/// n'est pas à la désinstallation forcée d'échouer entièrement pour un seul
/// processus récalcitrant, le scan de résidus F4 rattrapera ce qui reste.
pub fn kill_processes_under(install_location: &str) -> u32 {
    let root = normalize_root(install_location);
    if root.is_empty() {
        return 0;
    }
    let mut killed = 0u32;
    for proc in list_running_processes() {
        let Some(path) = &proc.exe_path else { continue };
        if !path_is_under(path, &root) {
            continue;
        }
        if terminate_pid(proc.pid) {
            killed += 1;
        }
    }
    killed
}

fn normalize_root(path: &str) -> String {
    let trimmed = path.trim().trim_end_matches(['\\', '/']);
    trimmed.to_ascii_lowercase()
}

fn path_is_under(candidate: &str, root_lower: &str) -> bool {
    let candidate_lower = candidate.to_ascii_lowercase();
    let candidate_path = Path::new(&candidate_lower);
    let root_path = Path::new(root_lower);
    candidate_path.starts_with(root_path)
}

fn terminate_pid(pid: u32) -> bool {
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) else {
            return false;
        };
        let ok = TerminateProcess(handle, 1).is_ok();
        let _ = CloseHandle(handle);
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_under_matches_subdirectory_case_insensitive() {
        assert!(path_is_under(
            r"C:\Apps\Foo\bin\app.exe",
            &normalize_root(r"c:\apps\foo")
        ));
    }

    #[test]
    fn path_is_under_rejects_sibling_with_shared_prefix() {
        // "C:\Apps\FooBar" ne doit jamais matcher la racine "C:\Apps\Foo" —
        // sinon on tuerait le processus d'un autre programme (FooBar).
        assert!(!path_is_under(
            r"C:\Apps\FooBar\app.exe",
            &normalize_root(r"C:\Apps\Foo")
        ));
    }

    #[test]
    fn path_is_under_rejects_unrelated_path() {
        assert!(!path_is_under(
            r"C:\Windows\System32\notepad.exe",
            &normalize_root(r"C:\Apps\Foo")
        ));
    }

    #[test]
    fn kill_processes_under_empty_location_is_noop() {
        assert_eq!(kill_processes_under(""), 0);
        assert_eq!(kill_processes_under("   "), 0);
    }

    #[test]
    fn kill_processes_under_nonexistent_location_kills_nothing() {
        // Aucun processus réel ne tourne sous ce chemin inventé — la fonction
        // doit parcourir tous les processus système sans erreur et retourner 0.
        assert_eq!(kill_processes_under(r"C:\bu_inexistant_xyz_test_root"), 0);
    }
}
