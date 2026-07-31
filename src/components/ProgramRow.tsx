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
      <td className="px-3 py-2 font-medium">
        <span className="inline-flex items-center gap-2">
          {program.iconDataUri ? (
            <img src={program.iconDataUri} alt="" className="h-5 w-5" />
          ) : (
            <span className="h-5 w-5 rounded bg-neutral-200 dark:bg-neutral-700 text-[10px] font-bold inline-flex items-center justify-center text-neutral-600 dark:text-neutral-300">
              {program.name.charAt(0).toUpperCase()}
            </span>
          )}
          {program.name}
        </span>
      </td>
      <td className="px-3 py-2 text-neutral-500">{program.publisher ?? "—"}</td>
      <td className="px-3 py-2 text-neutral-500">{program.version ?? "—"}</td>
      <td className="px-3 py-2 text-neutral-500">{formatDate(program.installDate)}</td>
      <td className="px-3 py-2 text-right tabular-nums">{formatBytes(program.estimatedSizeBytes)}</td>
    </tr>
  );
}
