import i18n, { createInstance } from "i18next";
import { initReactI18next } from "react-i18next";
import { resources } from "./resources";
import { negotiate, registry, type AppLocale } from "./locale";
import { numberFormatter } from "./format";
export async function initializeI18n(locale: AppLocale, instance = i18n) {
  await instance.use(initReactI18next).init({
    resources, lng: locale, supportedLngs: Object.keys(registry), fallbackLng: "en", load: "currentOnly",
    defaultNS: "common", returnNull: false, returnEmptyString: false,
    interpolation: { escapeValue: false },
  });
  instance.services.formatter?.addCached("number", (language) => {
    const formatter = numberFormatter(negotiate([language ?? locale]));
    return (value: number) => formatter.format(value);
  });
  return instance;
}
export { createInstance };
export default i18n;
