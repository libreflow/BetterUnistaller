use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, KEY_READ, KEY_WOW64_32KEY,
    KEY_WOW64_64KEY,
};
use winreg::RegKey;

use crate::inventory::parse::{
    classify_visibility, clean_icon_path, estimated_size_to_bytes, parse_install_date, EntryVisibility,
};
use crate::models::{Program, Scope};

const UNINSTALL_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";

/// Lit les 3 sources du cahier des charges (F1) :
/// HKLM 64 bits, HKLM 32 bits (WOW64), HKCU.
pub fn read_installed_programs() -> Vec<Program> {
    let mut out = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    read_hive(&hklm, KEY_WOW64_64KEY, Scope::Machine, "HKLM64", &mut out);
    read_hive(&hklm, KEY_WOW64_32KEY, Scope::Machine, "HKLM32", &mut out);
    read_hive(&hkcu, 0, Scope::User, "HKCU", &mut out);
    out
}

fn read_hive(root: &RegKey, wow_flag: u32, scope: Scope, id_prefix: &str, out: &mut Vec<Program>) {
    let Ok(uninstall) = root.open_subkey_with_flags(UNINSTALL_PATH, KEY_READ | wow_flag) else {
        return;
    };
    for key_name in uninstall.enum_keys().flatten() {
        let Ok(sub) = uninstall.open_subkey_with_flags(&key_name, KEY_READ | wow_flag) else {
            continue;
        };
        if let Some(program) = read_entry(&sub, &key_name, scope, id_prefix) {
            out.push(program);
        }
    }
}

fn read_entry(key: &RegKey, key_name: &str, scope: Scope, id_prefix: &str) -> Option<Program> {
    let display_name: Option<String> = key.get_value("DisplayName").ok();
    let system_component: u32 = key.get_value("SystemComponent").unwrap_or(0);
    let parent_key_name: Option<String> = key.get_value("ParentKeyName").ok();
    let release_type: Option<String> = key.get_value("ReleaseType").ok();

    let visibility = classify_visibility(
        display_name.as_deref(),
        system_component,
        parent_key_name.as_deref(),
        release_type.as_deref(),
    );
    if visibility == EntryVisibility::Skip {
        return None;
    }

    let estimated_size_kib: Option<u32> = key.get_value("EstimatedSize").ok();
    let install_date_raw: Option<String> = key.get_value("InstallDate").ok();
    let display_icon_raw: Option<String> = key.get_value("DisplayIcon").ok();

    let non_empty = |s: String| if s.trim().is_empty() { None } else { Some(s) };

    let Some(name) = display_name else {
        // Invariant garanti par classify_visibility (Skip déjà filtré ci-dessus) ;
        // retour défensif au lieu d'un panic si l'invariant est un jour cassé ailleurs.
        return None;
    };

    Some(Program {
        id: format!("{id_prefix}\\{key_name}"),
        name,
        publisher: key.get_value::<String, _>("Publisher").ok().and_then(non_empty),
        version: key.get_value::<String, _>("DisplayVersion").ok().and_then(non_empty),
        install_date: install_date_raw.as_deref().and_then(parse_install_date),
        estimated_size_bytes: estimated_size_kib.map(estimated_size_to_bytes),
        install_location: key.get_value::<String, _>("InstallLocation").ok().and_then(non_empty),
        uninstall_string: key.get_value::<String, _>("UninstallString").ok().and_then(non_empty),
        quiet_uninstall_string: key.get_value::<String, _>("QuietUninstallString").ok().and_then(non_empty),
        display_icon: display_icon_raw.as_deref().and_then(clean_icon_path),
        scope,
        is_system_entry: visibility == EntryVisibility::Hidden,
    })
}

/// Éclate un `Program::id` (`"HKLM64\\{key}"`, `"HKLM32\\{key}"`, `"HKCU\\{key}"`)
/// produit par `read_hive` en (racine, flag WOW64, nom de sous-clé). Erreur si
/// le préfixe est inconnu — protège `delete_uninstall_key` contre un id
/// mal formé ou forgé côté frontend.
fn resolve_id(id: &str) -> Result<(RegKey, u32, &str), String> {
    let (prefix, key_name) = id
        .split_once('\\')
        .ok_or_else(|| format!("id de programme invalide : {id}"))?;
    match prefix {
        "HKLM64" => Ok((RegKey::predef(HKEY_LOCAL_MACHINE), KEY_WOW64_64KEY, key_name)),
        "HKLM32" => Ok((RegKey::predef(HKEY_LOCAL_MACHINE), KEY_WOW64_32KEY, key_name)),
        "HKCU" => Ok((RegKey::predef(HKEY_CURRENT_USER), 0, key_name)),
        _ => Err(format!("id de programme invalide : {id}")),
    }
}

/// Supprime la clé `Uninstall\{key_name}` correspondant à `id` (F6 —
/// désinstallation forcée : suppression de la clé Uninstall après arrêt des
/// processus et suppression de l'InstallLocation). Idempotent : une clé déjà
/// absente n'est pas une erreur.
pub fn delete_uninstall_key(id: &str) -> Result<(), String> {
    let (root, wow_flag, key_name) = resolve_id(id)?;
    let uninstall = root
        .open_subkey_with_flags(UNINSTALL_PATH, KEY_ALL_ACCESS | wow_flag)
        .map_err(|e| format!("ouverture de la clé Uninstall impossible : {e}"))?;
    match uninstall.delete_subkey_all(key_name) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("suppression de la clé de registre impossible : {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test d'intégration : lit le vrai registre de la machine de dev/CI Windows.
    #[test]
    fn reads_at_least_one_visible_program_from_real_registry() {
        let programs = read_installed_programs();
        assert!(
            programs.iter().any(|p| !p.is_system_entry),
            "aucun programme visible trouvé — inattendu sur une machine Windows réelle"
        );
        // Tous les programmes retournés ont un nom non vide et un id unique.
        assert!(programs.iter().all(|p| !p.name.is_empty()));
        let mut ids: Vec<&str> = programs.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "ids en double");
    }

    #[test]
    fn resolve_id_parses_known_prefixes() {
        let (_, wow, name) = resolve_id(r"HKLM64\Foo").unwrap();
        assert_eq!((wow, name), (KEY_WOW64_64KEY, "Foo"));
        let (_, wow, name) = resolve_id(r"HKLM32\Bar").unwrap();
        assert_eq!((wow, name), (KEY_WOW64_32KEY, "Bar"));
        let (_, wow, name) = resolve_id(r"HKCU\Baz").unwrap();
        assert_eq!((wow, name), (0, "Baz"));
    }

    #[test]
    fn resolve_id_rejects_unknown_prefix_or_missing_separator() {
        assert!(resolve_id("Unknown\\Key").is_err());
        assert!(resolve_id("NoSeparator").is_err());
    }

    /// Écrit une fausse entrée Uninstall dans HKCU (aucun droit admin requis),
    /// vérifie que delete_uninstall_key la supprime, et que rappeler la
    /// fonction sur une clé déjà absente reste un succès (idempotence).
    #[test]
    fn delete_uninstall_key_removes_hkcu_test_entry_and_is_idempotent() {
        let key_name = "BU_delete_test_entry";
        let id = format!(r"HKCU\{key_name}");

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let uninstall = hkcu
            .open_subkey_with_flags(UNINSTALL_PATH, KEY_ALL_ACCESS)
            .expect("HKCU\\...\\Uninstall doit exister sur toute machine Windows");
        let (test_key, _) = uninstall.create_subkey(key_name).unwrap();
        test_key.set_value("DisplayName", &"BU Test Entry").unwrap();
        drop(test_key);

        assert!(uninstall.open_subkey(key_name).is_ok());

        delete_uninstall_key(&id).expect("suppression doit réussir");
        assert!(uninstall.open_subkey(key_name).is_err());

        // Idempotence : la clé n'existe déjà plus.
        delete_uninstall_key(&id).expect("suppression d'une clé absente doit rester Ok");
    }

    #[test]
    fn delete_uninstall_key_rejects_invalid_id() {
        assert!(delete_uninstall_key("garbage").is_err());
    }
}
