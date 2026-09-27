import { LazyStore } from "@tauri-apps/plugin-store";
import { isTauri } from "@tauri-apps/api/core";
import { validatePreference, type LocalePreference } from "@/i18n/locale";
import type { GameLanguageDto, SwapUnknownStrategyDto, UnknownStrategyDto } from "@/ipc/bindings";
export type ThemePreference = "light" | "dark" | "system";

export interface PersistedData {
  schemaVersion: number;
  overrides: {
    gameRoot: string | null;
    documentsDir: string | null;
    launcherDb: string | null;
  };
  languages: GameLanguageDto[];
  unknownStrategy: UnknownStrategyDto;
  swapUnknownStrategy: SwapUnknownStrategyDto;
  localePreference: LocalePreference;
  uiTheme: ThemePreference;
  saveSource: "local" | "steam_cloud";
}

export const DEFAULT_PERSISTED_DATA: PersistedData = {
  schemaVersion: 2,
  overrides: {
    gameRoot: null,
    documentsDir: null,
    launcherDb: null,
  },
  languages: ["english"],
  unknownStrategy: "include_flagged",
  swapUnknownStrategy: "keep_base",
  localePreference: "system",
  uiTheme: "system",
  saveSource: "local",
};


const STORE_FILENAME = "dtt-desktop.json";
const store = new LazyStore(STORE_FILENAME);
export function validatePersistedData(raw: unknown): PersistedData {
  const value = raw && typeof raw === "object" ? raw as Record<string, unknown> : {};
  if (value.schemaVersion !== 2) return { ...DEFAULT_PERSISTED_DATA };
  const languageValues = Array.isArray(value.languages) ? value.languages : ["english"];
  const languages = [...new Set(languageValues.filter((v): v is "english" | "simp_chinese" => v === "english" || v === "simp_chinese"))];
  const overrides = value.overrides && typeof value.overrides === "object" ? value.overrides as Record<string, unknown> : {};
  const path = (key: string) => typeof overrides[key] === "string" ? overrides[key] as string : null;
  return {
    schemaVersion: 2,
    overrides: { gameRoot: path("gameRoot"), documentsDir: path("documentsDir"), launcherDb: path("launcherDb") },
    languages: languages.length ? languages : ["english"],
    unknownStrategy: value.unknownStrategy === "exclude_strict" || value.unknownStrategy === "error" ? value.unknownStrategy : "include_flagged",
    swapUnknownStrategy: value.swapUnknownStrategy === "error" ? "error" : "keep_base",
    localePreference: validatePreference(value.localePreference),
    uiTheme: value.uiTheme === "light" || value.uiTheme === "dark" ? value.uiTheme : "system",
    saveSource: value.saveSource === "steam_cloud" ? "steam_cloud" : "local",
  };
}
export async function loadPersistedData(): Promise<PersistedData> {
  let raw: unknown;
  try {
    if (isTauri()) {
      raw = await store.get("preferences");
    } else {
      raw = JSON.parse(localStorage.getItem(STORE_FILENAME) ?? "null");
    }
  } catch { raw = null; }
  return validatePersistedData(raw);
}
let writes: Promise<void> = Promise.resolve();
export function savePersistedData(data: Partial<PersistedData>): Promise<void> {
  const next = writes.catch(() => {}).then(async () => {
    const current = await loadPersistedData();
    const updated = validatePersistedData({ ...current, ...data, schemaVersion: 2 });
    if (isTauri()) {
      await store.set("preferences", updated);
      await store.save();
    } else {
      localStorage.setItem(STORE_FILENAME, JSON.stringify(updated));
    }
  });
  writes = next;
  return next;
}
