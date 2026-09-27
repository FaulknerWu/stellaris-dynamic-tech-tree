import { useSyncExternalStore } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import i18n, { initializeI18n } from "./index";
import { registry, resolveLocale, type AppLocale, type LocalePreference } from "./locale";
import { savePersistedData } from "@/app/store";
let state: { preference: LocalePreference; resolvedLocale: AppLocale; saveFailed: boolean } = { preference: "system", resolvedLocale: "en", saveFailed: false };
const listeners = new Set<() => void>();
function publish() { for (const listener of listeners) listener(); }
function syncDocument(locale: AppLocale) {
  document.documentElement.lang = locale;
  document.documentElement.dir = registry[locale].dir;
  document.title = i18n.t($ => $.appTitle);
  if (isTauri()) void getCurrentWindow().setTitle(document.title).catch(() => {});
}
export async function bootstrapLocale(preference: LocalePreference) {
  const resolvedLocale = resolveLocale(preference);
  await initializeI18n(resolvedLocale);
  state = { preference, resolvedLocale, saveFailed: false };
  syncDocument(resolvedLocale);
}
let sequence = 0;
export async function setLocalePreference(preference: LocalePreference) {
  const current = ++sequence;
  const resolvedLocale = resolveLocale(preference);
  state = { preference, resolvedLocale, saveFailed: false };
  await i18n.changeLanguage(resolvedLocale);
  if (current !== sequence) return;
  syncDocument(resolvedLocale); publish();
  try { await savePersistedData({ localePreference: preference }); }
  catch { if (current === sequence) { state = { ...state, saveFailed: true }; publish(); } }
}
export function listenForSystemLocale() {
  const onChange = () => {
    if (state.preference === "system") void setLocalePreference("system");
  };
  window.addEventListener("languagechange", onChange);
  return () => window.removeEventListener("languagechange", onChange);
}
export function currentLocale() { return state.resolvedLocale; }
export function useLocale() {
  return useSyncExternalStore((listener) => { listeners.add(listener); return () => { listeners.delete(listener); }; }, () => state);
}
