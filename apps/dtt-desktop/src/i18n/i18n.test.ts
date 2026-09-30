import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { bootstrapLocale, currentLocale, listenForSystemLocale, setLocalePreference } from "./controller";
import { normalizeError } from "@/ipc/errors";
import { errorTitle, diagnosticMessage } from "./messages";
import i18n, { createInstance, initializeI18n } from "./index";
import { resources } from "./resources";

vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => false }));
vi.mock("@tauri-apps/plugin-store", () => ({ LazyStore: class {} }));

const storage = new Map<string, string>();
const cleanups: Array<() => void> = [];

beforeEach(async () => {
  storage.clear();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => storage.set(key, value),
  });
  vi.stubGlobal("document", { documentElement: { lang: "", dir: "" }, title: "" });
  vi.stubGlobal("window", new EventTarget());
  vi.stubGlobal("navigator", { languages: ["en-US"] });
  await bootstrapLocale("system");
});

afterEach(() => {
  for (const cleanup of cleanups.splice(0)) cleanup();
  vi.unstubAllGlobals();
});

describe("locale lifecycle", () => {
  it.each([
    { locale: "ja", system: "ja-JP", title: "DTT — 動的技術ツリー", error: "セーブにアクセスできません" },
    { locale: "ru", system: "ru-RU", title: "DTT — Динамическое дерево технологий", error: "Сохранение недоступно" },
  ] as const)("restores and switches the $locale interface", async ({ locale, system, title, error }) => {
    vi.stubGlobal("navigator", { languages: [system] });
    await bootstrapLocale("system");
    expect(currentLocale()).toBe(locale);
    expect(document.title).toBe(title);
    await setLocalePreference("en");
    await setLocalePreference(locale);
    expect(document.documentElement.lang).toBe(locale);
    expect(document.documentElement.dir).toBe("ltr");
    expect(document.title).toBe(title);
    expect(i18n.resolvedLanguage).toBe(locale);
    expect(errorTitle(normalizeError({ code: "SAVE_UNAVAILABLE" }))).toBe(error);
    expect(JSON.parse(storage.get("dtt-desktop.json")!).localePreference).toBe(locale);
  });

  it("sets initial document language and ignores system changes after explicit selection", async () => {
    vi.stubGlobal("navigator", { languages: ["fr-FR", "zh-SG"] });
    await bootstrapLocale("system");
    expect(document.documentElement.lang).toBe("zh-Hans");
    cleanups.push(listenForSystemLocale());

    await setLocalePreference("en");
    window.dispatchEvent(new Event("languagechange"));
    expect(currentLocale()).toBe("en");

    await setLocalePreference("system");
    expect(currentLocale()).toBe("zh-Hans");
  });

  it("persists the last concurrent selection", async () => {
    await Promise.all([setLocalePreference("zh-Hans"), setLocalePreference("en")]);
    expect(currentLocale()).toBe("en");
    expect(JSON.parse(storage.get("dtt-desktop.json")!).localePreference).toBe("en");
  });

  it("keeps the in-memory locale when persistence fails", async () => {
    vi.stubGlobal("localStorage", {
      getItem: () => null,
      setItem: () => { throw new Error("disk full"); },
    });
    await setLocalePreference("zh-Hans");
    expect(currentLocale()).toBe("zh-Hans");
  });
});

it.each([
  [0, "Доступно 0 сохранений"], [1, "Доступно 1 сохранение"],
  [2, "Доступно 2 сохранения"], [5, "Доступно 5 сохранений"],
  [11, "Доступно 11 сохранений"], [21, "Доступно 21 сохранение"],
  [22, "Доступно 22 сохранения"], [25, "Доступно 25 сохранений"],
  [1.5, "Доступно 1,5 сохранения"],
] as const)("renders Russian plurals for %s", async (count, expected) => {
  const instance = await initializeI18n("ru", createInstance());
  expect(instance.t($ => $.availableCount, { ns: "saves", count })).toBe(expected);
});

it("renders Japanese counts without a singular form", async () => {
  const instance = await initializeI18n("ja", createInstance());
  for (const count of [0, 1, 2, 5]) {
    expect(instance.t($ => $.availableCount, { ns: "saves", count })).toBe("利用可能なセーブ：" + count + " 件");
  }
});

it("uses native output language names in every interface locale", () => {
  for (const resource of Object.values(resources)) {
    expect(resource.generation.outputLanguages.items).toEqual({
      english: "English", simp_chinese: "简体中文", japanese: "日本語", russian: "Русский",
    });
  }
});

it("retranslates retained errors and diagnostics on language change", async () => {
  const error = normalizeError({ code: "SAVE_UNAVAILABLE", context: { path: "synthetic.sav" } });
  const item = { kind: "missing_mod_descriptor", modName: "Example" } as const;
  expect(errorTitle(error)).toBe("Save unavailable");
  const original = diagnosticMessage(item).summary;

  await setLocalePreference("zh-Hans");
  expect(errorTitle(error)).toBe("存档不可访问");
  expect(diagnosticMessage(item).summary).not.toBe(original);
});
