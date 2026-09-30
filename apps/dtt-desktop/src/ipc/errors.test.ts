import { describe, expect, it } from "vitest";
import { normalizeError } from "./errors";

describe("error normalization", () => {
  it.each([
    { name: "undefined", payload: undefined },
    { name: "an unknown code", payload: { code: "FUTURE" } },
  ])("normalizes $name to an internal error", ({ payload }) => {
    expect(normalizeError(payload).code).toBe("INTERNAL");
  });

  it("normalizes cyclic payloads without throwing", () => {
    const cycle: { self?: unknown } = {};
    cycle.self = cycle;

    expect(normalizeError(cycle)).toEqual({
      code: "INTERNAL",
      technicalDetail: "Unserializable error payload",
    });
  });
});
