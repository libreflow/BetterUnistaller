import type { ForceUninstallOutcome } from "./api";
import { STR } from "./strings.fr";
import { useProgramsStore } from "../store/programs";

/// Traite le résultat d'une désinstallation forcée (immédiat ou après reprise
/// post-relance élevée) : retire le programme de la liste si réussi, pousse
/// un message de feedback dans tous les cas. Centralisé ici pour que
/// `DetailPanel` (déclenchement direct) et `App` (reprise après relance
/// élevée) affichent exactement le même comportement.
export function applyOutcome(
  outcome: ForceUninstallOutcome,
  programId: string | null,
  setFeedback: (message: string) => void
): void {
  switch (outcome.outcome) {
    case "completed":
      if (outcome.result.succeeded) {
        const warnings = outcome.result.steps
          .filter((s) => s.step === "restorePointFailed" || s.step === "installLocationRemovalFailed")
          .map((s) => (s.step === "restorePointFailed" ? STR.restorePointFailed : STR.locationRemovalFailed))
          .join(" ");
        const message = warnings ? `${STR.forceUninstallSuccess} ${warnings}` : STR.forceUninstallSuccess;
        setFeedback(message);
        if (programId) useProgramsStore.getState().removeProgram(programId);
      } else {
        const failure = outcome.result.steps.find((s) => s.step === "registryKeyRemovalFailed");
        setFeedback(
          failure && failure.step === "registryKeyRemovalFailed"
            ? `${STR.forceUninstallFailure} ${failure.message}`
            : STR.forceUninstallFailure
        );
      }
      return;
    case "elevationRequested":
      // L'app va se fermer côté backend (app.exit) juste après avoir renvoyé
      // cette réponse — ce message n'a en pratique presque jamais le temps
      // de s'afficher, mais reste correct si la fermeture est retardée.
      setFeedback(STR.forceUninstallElevationRequested);
      return;
    case "failed":
      setFeedback(outcome.message || STR.forceUninstallFailure);
  }
}
