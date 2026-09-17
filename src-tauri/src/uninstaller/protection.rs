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
}
