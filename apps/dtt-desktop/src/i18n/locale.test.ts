import { describe, expect, it } from "vitest";
import cases from "../../../../crates/dtt-i18n/negotiation-cases.json";
import { negotiate } from "./locale";

describe("locale negotiation", () => {
  it.each(cases)("negotiates $candidates consistently with Rust", ({ candidates, expected }) => {
    expect(negotiate(candidates)).toBe(expected);
  });
});
