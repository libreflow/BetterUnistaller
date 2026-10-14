use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Enregistrement d'une opération consignée dans le journal (F7). Le journal
/// est volontairement en texte lisible (une ligne par opération) : consultable
/// dans l'application et exportable tel quel.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    /// Horodatage ISO 8601 UTC, seconde près — assez pour un journal.
    pub timestamp: String,
    /// Nom du programme concerné.
    pub program_name: String,
    /// Type d'opération (ex. `force_uninstall`).
    pub operation: String,
    /// `true` si l'opération a atteint son objectif principal.
    pub succeeded: bool,
    /// Résumé lisible des étapes / du résultat, une phrase par étape.
    pub details: Vec<String>,
}

/// Résultat d'ajout au journal : le chemin du fichier pour l'afficher /
/// l'exporter, ou un message d'erreur. L'échec d'écriture du journal ne doit
/// JAMAIS faire échouer l'opération qui l'alimente — le journal est un
/// garde-fou annexe, pas une dépendance.
pub type JournalResult = Result<PathBuf, String>;

/// Emplacement du journal : `%LocalAppData%\BetterUnistaller\operations.log`
/// pour la version installée (la version portable, M5, le stockera à côté de
/// l'exécutable). Résolu dynamiquement à chaque écriture plutôt qu'en lazy
/// static : l'erreur reste rapportable au lieu d'être figée au premier appel.
fn journal_path() -> Result<PathBuf, String> {
    // Surcharge pour les tests (sandbox Linux sans %LOCALAPPDATA% ; les
    // tests CI Windows n'en ont pas besoin mais l'utilisent aussi pour
    // isoler leurs écritures du vrai journal de la machine).
    if let Ok(custom) = std::env::var("BU_JOURNAL_TEST_PATH") {
        let p = PathBuf::from(custom);
        if let Some(dir) = p.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("création du dossier de journal impossible : {e}"))?;
        }
        return Ok(p);
    }
    let base = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| "variable d'environnement LOCALAPPDATA introuvable".to_string())?;
    let dir = base.join("BetterUnistaller");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("création du dossier de journal impossible : {e}"))?;
    Ok(dir.join("operations.log"))
}

/// Horodatage ISO 8601 UTC à partir de l'heure système, sans dépendre d'une
/// crate de temps : calcul direct depuis l'epoch, valide jusqu'en 9999.
pub fn iso_utc_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, min, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Algorithme de conversion jours-depuis-epoch -> (y, m, d) de Howard Hinnant.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}

/// Écrit une entrée dans le journal. Une ligne JSON par opération : lisible
/// à l'œil, trivial à parser pour un futur écran "Journal" ou un export.
/// Le fichier est ouvert en append et refermé immédiatement — pas de handle
/// retenu, chaque écriture est autonome (le process peut être tué à tout
/// moment pendant une désinstallation).
pub fn record(entry: &JournalEntry) -> JournalResult {
    let path = journal_path()?;
    let line = serde_json::to_string(entry)
        .map_err(|e| format!("sérialisation de l'entrée de journal impossible : {e}"))?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("ouverture du journal impossible : {e}"))?;
    let mut w = BufWriter::new(file);
    writeln!(w, "{line}").map_err(|e| format!("écriture dans le journal impossible : {e}"))?;
    Ok(path)
}

/// Représentation lisible d'une entrée, pour un export texte (F7 : "consultable
/// dans l'application et exportable en texte").
pub fn format_entry(entry: &JournalEntry) -> String {
    let status = if entry.succeeded { "OK" } else { "ÉCHEC" };
    let details = if entry.details.is_empty() {
        String::new()
    } else {
        format!(" — {}", entry.details.join(" ; "))
    };
    format!(
        "[{}] {} : {} ({}){}",
        entry.timestamp, entry.operation, entry.program_name, status, details
    )
}

/// Lit toutes les entrées du journal, les plus récentes en tête (ordre
/// naturel d'affichage pour un écran de consultation). Tolérante : un journal
/// absent ou une ligne corrompue ne casse pas la lecture — les lignes
/// invalides sont ignorées.
pub fn read_all() -> Vec<JournalEntry> {
    let Ok(path) = journal_path() else {
        return Vec::new();
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut entries: Vec<JournalEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    entries.reverse();
    entries
}

/// Export texte complet du journal : une ligne lisible par opération, la plus
/// récente en tête. Retourne une chaîne vide si le journal est absent —
/// exporter "rien" est un cas normal (aucune opération consignée), pas une
/// erreur.
pub fn export_text() -> Result<String, String> {
    let entries = read_all();
    Ok(entries
        .iter()
        .map(format_entry)
        .collect::<Vec<_>>()
        .join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> JournalEntry {
        JournalEntry {
            timestamp: "2026-10-07T11:30:00Z".into(),
            program_name: "Test App".into(),
            operation: "force_uninstall".into(),
            succeeded: true,
            details: vec!["point de restauration créé".into()],
        }
    }

    #[test]
    fn iso_utc_now_produces_valid_iso8601() {
        let iso = iso_utc_now();
        // Format YYYY-MM-DDTHH:MM:SSZ, longueur fixe, bornes plausibles.
        assert_eq!(iso.len(), 20);
        assert!(iso.ends_with('Z'));
        assert_eq!(&iso[4..5], "-");
        assert_eq!(&iso[10..11], "T");
        let month: u32 = iso[5..7].parse().unwrap();
        assert!((1..=12).contains(&month), "mois invalide : {iso}");
        let day: u32 = iso[8..10].parse().unwrap();
        assert!((1..=31).contains(&day), "jour invalide : {iso}");
        // Cohérence avec std : l'epoch convertie en jours doit retomber sur
        // une date >= 1970-01-01.
        assert!(iso.as_str() >= "1970-01-01T00:00:00Z");
    }

    #[test]
    fn format_entry_is_human_readable() {
        let line = format_entry(&sample());
        assert!(line.starts_with("[2026-10-07T11:30:00Z] force_uninstall : Test App (OK)"));
        assert!(line.contains("point de restauration créé"));
    }

    /// Fichier journal isolé pour chaque test d'écriture (via la variable
    /// d'environnement lue par `journal_path`) : les tests ne touchent jamais
    /// le vrai journal de la machine, et tournent identiquement sur le CI
    /// Windows (%LOCALAPPDATA% existe) et sur un sandbox Linux.
    fn isolated_journal(tag: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("bu_journal_test_{tag}.log"));
        let _ = std::fs::remove_file(&path);
        std::env::set_var("BU_JOURNAL_TEST_PATH", &path);
        path
    }

    #[test]
    fn record_writes_json_line() {
        let _path = isolated_journal("record");
        let path = record(&sample()).expect("l'écriture doit réussir");
        assert!(path.to_string_lossy().contains("bu_journal_test_record.log"));
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(content.lines().last().expect("journal non vide")).unwrap();
        assert_eq!(parsed["programName"], "Test App");
        assert_eq!(parsed["operation"], "force_uninstall");
        assert_eq!(parsed["succeeded"], true);
    }

    #[test]
    fn read_all_returns_recent_entries_first() {
        let _p = isolated_journal("read_all");
        let e1 = JournalEntry {
            timestamp: "2026-10-07T10:00:00Z".into(),
            program_name: "App A".into(),
            operation: "force_uninstall".into(),
            succeeded: true,
            details: vec![],
        };
        let e2 = JournalEntry {
            timestamp: "2026-10-07T11:00:00Z".into(),
            program_name: "App B".into(),
            operation: "force_uninstall".into(),
            succeeded: false,
            details: vec![],
        };
        let _ = record(&e1);
        let _ = record(&e2);
        let all = read_all();
        assert_eq!(all.len(), 2, "les deux entrées doivent être lues");
        assert_eq!(all[0].program_name, "App B", "la plus récente en tête");
        assert_eq!(all[1].program_name, "App A");
    }

    #[test]
    fn export_text_joins_readable_lines() {
        let _p = isolated_journal("export");
        let _ = record(&sample());
        let text = export_text().expect("l'export ne doit pas échouer");
        assert!(text.contains("force_uninstall : Test App (OK)"));
        assert!(text.contains("point de restauration créé"));
    }

    #[test]
    fn read_all_on_missing_journal_is_empty() {
        let _p = isolated_journal("missing");
        // Pas de record() : le journal n'existe pas.
        assert!(read_all().is_empty());
        assert_eq!(export_text().unwrap(), "");
    }

    #[test]
    fn record_survives_missing_localappdata() {
        // LOCALAPPDATA absente ET pas de surcharge de test : l'écriture doit
        // échouer proprement (Err), pas paniquer.
        std::env::remove_var("LOCALAPPDATA");
        std::env::remove_var("BU_JOURNAL_TEST_PATH");
        let result = record(&sample());
        std::env::set_var("BU_JOURNAL_TEST_PATH", isolated_journal("noop"));
        let err = result.expect_err("LOCALAPPDATA absente doit donner Err");
        assert!(err.contains("LOCALAPPDATA"), "message inattendu : {err}");
    }
}
