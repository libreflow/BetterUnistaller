import { useEffect } from "react";
import { ProgramList } from "./components/ProgramList";
import { Toolbar } from "./components/Toolbar";
import { computeSizes, onSizeComputed } from "./lib/api";
import { applyFilters } from "./lib/filters";
import { compareBy, matches } from "./lib/search";
import { STR } from "./lib/strings.fr";
import { useProgramsStore } from "./store/programs";
import "./App.css";

export default function App() {
  const {
    programs,
    loading,
    error,
    selectedId,
    query,
    sortKey,
    sortDir,
    activeFilters,
    showSystem,
    load,
    select,
    setQuery,
    setSort,
    toggleFilter,
    toggleShowSystem,
  } = useProgramsStore();
  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    const un = onSizeComputed(({ id, sizeBytes }) =>
      useProgramsStore.getState().patchProgram(id, { estimatedSizeBytes: sizeBytes })
    );
    return () => {
      void un.then((f) => f());
    };
  }, []);

  const loaded = programs.length > 0;
  useEffect(() => {
    if (!loaded) return;
    const missing = useProgramsStore
      .getState()
      .programs.filter((p) => p.estimatedSizeBytes === null && p.installLocation !== null)
      .map((p) => ({ id: p.id, path: p.installLocation! }));
    if (missing.length > 0) void computeSizes(missing);
  }, [loaded]); // volontairement déclenché une seule fois après le premier chargement

  const nowIso = new Date().toISOString().slice(0, 10);
  const visible = applyFilters(
    programs.filter((p) => showSystem || !p.isSystemEntry).filter((p) => matches(p, query)),
    activeFilters,
    nowIso
  ).sort(compareBy(sortKey, sortDir));

  return (
    <main className="h-screen flex flex-col bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <header className="px-4 py-3 border-b border-neutral-200 dark:border-neutral-800">
        <h1 className="text-lg font-semibold">{STR.appTitle}</h1>
        <p className="text-sm text-neutral-500">{STR.programsCount(visible.length)}</p>
      </header>
      <Toolbar
        query={query}
        onQuery={setQuery}
        activeFilters={activeFilters}
        onToggleFilter={toggleFilter}
        showSystem={showSystem}
        onToggleShowSystem={toggleShowSystem}
      />
      <section className="flex-1 overflow-auto">
        {loading && <p className="p-6 text-neutral-500">{STR.loading}</p>}
        {error && <p className="p-6 text-red-600">{error}</p>}
        {!loading && !error && (
          <ProgramList
            programs={visible}
            selectedId={selectedId}
            onSelect={select}
            sortKey={sortKey}
            sortDir={sortDir}
            onSort={setSort}
          />
        )}
      </section>
    </main>
  );
}
