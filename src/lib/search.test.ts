import { describe, expect, it } from "vitest";
import { compareBy, matches, normalize } from "./search";
import type { Program } from "./types";

const prog = (over: Partial<Program>): Program => ({
  id: "x", name: "", publisher: null, version: null, installDate: null,
  estimatedSizeBytes: null, installLocation: null, uninstallString: null,
  quietUninstallString: null, displayIcon: null, scope: "machine",
  isSystemEntry: false, ...over,
});

describe("normalize", () => {
  it("retire accents et casse", () => {
    expect(normalize("Éditeur Génial")).toBe("editeur genial");
  });
});

describe("matches", () => {
  it("matche le nom sans tenir compte des accents", () => {
    expect(matches(prog({ name: "Télécharger Vidéo" }), "telecharger")).toBe(true);
  });
  it("matche l'éditeur", () => {
    expect(matches(prog({ name: "X", publisher: "Société Nùmerique" }), "societe")).toBe(true);
  });
  it("ne matche pas ailleurs", () => {
    expect(matches(prog({ name: "Paint", publisher: "Contoso" }), "zzz")).toBe(false);
  });
  it("requête vide matche tout", () => {
    expect(matches(prog({ name: "Paint" }), "  ")).toBe(true);
  });
});

describe("compareBy", () => {
  it("trie par taille desc avec null en dernier", () => {
    const a = prog({ id: "a", estimatedSizeBytes: 100 });
    const b = prog({ id: "b", estimatedSizeBytes: null });
    const c = prog({ id: "c", estimatedSizeBytes: 900 });
    const sorted = [a, b, c].sort(compareBy("size", "desc"));
    expect(sorted.map((p) => p.id)).toEqual(["c", "a", "b"]);
  });
  it("trie par nom asc, insensible aux accents", () => {
    const a = prog({ id: "a", name: "Zèbre" });
    const b = prog({ id: "b", name: "abricot" });
    const sorted = [a, b].sort(compareBy("name", "asc"));
    expect(sorted.map((p) => p.id)).toEqual(["b", "a"]);
  });
  it("trie par date desc avec null en dernier", () => {
    const a = prog({ id: "a", installDate: "2026-01-01" });
    const b = prog({ id: "b", installDate: null });
    const c = prog({ id: "c", installDate: "2026-06-15" });
    const sorted = [a, b, c].sort(compareBy("date", "desc"));
    expect(sorted.map((p) => p.id)).toEqual(["c", "a", "b"]);
  });
});
