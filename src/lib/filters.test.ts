import { describe, expect, it } from "vitest";
import { applyFilters } from "./filters";
import type { Program } from "./types";

const prog = (over: Partial<Program>): Program => ({
  id: Math.random().toString(36).slice(2), name: "P", publisher: "E", version: null,
  installDate: null, estimatedSizeBytes: null, installLocation: null,
  uninstallString: null, quietUninstallString: null, displayIcon: null,
  scope: "machine", isSystemEntry: false, ...over,
});

const NOW = "2026-07-31";

describe("applyFilters", () => {
  it("sans filtre actif, rend tout", () => {
    const list = [prog({}), prog({})];
    expect(applyFilters(list, new Set(), NOW)).toHaveLength(2);
  });
  it("large : > 1 Go strictement", () => {
    const big = prog({ estimatedSizeBytes: 2 * 1024 ** 3 });
    const small = prog({ estimatedSizeBytes: 500 * 1024 ** 2 });
    const unknown = prog({ estimatedSizeBytes: null });
    expect(applyFilters([big, small, unknown], new Set(["large"]), NOW)).toEqual([big]);
  });
  it("recent : installé il y a moins de 30 jours", () => {
    const recent = prog({ installDate: "2026-07-15" });
    const old = prog({ installDate: "2026-01-01" });
    const unknown = prog({ installDate: null });
    const future = prog({ installDate: "2026-12-31" });
    expect(applyFilters([recent, old, unknown, future], new Set(["recent"]), NOW)).toEqual([recent]);
  });
  it("noPublisher : éditeur absent", () => {
    const anon = prog({ publisher: null });
    const named = prog({ publisher: "Contoso" });
    expect(applyFilters([anon, named], new Set(["noPublisher"]), NOW)).toEqual([anon]);
  });
  it("userScope : installations par utilisateur", () => {
    const user = prog({ scope: "user" });
    const machine = prog({ scope: "machine" });
    expect(applyFilters([user, machine], new Set(["userScope"]), NOW)).toEqual([user]);
  });
  it("les filtres se cumulent (ET)", () => {
    const match = prog({ publisher: null, scope: "user" });
    const half = prog({ publisher: null, scope: "machine" });
    expect(applyFilters([match, half], new Set(["noPublisher", "userScope"]), NOW)).toEqual([match]);
  });
});
