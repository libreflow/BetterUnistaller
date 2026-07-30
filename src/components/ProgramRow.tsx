import type { Program } from "../lib/types";
import { formatBytes, formatDate } from "../lib/format";

export function ProgramRow({ program, selected, onSelect }: {
  program: Program;
  selected: boolean;
  onSelect: (id: string) => void;
}) {
  return (
    <tr
      onClick={() => onSelect(program.id)}
      aria-selected={selected}
      className={`cursor-pointer border-b border-neutral-200 dark:border-neutral-800 ${
        selected ? "bg-blue-50 dark:bg-blue-950" : "hover:bg-neutral-50 dark:hover:bg-neutral-900"
      }`}
    >
      <td className="px-3 py-2 font-medium">{program.name}</td>
      <td className="px-3 py-2 text-neutral-500">{program.publisher ?? "—"}</td>
      <td className="px-3 py-2 text-neutral-500">{program.version ?? "—"}</td>
      <td className="px-3 py-2 text-neutral-500">{formatDate(program.installDate)}</td>
      <td className="px-3 py-2 text-right tabular-nums">{formatBytes(program.estimatedSizeBytes)}</td>
    </tr>
  );
}
