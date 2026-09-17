use windows::Win32::System::Restore::{
    SRSetRestorePointW, RESTOREPOINTINFOW, RESTOREPOINTINFO_EVENT_TYPE,
    RESTOREPOINTINFO_TYPE, STATEMGRSTATUS,
};

const APPLICATION_UNINSTALL: RESTOREPOINTINFO_TYPE = RESTOREPOINTINFO_TYPE(1);
const BEGIN_SYSTEM_CHANGE: RESTOREPOINTINFO_EVENT_TYPE = RESTOREPOINTINFO_EVENT_TYPE(100);

/// Crée un point de restauration système avant une désinstallation forcée
/// (F6 — obligatoire, non désactivable). `description` apparaît dans la liste
/// des points de restauration Windows.
///
/// Limite Windows connue (cahier des charges §9) : au plus un point créé par
/// tranche de 24 h — un appel supplémentaire dans ce délai renvoie `Ok` en
/// réutilisant le point existant, ce n'est pas un échec de cette fonction.
///
/// Échoue si System Restore est désactivé, en mode sans échec, ou si le
/// process n'a pas les privilèges nécessaires (nécessite l'élévation UAC,
/// hors périmètre de ce module — voir `docs/cahier-des-charges.md` §5.3).
pub fn create_restore_point(description: &str) -> Result<(), String> {
    let mut wide: Vec<u16> = description.encode_utf16().take(255).collect();
    wide.resize(256, 0);
    let mut sz_description = [0u16; 256];
    sz_description.copy_from_slice(&wide);

    let spec = RESTOREPOINTINFOW {
        dwEventType: BEGIN_SYSTEM_CHANGE,
        dwRestorePtType: APPLICATION_UNINSTALL,
        llSequenceNumber: 0,
        szDescription: sz_description,
    };
    let mut status = STATEMGRSTATUS::default();

    let ok = unsafe { SRSetRestorePointW(&spec, &mut status) };
    if ok.as_bool() {
        Ok(())
    } else {
        // status est un struct packed(1) : copier le champ dans une locale
        // avant usage, une référence directe sur un champ non aligné est UB.
        let code = status.nStatus.0;
        Err(format!(
            "création du point de restauration impossible (code {code})"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test d'intégration : appelle la vraie API System Restore. Ne peut pas
    /// asserter un succès garanti (System Restore peut être désactivé sur la
    /// machine CI/dev, ou la limite d'un point par 24h peut déjà être
    /// atteinte) — vérifie seulement que l'appel ne panique pas et renvoie
    /// une réponse structurée dans les deux cas.
    #[test]
    fn create_restore_point_returns_a_structured_result() {
        let result = create_restore_point("BetterUnistaller test — sans effet destructif");
        match result {
            Ok(()) => {}
            Err(msg) => assert!(!msg.is_empty()),
        }
    }
}
