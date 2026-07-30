#[derive(Debug, PartialEq, Eq)]
pub enum EntryVisibility {
    /// Pas un programme (pas de DisplayName) : ne pas retourner du tout.
    Skip,
    /// Programme système / mise à jour : retourné avec is_system_entry = true.
    Hidden,
    Visible,
}

/// InstallDate registre "YYYYMMDD" -> ISO "YYYY-MM-DD".
pub fn parse_install_date(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.len() != 8 || !raw.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let month: u32 = raw[4..6].parse().ok()?;
    let day: u32 = raw[6..8].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(format!("{}-{}-{}", &raw[0..4], &raw[4..6], &raw[6..8]))
}

/// EstimatedSize (REG_DWORD) est exprimé en Kio.
pub fn estimated_size_to_bytes(kib: u32) -> u64 {
    (kib as u64) * 1024
}

pub fn classify_visibility(
    display_name: Option<&str>,
    system_component: u32,
    parent_key_name: Option<&str>,
    release_type: Option<&str>,
) -> EntryVisibility {
    match display_name {
        None | Some("") => return EntryVisibility::Skip,
        _ => {}
    }
    if system_component == 1 || parent_key_name.is_some() {
        return EntryVisibility::Hidden;
    }
    if let Some(rt) = release_type {
        if rt.to_ascii_lowercase().contains("update") {
            return EntryVisibility::Hidden;
        }
    }
    EntryVisibility::Visible
}

/// DisplayIcon: `"C:\...\app.exe",0` -> `C:\...\app.exe`
pub fn clean_icon_path(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let no_index = match raw.rfind(',') {
        Some(pos) if raw[pos + 1..].trim().parse::<i32>().is_ok() => &raw[..pos],
        _ => raw,
    };
    let cleaned = no_index.trim().trim_matches('"').trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_install_date_yyyymmdd() {
        assert_eq!(parse_install_date("20260731"), Some("2026-07-31".into()));
    }

    #[test]
    fn parse_install_date_rejects_garbage() {
        assert_eq!(parse_install_date(""), None);
        assert_eq!(parse_install_date("2026"), None);
        assert_eq!(parse_install_date("2026-07-31"), None); // déjà formatée = inattendu
        assert_eq!(parse_install_date("abcdefgh"), None);
        assert_eq!(parse_install_date("20261332"), None); // mois 13, jour 32
    }

    #[test]
    fn estimated_size_is_kb_to_bytes() {
        // EstimatedSize est en Kio dans le registre
        assert_eq!(estimated_size_to_bytes(1024), 1_048_576);
        assert_eq!(estimated_size_to_bytes(0), 0);
    }

    #[test]
    fn entry_without_display_name_is_skipped() {
        let v = classify_visibility(None, 0, None, None);
        assert_eq!(v, EntryVisibility::Skip);
    }

    #[test]
    fn system_component_is_hidden() {
        let v = classify_visibility(Some("Runtime X"), 1, None, None);
        assert_eq!(v, EntryVisibility::Hidden);
    }

    #[test]
    fn windows_update_entries_are_hidden() {
        assert_eq!(classify_visibility(Some("KB123"), 0, Some("OperatingSystem"), None), EntryVisibility::Hidden);
        assert_eq!(classify_visibility(Some("Hotfix"), 0, None, Some("Security Update")), EntryVisibility::Hidden);
        assert_eq!(classify_visibility(Some("Hotfix"), 0, None, Some("Update Rollup")), EntryVisibility::Hidden);
    }

    #[test]
    fn normal_program_is_visible() {
        assert_eq!(classify_visibility(Some("7-Zip"), 0, None, None), EntryVisibility::Visible);
    }

    #[test]
    fn clean_icon_path_strips_quotes_and_index() {
        assert_eq!(
            clean_icon_path(r#""C:\Apps\x\app.exe",0"#),
            Some(r"C:\Apps\x\app.exe".into())
        );
        assert_eq!(clean_icon_path(r"C:\Apps\x\app.ico"), Some(r"C:\Apps\x\app.ico".into()));
        assert_eq!(clean_icon_path(""), None);
    }
}
