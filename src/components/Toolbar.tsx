import { STR } from "../lib/strings.fr";

export function Toolbar({ query, onQuery }: { query: string; onQuery: (q: string) => void }) {
  return (
    <div className="px-4 py-2 border-b border-neutral-200 dark:border-neutral-800">
      <input
        type="search"
        value={query}
        onChange={(e) => onQuery(e.target.value)}
        placeholder={STR.searchPlaceholder}
        className="w-full max-w-md rounded-md border border-neutral-300 dark:border-neutral-700 bg-transparent px-3 py-1.5 text-sm outline-none focus:border-blue-500"
      />
    </div>
  );
}
