import { registry, type AppLocale } from "./locale";
const numbers = new Map<string, Intl.NumberFormat>();
const dates = new Map<string, Intl.DateTimeFormat>();
const lists = new Map<string, Intl.ListFormat>();
export function numberFormatter(locale: AppLocale, options: Intl.NumberFormatOptions = {}) {
  const key = JSON.stringify([locale, options]);
  let formatter = numbers.get(key);
  if (!formatter) { formatter = new Intl.NumberFormat(registry[locale].formatLocale, options); numbers.set(key, formatter); }
  return formatter;
}
export function dateFormatter(locale: AppLocale, options: Intl.DateTimeFormatOptions) {
  const key = JSON.stringify([locale, options]);
  let formatter = dates.get(key);
  if (!formatter) { formatter = new Intl.DateTimeFormat(registry[locale].formatLocale, options); dates.set(key, formatter); }
  return formatter;
}
export function listFormatter(locale: AppLocale) {
  let formatter = lists.get(locale);
  if (!formatter) { formatter = new Intl.ListFormat(registry[locale].formatLocale); lists.set(locale, formatter); }
  return formatter;
}
