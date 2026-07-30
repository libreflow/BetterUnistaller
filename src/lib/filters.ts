import type { Program } from "./types";

export type FilterId = "large" | "recent" | "noPublisher" | "userScope";

const ONE_GIB = 1024 ** 3;
const THIRTY_DAYS_MS = 30 * 24 * 60 * 60 * 1000;

const PREDICATES: Record<FilterId, (p: Program, nowIso: string) => boolean> = {
  large: (p) => p.estimatedSizeBytes !== null && p.estimatedSizeBytes > ONE_GIB,
  recent: (p, nowIso) =>
    p.installDate !== null &&
    Date.parse(nowIso) - Date.parse(p.installDate) < THIRTY_DAYS_MS,
  noPublisher: (p) => p.publisher === null,
  userScope: (p) => p.scope === "user",
};

/** Les filtres actifs se cumulent en ET. `nowIso` est injecté pour la testabilité. */
export function applyFilters(programs: Program[], active: Set<FilterId>, nowIso: string): Program[] {
  if (active.size === 0) return programs;
  return programs.filter((p) => [...active].every((id) => PREDICATES[id](p, nowIso)));
}
