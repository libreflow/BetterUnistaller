import { create } from "zustand";
import { listPrograms } from "../lib/api";
import type { FilterId } from "../lib/filters";
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
  activeFilters: Set<FilterId>;
  showSystem: boolean;
  load: () => Promise<void>;
  select: (id: string | null) => void;
  patchProgram: (id: string, patch: Partial<Program>) => void;
  removeProgram: (id: string) => void;
  setQuery: (q: string) => void;
  setSort: (key: SortKey) => void;
  toggleFilter: (id: FilterId) => void;
  toggleShowSystem: () => void;
}

export const useProgramsStore = create<ProgramsState>((set) => ({
  programs: [],
  loading: false,
  error: null,
  selectedId: null,
  query: "",
  sortKey: "name" as SortKey,
  sortDir: "asc" as SortDir,
  activeFilters: new Set<FilterId>(),
  showSystem: false,
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
  removeProgram: (id) =>
    set((s) => ({
      programs: s.programs.filter((p) => p.id !== id),
      selectedId: s.selectedId === id ? null : s.selectedId,
    })),
  setQuery: (query) => set({ query }),
  toggleFilter: (id) =>
    set((s) => {
      const next = new Set(s.activeFilters);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return { activeFilters: next };
    }),
  toggleShowSystem: () => set((s) => ({ showSystem: !s.showSystem })),
  // Re-cliquer la même colonne inverse le sens ; nouvelle colonne : asc pour
  // les textes, desc pour taille/date (usage le plus courant).
  setSort: (key) =>
    set((s) =>
      s.sortKey === key
        ? { sortDir: s.sortDir === "asc" ? "desc" : "asc" }
        : { sortKey: key, sortDir: key === "name" || key === "publisher" ? "asc" : "desc" }
    ),
}));
