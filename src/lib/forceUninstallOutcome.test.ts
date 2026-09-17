import { describe, expect, it, vi, beforeEach } from "vitest";
import { applyOutcome } from "./forceUninstallOutcome";
import { useProgramsStore } from "../store/programs";
import type { Program } from "./types";

function makeProgram(id: string): Program {
  return {
    id,
    name: "Test",
    publisher: null,
    version: null,
    installDate: null,
    estimatedSizeBytes: null,
    installLocation: null,
    uninstallString: null,
    quietUninstallString: null,
    displayIcon: null,
    scope: "user",
    isSystemEntry: false,
  };
}

describe("applyOutcome", () => {
  beforeEach(() => {
    useProgramsStore.setState({ programs: [], selectedId: null });
  });

  it("removes the program and sets a success message on completed+succeeded", () => {
    useProgramsStore.setState({ programs: [makeProgram("A"), makeProgram("B")] });
    const setFeedback = vi.fn();

    applyOutcome(
      { outcome: "completed", result: { steps: [], succeeded: true } },
      "A",
      setFeedback
    );

    expect(useProgramsStore.getState().programs.map((p) => p.id)).toEqual(["B"]);
    expect(setFeedback).toHaveBeenCalledWith(expect.any(String));
  });

  it("keeps the program and sets a failure message on completed+not succeeded", () => {
    useProgramsStore.setState({ programs: [makeProgram("A")] });
    const setFeedback = vi.fn();

    applyOutcome(
      { outcome: "completed", result: { steps: [], succeeded: false } },
      "A",
      setFeedback
    );

    expect(useProgramsStore.getState().programs.map((p) => p.id)).toEqual(["A"]);
    expect(setFeedback).toHaveBeenCalledWith(expect.any(String));
  });

  it("does not touch the store when programId is null (post-relaunch case)", () => {
    useProgramsStore.setState({ programs: [makeProgram("A")] });
    const setFeedback = vi.fn();

    applyOutcome(
      { outcome: "completed", result: { steps: [], succeeded: true } },
      null,
      setFeedback
    );

    expect(useProgramsStore.getState().programs.map((p) => p.id)).toEqual(["A"]);
  });

  it("reports the elevation-requested outcome without touching the store", () => {
    useProgramsStore.setState({ programs: [makeProgram("A")] });
    const setFeedback = vi.fn();

    applyOutcome({ outcome: "elevationRequested" }, "A", setFeedback);

    expect(useProgramsStore.getState().programs.map((p) => p.id)).toEqual(["A"]);
    expect(setFeedback).toHaveBeenCalledWith(expect.any(String));
  });

  it("reports the failed outcome's message", () => {
    const setFeedback = vi.fn();
    applyOutcome({ outcome: "failed", message: "boom" }, "A", setFeedback);
    expect(setFeedback).toHaveBeenCalledWith("boom");
  });
});
