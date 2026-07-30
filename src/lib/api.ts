import { invoke } from "@tauri-apps/api/core";
import type { Program } from "./types";

export function listPrograms(): Promise<Program[]> {
  return invoke<Program[]>("list_programs");
}
