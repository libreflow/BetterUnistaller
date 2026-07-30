import { create } from "zustand";
import { listPrograms } from "../lib/api";
import type { SortDir, SortKey } from "../lib/search";
import type { Program } from "../lib/types";
import { STR } from "../lib/strings.fr";

interface ProgramsState {
  programs: Program[];
  loading: boolean;
  error: string | null;
  selectedId: string | null;
  query: string;
  sortKey: SortKey;
  sortDir: SortDir;
  load: () => Promise<void>;
  select: (id: string | null) => void;
  patchProgram: (id: string, patch: Partial<Program>) => void;
  setQuery: (q: string) => void;
  setSort: (key: SortKey) => void;
}

export const useProgramsStore = create<ProgramsState>((set) => ({
  programs: [],
  loading: false,
  error: null,
  selectedId: null,
  query: "",
  sortKey: "name" as SortKey,
  sortDir: "asc" as SortDir,
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
  setQuery: (query) => set({ query }),
  // Re-cliquer la même colonne inverse le sens ; nouvelle colonne : asc pour
  // les textes, desc pour taille/date (usage le plus courant).
  setSort: (key) =>
    set((s) =>
      s.sortKey === key
        ? { sortDir: s.sortDir === "asc" ? "desc" : "asc" }
        : { sortKey: key, sortDir: key === "name" || key === "publisher" ? "asc" : "desc" }
    ),
}));
