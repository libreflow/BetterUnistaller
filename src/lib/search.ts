import type { Program } from "./types";

const DIACRITICS_RE = new RegExp(
  `[${String.fromCharCode(0x0300)}-${String.fromCharCode(0x036f)}]`,
  "g",
);

export function normalize(s: string): string {
  return s
    .normalize("NFD")
    .replace(DIACRITICS_RE, "")
    .toLowerCase()
    .trim();
}

export function matches(program: Program, query: string): boolean {
  const q = normalize(query);
  if (q === "") return true;
  return (
    normalize(program.name).includes(q) ||
    (program.publisher !== null && normalize(program.publisher).includes(q))
  );
}

export type SortKey = "name" | "size" | "date" | "publisher";
export type SortDir = "asc" | "desc";

/** null est toujours classé en dernier, quel que soit le sens. */
export function compareBy(key: SortKey, dir: SortDir) {
  const sign = dir === "asc" ? 1 : -1;
  return (a: Program, b: Program): number => {
    const [va, vb] = [sortValue(a, key), sortValue(b, key)];
    if (va === null && vb === null) return 0;
    if (va === null) return 1;
    if (vb === null) return -1;
    if (typeof va === "number" && typeof vb === "number") return sign * (va - vb);
    return sign * String(va).localeCompare(String(vb));
  };
}

function sortValue(p: Program, key: SortKey): string | number | null {
  switch (key) {
    case "name": return normalize(p.name);
    case "publisher": return p.publisher === null ? null : normalize(p.publisher);
    case "size": return p.estimatedSizeBytes;
    case "date": return p.installDate; // ISO : ordre lexicographique = ordre chronologique
  }
}
