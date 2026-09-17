import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import type { Program } from "./types";

export function listPrograms(): Promise<Program[]> {
  return invoke<Program[]>("list_programs");
}

export interface SizeRequest {
  id: string;
  path: string;
}
export interface SizeComputed {
  id: string;
  sizeBytes: number;
}

export function computeSizes(requests: SizeRequest[]): Promise<void> {
  return invoke("compute_sizes", { requests });
}

export function onSizeComputed(cb: (e: SizeComputed) => void): Promise<UnlistenFn> {
  return listen<SizeComputed>("size-computed", (event) => cb(event.payload));
}

export interface IconRequest {
  id: string;
  path: string;
}
export interface IconReady {
  id: string;
  dataUri: string;
}

export function loadIcons(requests: IconRequest[]): Promise<void> {
  return invoke("load_icons", { requests });
}

export function onIconReady(cb: (e: IconReady) => void): Promise<UnlistenFn> {
  return listen<IconReady>("icon-ready", (event) => cb(event.payload));
}

export function openFolder(path: string): Promise<void> {
  return revealItemInDir(path);
}

export type ForceUninstallStep =
  | { step: "restorePointCreated" }
  | { step: "restorePointFailed"; message: string }
  | { step: "processesTerminated"; count: number }
  | { step: "installLocationRemoved" }
  | { step: "installLocationRemovalFailed"; message: string }
  | { step: "registryKeyRemoved" }
  | { step: "registryKeyRemovalFailed"; message: string };

export interface ForceUninstallResult {
  steps: ForceUninstallStep[];
  succeeded: boolean;
}

export type ForceUninstallOutcome =
  | { outcome: "completed"; result: ForceUninstallResult }
  | { outcome: "elevationRequested" }
  | { outcome: "failed"; message: string };

/// Désinstallation forcée (F6) — destructive, réservée aux cas où la
/// désinstallation standard a échoué ou est absente. L'appelant doit avoir
/// obtenu une confirmation explicite de l'utilisateur au préalable. Si le
/// programme est machine-wide et nécessite l'élévation UAC, l'app se ferme
/// pour relancer une instance élevée qui termine l'opération — l'appelant
/// reçoit alors `{ outcome: "elevationRequested" }`.
export function forceUninstall(program: Program): Promise<ForceUninstallOutcome> {
  return invoke<ForceUninstallOutcome>("force_uninstall", { program });
}

/// À appeler une fois au démarrage : récupère le résultat d'une
/// désinstallation forcée qui a motivé une relance élevée avant que
/// l'utilisateur ne revoie l'interface. `null` si l'instance courante n'a pas
/// été lancée pour reprendre une opération en attente.
export function takePendingUninstallOutcome(): Promise<ForceUninstallOutcome | null> {
  return invoke<ForceUninstallOutcome | null>("take_pending_uninstall_outcome");
}
