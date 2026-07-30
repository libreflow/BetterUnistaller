import type { Program } from "../lib/types";
import { STR } from "../lib/strings.fr";
import { ProgramRow } from "./ProgramRow";

export function ProgramList({ programs, selectedId, onSelect }: {
  programs: Program[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}) {
  if (programs.length === 0) {
    return <p className="p-6 text-neutral-500">{STR.emptyResult}</p>;
  }
  return (
    <table className="w-full text-sm">
      <thead className="sticky top-0 bg-white dark:bg-neutral-950 text-left text-neutral-400">
        <tr>
          <th className="px-3 py-2">{STR.colName}</th>
          <th className="px-3 py-2">{STR.colPublisher}</th>
          <th className="px-3 py-2">{STR.colVersion}</th>
          <th className="px-3 py-2">{STR.colDate}</th>
          <th className="px-3 py-2 text-right">{STR.colSize}</th>
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
