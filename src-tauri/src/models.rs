use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    Machine,
    User,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    /// Chemin registre complet de la clé — sert d'identifiant unique côté frontend.
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    /// Format ISO "YYYY-MM-DD".
    pub install_date: Option<String>,
    pub estimated_size_bytes: Option<u64>,
    pub install_location: Option<String>,
    pub uninstall_string: Option<String>,
    pub quiet_uninstall_string: Option<String>,
    /// Chemin d'icône nettoyé (exe ou ico), à résoudre par inventory::icon.
    pub display_icon: Option<String>,
    pub scope: Scope,
    /// true = masqué par défaut dans l'UI (composant système / mise à jour).
    pub is_system_entry: bool,
}
