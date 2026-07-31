import type { Program } from "../lib/types";
import { formatBytes, formatDate } from "../lib/format";
import { openFolder } from "../lib/api";
import { STR } from "../lib/strings.fr";

function TechRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="mt-2">
      <p className="text-xs text-neutral-400">{label}</p>
      <div className="flex items-start gap-2">
        <code className="text-xs break-all">{value}</code>
        <button
          className="text-xs text-blue-600 hover:underline shrink-0"
          onClick={() => navigator.clipboard.writeText(value)}
        >
          {STR.copy}
        </button>
      </div>
    </div>
  );
}

export function DetailPanel({ program }: { program: Program }) {
  return (
    <aside className="w-96 shrink-0 border-l border-neutral-200 dark:border-neutral-800 p-4 overflow-auto">
      <div className="flex items-center gap-3">
        {program.iconDataUri && <img src={program.iconDataUri} alt="" className="h-8 w-8" />}
        <div>
          <h2 className="font-semibold">{program.name}</h2>
          <p className="text-sm text-neutral-500">{program.publisher ?? "—"}</p>
        </div>
      </div>

      <dl className="mt-4 space-y-1 text-sm">
        <div className="flex justify-between"><dt className="text-neutral-500">{STR.colVersion}</dt><dd>{program.version ?? "—"}</dd></div>
        <div className="flex justify-between"><dt className="text-neutral-500">{STR.colDate}</dt><dd>{formatDate(program.installDate)}</dd></div>
        <div className="flex justify-between"><dt className="text-neutral-500">{STR.colSize}</dt><dd>{formatBytes(program.estimatedSizeBytes)}</dd></div>
      </dl>

      <div className="mt-4">
        {program.installLocation ? (
          <button
            className="rounded-md border border-neutral-300 dark:border-neutral-700 px-3 py-1.5 text-sm hover:bg-neutral-50 dark:hover:bg-neutral-900"
            onClick={() => void openFolder(program.installLocation!)}
          >
            {STR.detailOpenFolder}
          </button>
        ) : (
          <p className="text-sm text-neutral-400">{STR.detailNoLocation}</p>
        )}
      </div>

      <details className="mt-6">
        <summary className="cursor-pointer text-sm text-neutral-500">{STR.detailTechnical}</summary>
        <TechRow label={STR.detailRegistryKey} value={program.id} />
        {program.uninstallString && (
          <TechRow label={STR.detailUninstallString} value={program.uninstallString} />
        )}
      </details>
    </aside>
  );
}
