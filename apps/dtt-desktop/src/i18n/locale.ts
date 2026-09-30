import registry from "../../../../crates/dtt-i18n/locales.json";
export { registry };
export type AppLocale = keyof typeof registry;
export type LocalePreference = "system" | AppLocale;
export function validatePreference(value: unknown): LocalePreference {
  return typeof value === "string" && Object.hasOwn(registry, value) ? value as AppLocale : "system";
}
export function negotiate(candidates: readonly string[]): AppLocale {
  for (const candidate of candidates) {
    try {
      const tag = new Intl.Locale(candidate);
      if (tag.language === "en" || tag.language === "ja" || tag.language === "ru") return tag.language;
      if (tag.language === "zh" && (tag.script ? tag.script === "Hans" : !tag.region || ["CN", "SG"].includes(tag.region))) return "zh-Hans";
    } catch { /* Skip malformed system candidates. */ }
  }
  return "en";
}
export function resolveLocale(preference: LocalePreference): AppLocale {
  return preference === "system" ? negotiate(navigator.languages) : preference;
}
