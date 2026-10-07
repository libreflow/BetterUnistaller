/// Liste de protection F7 — "composants Windows, pilotes, runtimes
/// critiques [...] ces entrées ne sont jamais proposées à la désinstallation
/// forcée". Non exhaustive par nature (impossible de couvrir tous les
/// runtimes existants), mais couvre les cas les plus dangereux : supprimer
/// WebView2 casserait BetterUnistaller lui-même (Tauri en dépend), supprimer
/// le runtime Visual C++ ou .NET casserait un grand nombre d'autres
/// applications installées.
///
/// Comparaison insensible à la casse sur le nom affiché du programme.
const PROTECTED_NAME_KEYWORDS: &[&str] = &[
    "microsoft edge webview2",
    "microsoft visual c++",
    "microsoft .net",
    ".net runtime",
    ".net desktop runtime",
    "windows security",
    "microsoft defender",
];

/// `true` si `name` correspond à un composant protégé qui ne doit jamais
/// être proposé à la désinstallation forcée.
pub fn is_protected(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    PROTECTED_NAME_KEYWORDS
        .iter()
        .any(|kw| lower.contains(kw))
}

/// Racines système dont le contenu ne doit jamais être envoyé à la corbeille
/// ni voir ses processus tués par une désinstallation forcée. Un nom affiché
/// (éditable dans le registre) suffit à contourner la liste de mots-clés ;
/// vérifier aussi l'InstallLocation est un deuxième verrou indépendant.
const PROTECTED_LOCATION_PREFIXES: &[&str] = &[
    r"c:\windows",
    r"c:\program files",
    r"c:\program files (x86)",
    r"c:\programdata",
];

/// `true` si `install_location` pointe dans une racine système protégée.
/// Comparaison insensible à la casse, préfixe de chemin (avec séparateur :
/// `C:\Program Files\Foo` ne protège pas `C:\Program FilesFoo`).
pub fn is_protected_location(install_location: &str) -> bool {
    let lower = install_location.trim().trim_end_matches(['\\', '/']).to_ascii_lowercase();
    PROTECTED_LOCATION_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_known_critical_runtimes_case_insensitively() {
        assert!(is_protected("Microsoft Edge WebView2 Runtime"));
        assert!(is_protected("MICROSOFT VISUAL C++ 2015-2022 Redistributable (x64)"));
        assert!(is_protected("Microsoft .NET Desktop Runtime - 8.0.1"));
    }

    #[test]
    fn does_not_protect_ordinary_programs() {
        assert!(!is_protected("Adobe Creative Cloud"));
        assert!(!is_protected("7-Zip"));
        assert!(!is_protected(""));
    }

    #[test]
    fn protects_system_install_roots_case_insensitively() {
        assert!(is_protected_location(r"C:\Windows"));
        assert!(is_protected_location(r"c:\program files\Foo"));
        assert!(is_protected_location(r"C:\Program Files (x86)\Bar"));
        assert!(is_protected_location(r"C:\ProgramData\Baz "));
    }

    #[test]
    fn does_not_protect_ordinary_locations() {
        assert!(!is_protected_location(r"C:\Users\me\AppData\Local\Foo"));
        assert!(!is_protected_location(r"D:\Apps\Bar"));
        assert!(!is_protected_location(""));
    }

    #[test]
    fn protected_location_prefix_needs_separator_continuity() {
        assert!(!is_protected_location(r"C:\ProgramFilesFake\x"));
    }
}
