const UNITS = [
  { limit: 1024 ** 3, div: 1024 ** 3, suffix: "Go" },
  { limit: 1024 ** 2, div: 1024 ** 2, suffix: "Mo" },
  { limit: 0, div: 1024, suffix: "Ko" },
];

export function formatBytes(bytes: number | null): string {
  if (bytes === null) return "—";
  const unit = UNITS.find((u) => bytes >= u.limit)!;
  const value = (bytes / unit.div).toFixed(1).replace(".", ",");
  return `${value} ${unit.suffix}`;
}

export function formatDate(iso: string | null): string {
  if (iso === null) return "—";
  const [y, m, d] = iso.split("-");
  return `${d}/${m}/${y}`;
}
