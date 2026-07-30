import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
