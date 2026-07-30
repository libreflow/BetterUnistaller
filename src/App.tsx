import { useEffect } from "react";
import { ProgramList } from "./components/ProgramList";
import { Toolbar } from "./components/Toolbar";
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
