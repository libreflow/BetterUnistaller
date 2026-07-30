import type { FilterId } from "../lib/filters";
import { STR } from "../lib/strings.fr";

const FILTER_CHIPS: { id: FilterId; label: string }[] = [
  { id: "large", label: STR.filterLarge },
  { id: "recent", label: STR.filterRecent },
  { id: "noPublisher", label: STR.filterNoPublisher },
  { id: "userScope", label: STR.filterScopeUser },
];

function FilterChip({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: () => void;
}) {
  return (
    <label
      className={`inline-flex cursor-pointer items-center gap-1.5 rounded-full border px-3 py-1 text-xs transition-colors ${
        checked
          ? "border-blue-500 bg-blue-50 text-blue-700 dark:bg-blue-950 dark:text-blue-300"
          : "border-neutral-300 text-neutral-600 hover:border-neutral-400 dark:border-neutral-700 dark:text-neutral-400 dark:hover:border-neutral-600"
      }`}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={onChange}
        className="sr-only"
      />
      {label}
    </label>
  );
}

export function Toolbar({
  query,
  onQuery,
  activeFilters,
  onToggleFilter,
  showSystem,
  onToggleShowSystem,
}: {
  query: string;
  onQuery: (q: string) => void;
  activeFilters: Set<FilterId>;
  onToggleFilter: (id: FilterId) => void;
  showSystem: boolean;
  onToggleShowSystem: () => void;
}) {
  return (
    <div className="px-4 py-2 border-b border-neutral-200 dark:border-neutral-800 flex flex-col gap-2">
      <input
        type="search"
        value={query}
        onChange={(e) => onQuery(e.target.value)}
        placeholder={STR.searchPlaceholder}
        className="w-full max-w-md rounded-md border border-neutral-300 dark:border-neutral-700 bg-transparent px-3 py-1.5 text-sm outline-none focus:border-blue-500"
      />
      <div className="flex flex-wrap items-center gap-2">
        {FILTER_CHIPS.map(({ id, label }) => (
          <FilterChip
            key={id}
            label={label}
            checked={activeFilters.has(id)}
            onChange={() => onToggleFilter(id)}
          />
        ))}
        <label className="ml-auto inline-flex items-center gap-1.5 text-xs text-neutral-600 dark:text-neutral-400">
          <input
            type="checkbox"
            checked={showSystem}
            onChange={onToggleShowSystem}
            className="h-3.5 w-3.5 rounded border-neutral-300 dark:border-neutral-700"
          />
          {STR.showSystem}
        </label>
      </div>
    </div>
  );
}
