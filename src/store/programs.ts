import { create } from "zustand";
import { listPrograms } from "../lib/api";
import type { Program } from "../lib/types";
import { STR } from "../lib/strings.fr";

interface ProgramsState {
  programs: Program[];
  loading: boolean;
  error: string | null;
  selectedId: string | null;
  load: () => Promise<void>;
  select: (id: string | null) => void;
  patchProgram: (id: string, patch: Partial<Program>) => void;
}

export const useProgramsStore = create<ProgramsState>((set) => ({
  programs: [],
  loading: false,
  error: null,
  selectedId: null,
  load: async () => {
    set({ loading: true, error: null });
    try {
      set({ programs: await listPrograms(), loading: false });
    } catch {
      set({ error: STR.loadError, loading: false });
    }
  },
  select: (id) => set({ selectedId: id }),
  patchProgram: (id, patch) =>
    set((s) => ({
      programs: s.programs.map((p) => (p.id === id ? { ...p, ...patch } : p)),
    })),
}));
