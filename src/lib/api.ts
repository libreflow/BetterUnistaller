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
