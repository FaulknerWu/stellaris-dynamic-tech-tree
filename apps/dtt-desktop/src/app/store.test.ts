import { describe, expect, it } from "vitest";
import { DEFAULT_PERSISTED_DATA, validatePersistedData } from "./store";

describe("persisted preferences", () => {
  it.each([
    { name: "missing data", value: null },
    { name: "an outdated schema", value: { schemaVersion: 1, languages: ["simp_chinese"] } },
  ])("discards $name", ({ value }) => {
    expect(validatePersistedData(value)).toEqual(DEFAULT_PERSISTED_DATA);
  });

  it("defaults unsupported language preferences while retaining valid settings", () => {
    const data = validatePersistedData({
      schemaVersion: 2,
      localePreference: "zh-CN",
      languages: ["french"],
      uiTheme: "dark",
      overrides: { gameRoot: "synthetic-root" },
    });

    expect(data.localePreference).toBe("system");
    expect(data.languages).toEqual(["english"]);
    expect(data.uiTheme).toBe("dark");
    expect(data.overrides.gameRoot).toBe("synthetic-root");
  });
});
