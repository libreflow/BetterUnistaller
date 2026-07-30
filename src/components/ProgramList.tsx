import type { SortDir, SortKey } from "../lib/search";
import type { Program } from "../lib/types";
import { STR } from "../lib/strings.fr";
import { ProgramRow } from "./ProgramRow";

function SortButton({
  label,
  active,
  dir,
  align = "start",
  onClick,
}: {
  label: string;
  active: boolean;
  dir: SortDir;
  align?: "start" | "end";
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex w-full items-center gap-1 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 ${
        align === "end" ? "justify-end" : "justify-start"
      }`}
    >
      {label}
      {active && <span aria-hidden="true">{dir === "asc" ? "▲" : "▼"}</span>}
    </button>
  );
}

export function ProgramList({
  programs,
  selectedId,
  onSelect,
  sortKey,
  sortDir,
  onSort,
}: {
  programs: Program[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  sortKey: SortKey;
  sortDir: SortDir;
  onSort: (k: SortKey) => void;
}) {
  if (programs.length === 0) {
    return <p className="p-6 text-neutral-500">{STR.emptyResult}</p>;
  }

  const ariaSortFor = (key: SortKey): "ascending" | "descending" | "none" =>
    sortKey === key ? (sortDir === "asc" ? "ascending" : "descending") : "none";

  return (
    <table className="w-full text-sm">
      <thead className="sticky top-0 bg-white dark:bg-neutral-950 text-left text-neutral-400">
        <tr>
          <th className="px-3 py-2" aria-sort={ariaSortFor("name")}>
            <SortButton label={STR.colName} active={sortKey === "name"} dir={sortDir} onClick={() => onSort("name")} />
          </th>
          <th className="px-3 py-2" aria-sort={ariaSortFor("publisher")}>
            <SortButton label={STR.colPublisher} active={sortKey === "publisher"} dir={sortDir} onClick={() => onSort("publisher")} />
          </th>
          <th className="px-3 py-2">{STR.colVersion}</th>
          <th className="px-3 py-2" aria-sort={ariaSortFor("date")}>
            <SortButton label={STR.colDate} active={sortKey === "date"} dir={sortDir} onClick={() => onSort("date")} />
          </th>
          <th className="px-3 py-2 text-right" aria-sort={ariaSortFor("size")}>
            <SortButton
              label={STR.colSize}
              active={sortKey === "size"}
              dir={sortDir}
              align="end"
              onClick={() => onSort("size")}
            />
          </th>
        </tr>
      </thead>
      <tbody>
        {programs.map((p) => (
          <ProgramRow key={p.id} program={p} selected={p.id === selectedId} onSelect={onSelect} />
        ))}
      </tbody>
    </table>
  );
}
