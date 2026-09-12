import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import enCommon from "./locales/en/common.json";
import enEmpire from "./locales/en/empire.json";
import enEnvironment from "./locales/en/environment.json";
import enErrors from "./locales/en/errors.json";
import enGeneration from "./locales/en/generation.json";
import enResults from "./locales/en/results.json";
import enSaves from "./locales/en/saves.json";
import zhCommon from "./locales/zh-CN/common.json";
import zhEmpire from "./locales/zh-CN/empire.json";
import zhEnvironment from "./locales/zh-CN/environment.json";
import zhErrors from "./locales/zh-CN/errors.json";
import zhGeneration from "./locales/zh-CN/generation.json";
import zhResults from "./locales/zh-CN/results.json";
import zhSaves from "./locales/zh-CN/saves.json";

const resources = {
  "zh-CN": {
    common: zhCommon,
    environment: zhEnvironment,
    saves: zhSaves,
    empire: zhEmpire,
    generation: zhGeneration,
    results: zhResults,
    errors: zhErrors,
  },
  en: {
    common: enCommon,
    environment: enEnvironment,
    saves: enSaves,
    empire: enEmpire,
    generation: enGeneration,
    results: enResults,
    errors: enErrors,
  },
} as const;

i18n.use(initReactI18next).init({
  resources,
  lng: "zh-CN",
  fallbackLng: "zh-CN",
  defaultNS: "common",
  ns: ["common", "environment", "saves", "empire", "generation", "results", "errors"],
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
