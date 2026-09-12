import { LazyStore } from "@tauri-apps/plugin-store";

import type {
  SupportedLanguageDto,
  SwapUnknownStrategyDto,
  UnknownStrategyDto,
} from "@/ipc/bindings";

export type ThemePreference = "light" | "dark" | "system";
export type UiLocale = "zh-CN" | "en";

export interface PersistedData {
  overrides: {
    gameRoot: string | null;
    documentsDir: string | null;
    launcherDb: string | null;
  };
  languages: SupportedLanguageDto[];
  unknownStrategy: UnknownStrategyDto;
  swapUnknownStrategy: SwapUnknownStrategyDto;
  uiLocale: UiLocale;
  uiTheme: ThemePreference;
  saveSource: "local" | "steam_cloud";
}

export const DEFAULT_PERSISTED_DATA: PersistedData = {
  overrides: {
    gameRoot: null,
    documentsDir: null,
    launcherDb: null,
  },
  languages: ["simp_chinese"],
  unknownStrategy: "include_flagged",
  swapUnknownStrategy: "keep_base",
  uiLocale: "zh-CN",
  uiTheme: "system",
  saveSource: "local",
};

const STORE_FILENAME = "dtt-desktop.json";
let storeInstance: LazyStore | null = null;

function validatedSwapStrategy(value: unknown): SwapUnknownStrategyDto {
  return value === "keep_base" || value === "error"
    ? value
    : DEFAULT_PERSISTED_DATA.swapUnknownStrategy;
}

function getStore(): LazyStore {
  if (!storeInstance) {
    storeInstance = new LazyStore(STORE_FILENAME);
  }
  return storeInstance;
}

export async function loadPersistedData(): Promise<PersistedData> {
  try {
    const store = getStore();
    const overrides = (await store.get<PersistedData["overrides"]>("overrides")) ?? DEFAULT_PERSISTED_DATA.overrides;
    const languages = (await store.get<SupportedLanguageDto[]>("settings.languages")) ?? DEFAULT_PERSISTED_DATA.languages;
    const unknownStrategy = (await store.get<UnknownStrategyDto>("settings.unknownStrategy")) ?? DEFAULT_PERSISTED_DATA.unknownStrategy;
    const swapUnknownStrategy = validatedSwapStrategy(await store.get<unknown>("settings.swapUnknownStrategy"));
    const uiLocale = (await store.get<UiLocale>("ui.locale")) ?? DEFAULT_PERSISTED_DATA.uiLocale;
    const uiTheme = (await store.get<ThemePreference>("ui.theme")) ?? DEFAULT_PERSISTED_DATA.uiTheme;
    const saveSource = (await store.get<"local" | "steam_cloud">("saveSource")) ?? DEFAULT_PERSISTED_DATA.saveSource;

    return {
      overrides,
      languages,
      unknownStrategy,
      swapUnknownStrategy,
      uiLocale,
      uiTheme,
      saveSource,
    };
  } catch (err) {
    console.warn("Failed to load from Tauri store, using defaults/localStorage fallback", err);
    try {
      const raw = localStorage.getItem(STORE_FILENAME);
      if (raw) {
        const stored = JSON.parse(raw);
        return {
          ...DEFAULT_PERSISTED_DATA,
          ...stored,
          swapUnknownStrategy: validatedSwapStrategy(stored.swapUnknownStrategy),
        };
      }
    } catch {
    }
    return DEFAULT_PERSISTED_DATA;
  }
}

export async function savePersistedData(data: Partial<PersistedData>): Promise<void> {
  try {
    const store = getStore();
    if (data.overrides !== undefined) {
      await store.set("overrides", data.overrides);
    }
    if (data.languages !== undefined) {
      await store.set("settings.languages", data.languages);
    }
    if (data.unknownStrategy !== undefined) {
      await store.set("settings.unknownStrategy", data.unknownStrategy);
    }
    if (data.swapUnknownStrategy !== undefined) {
      await store.set("settings.swapUnknownStrategy", data.swapUnknownStrategy);
    }
    if (data.uiLocale !== undefined) {
      await store.set("ui.locale", data.uiLocale);
    }
    if (data.uiTheme !== undefined) {
      await store.set("ui.theme", data.uiTheme);
    }
    if (data.saveSource !== undefined) {
      await store.set("saveSource", data.saveSource);
    }
    await store.save();
  } catch (err) {
    console.warn("Failed to save to Tauri store, falling back to localStorage", err);
    try {
      const current = localStorage.getItem(STORE_FILENAME);
      const parsed = current ? JSON.parse(current) : {};
      localStorage.setItem(STORE_FILENAME, JSON.stringify({ ...parsed, ...data }));
    } catch {
    }
  }
}
