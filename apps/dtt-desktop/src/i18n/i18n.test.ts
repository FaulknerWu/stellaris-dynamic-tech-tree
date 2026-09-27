import { beforeEach, describe, expect, it, vi } from "vitest";
import { negotiate } from "./locale";
import cases from "../../../../crates/dtt-i18n/negotiation-cases.json";
import { initializeI18n, createInstance } from "./index";
import { bootstrapLocale, currentLocale, listenForSystemLocale, setLocalePreference } from "./controller";
import { validatePersistedData } from "@/app/store";
import { formatDate, formatFileSize } from "@/lib/format";
import { normalizeError } from "@/ipc/errors";
import { errorTitle, diagnosticMessage } from "./messages";
vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => false }));
vi.mock("@tauri-apps/plugin-store", () => ({ LazyStore: class {} }));
const storage = new Map<string, string>();
beforeEach(async () => {
  storage.clear();
  vi.stubGlobal("localStorage", { getItem: (key: string) => storage.get(key) ?? null, setItem: (key: string, value: string) => storage.set(key, value) });
  vi.stubGlobal("document", { documentElement: { lang: "", dir: "" }, title: "" });
  vi.stubGlobal("window", new EventTarget());
  vi.stubGlobal("navigator", { languages: ["en-US"] });
  await bootstrapLocale("system");
});
describe("locale lifecycle", () => {
  it.each(cases)("negotiates $candidates consistently with Rust", ({ candidates, expected }) => {
    expect(negotiate(candidates)).toBe(expected);
  });
  it("validates only the current schema and discards old preferences", () => {
    const data = validatePersistedData({ schemaVersion: 2, localePreference: "zh-CN", languages: ["french"], uiTheme: "dark", overrides: { gameRoot: "synthetic-root" } });
    expect(data.localePreference).toBe("system"); expect(data.languages).toEqual(["english"]);
    expect(data.uiTheme).toBe("dark"); expect(data.overrides.gameRoot).toBe("synthetic-root");
    expect(validatePersistedData(null).localePreference).toBe("system");
    expect(validatePersistedData({ schemaVersion: 1, languages: ["simp_chinese"] }).languages).toEqual(["english"]);
    expect(validatePersistedData({ schemaVersion: 2, localePreference: "zh-CN" }).localePreference).toBe("system");
  });
  it("sets initial document language and ignores system changes after explicit selection", async () => {
    vi.stubGlobal("navigator", { languages: ["fr-FR", "zh-SG"] });
    await bootstrapLocale("system"); expect(document.documentElement.lang).toBe("zh-Hans");
    const cleanup = listenForSystemLocale();
    await setLocalePreference("en"); window.dispatchEvent(new Event("languagechange"));
    expect(currentLocale()).toBe("en");
    await setLocalePreference("system"); expect(currentLocale()).toBe("zh-Hans"); cleanup();
  });
  it("persists the last concurrent selection", async () => {
    await Promise.all([setLocalePreference("zh-Hans"), setLocalePreference("en")]);
    expect(currentLocale()).toBe("en"); expect(JSON.parse(storage.get("dtt-desktop.json")!).localePreference).toBe("en");
  });
  it("keeps the in-memory locale when persistence fails", async () => {
    vi.stubGlobal("localStorage", { getItem: () => null, setItem: () => { throw new Error("disk full"); } });
    await setLocalePreference("zh-Hans"); expect(currentLocale()).toBe("zh-Hans");
  });
});
it("initializes independent instances with correct plural and English fallback", async () => {
  const instance = await initializeI18n("en", createInstance());
  expect(instance.t($ => $.availableCount, { ns: "saves", count: 1 })).toBe("1 available save");
  expect(instance.t($ => $.availableCount, { ns: "saves", count: 2 })).toBe("2 available saves");
  await instance.changeLanguage("zh-Hans");
  expect(instance.t($ => $.availableCount, { ns: "saves", count: 0 })).toContain("0");
  expect(instance.options.fallbackLng).toEqual(["en"]);
});
it("formats epoch and IEC sizes and handles invalid dates", () => {
  expect(formatDate(0)).not.toBe("No date"); expect(formatDate(null)).toBe("No date"); expect(formatDate(1e30)).toBe("No date");
  expect(formatFileSize(1024)).toBe("1 KiB"); expect(formatFileSize(1048576)).toBe("1 MiB");
});
it("normalizes unknown and cyclic errors without throwing", () => {
  const cycle: { self?: unknown } = {}; cycle.self = cycle;
  for (const payload of [cycle, 1n, undefined, { code: "FUTURE" }, { code: "INTERNAL", technicalDetail: 42 }]) {
    expect(normalizeError(payload).code).toBe("INTERNAL");
  }
});
it("retranslates retained errors and diagnostics on language change", async () => {
  const error = normalizeError({ code: "SAVE_UNAVAILABLE", context: { path: "synthetic.sav" } });
  const item = { kind: "missing_mod_descriptor", modName: "Example" } as const;
  expect(errorTitle(error)).toBe("Save unavailable"); const original = diagnosticMessage(item).summary;
  await setLocalePreference("zh-Hans"); expect(errorTitle(error)).toBe("存档不可访问");
  expect(diagnosticMessage(item).summary).not.toBe(original);
});
