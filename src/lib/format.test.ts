import { describe, expect, it } from "vitest";
import { formatBytes, formatDate } from "./format";

describe("formatBytes", () => {
  it("affiche — pour null", () => expect(formatBytes(null)).toBe("—"));
  it("affiche en Mo sous le Go", () => expect(formatBytes(52_428_800)).toBe("50,0 Mo"));
  it("affiche en Go au-dessus", () => expect(formatBytes(1_610_612_736)).toBe("1,5 Go"));
  it("affiche en Ko sous le Mo", () => expect(formatBytes(10_240)).toBe("10,0 Ko"));
});

describe("formatDate", () => {
  it("affiche — pour null", () => expect(formatDate(null)).toBe("—"));
  it("formate en JJ/MM/AAAA", () => expect(formatDate("2026-07-31")).toBe("31/07/2026"));
});
