import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { bootstrapLocale, currentLocale, listenForSystemLocale, setLocalePreference } from "./controller";
import { normalizeError } from "@/ipc/errors";
import { errorTitle, diagnosticMessage } from "./messages";

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

it("retranslates retained errors and diagnostics on language change", async () => {
  const error = normalizeError({ code: "SAVE_UNAVAILABLE", context: { path: "synthetic.sav" } });
  const item = { kind: "missing_mod_descriptor", modName: "Example" } as const;
  expect(errorTitle(error)).toBe("Save unavailable");
  const original = diagnosticMessage(item).summary;

  await setLocalePreference("zh-Hans");
  expect(errorTitle(error)).toBe("存档不可访问");
  expect(diagnosticMessage(item).summary).not.toBe(original);
});
