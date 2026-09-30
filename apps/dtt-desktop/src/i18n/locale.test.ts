import { describe, expect, it } from "vitest";
import cases from "../../../../crates/dtt-i18n/negotiation-cases.json";
import { negotiate, registry, validatePreference } from "./locale";

describe("locale negotiation", () => {
  it.each(cases)("negotiates $candidates consistently with Rust", ({ candidates, expected }) => {
    expect(negotiate(candidates)).toBe(expected);
  });

  it.each(Object.keys(registry))("retains the registered %s preference", (locale) => {
    expect(validatePreference(locale)).toBe(locale);
  });

  it.each(["ja-JP", "ru-RU", "constructor", "toString", "__proto__", null, {}])("rejects unregistered preference %s", (value) => {
    expect(validatePreference(value)).toBe("system");
  });
});
