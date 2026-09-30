import ruDiagnostics from "./locales/ru/diagnostics.json";
import ruCommon from "./locales/ru/common.json";
import ruEnvironment from "./locales/ru/environment.json";
import ruSaves from "./locales/ru/saves.json";
import ruEmpire from "./locales/ru/empire.json";
import ruGeneration from "./locales/ru/generation.json";
import ruResults from "./locales/ru/results.json";
import ruErrors from "./locales/ru/errors.json";
import jaDiagnostics from "./locales/ja/diagnostics.json";
import jaCommon from "./locales/ja/common.json";
import jaEnvironment from "./locales/ja/environment.json";
import jaSaves from "./locales/ja/saves.json";
import jaEmpire from "./locales/ja/empire.json";
import jaGeneration from "./locales/ja/generation.json";
import jaResults from "./locales/ja/results.json";
import jaErrors from "./locales/ja/errors.json";
import enDiagnostics from "./locales/en/diagnostics.json";
import zhDiagnostics from "./locales/zh-Hans/diagnostics.json";
import enCommon from "./locales/en/common.json";
import enEmpire from "./locales/en/empire.json";
import enEnvironment from "./locales/en/environment.json";
import enErrors from "./locales/en/errors.json";
import enGeneration from "./locales/en/generation.json";
import enResults from "./locales/en/results.json";
import enSaves from "./locales/en/saves.json";
import zhCommon from "./locales/zh-Hans/common.json";
import zhEmpire from "./locales/zh-Hans/empire.json";
import zhEnvironment from "./locales/zh-Hans/environment.json";
import zhErrors from "./locales/zh-Hans/errors.json";
import zhGeneration from "./locales/zh-Hans/generation.json";
import zhResults from "./locales/zh-Hans/results.json";
import zhSaves from "./locales/zh-Hans/saves.json";

export const resources = {
  ru: {
    diagnostics: ruDiagnostics,
    common: ruCommon,
    environment: ruEnvironment,
    saves: ruSaves,
    empire: ruEmpire,
    generation: ruGeneration,
    results: ruResults,
    errors: ruErrors,
  },
  ja: {
    diagnostics: jaDiagnostics,
    common: jaCommon,
    environment: jaEnvironment,
    saves: jaSaves,
    empire: jaEmpire,
    generation: jaGeneration,
    results: jaResults,
    errors: jaErrors,
  },
  "zh-Hans": {
    diagnostics: zhDiagnostics,
    common: zhCommon,
    environment: zhEnvironment,
    saves: zhSaves,
    empire: zhEmpire,
    generation: zhGeneration,
    results: zhResults,
    errors: zhErrors,
  },
  en: {
    diagnostics: enDiagnostics,
    common: enCommon,
    environment: enEnvironment,
    saves: enSaves,
    empire: enEmpire,
    generation: enGeneration,
    results: enResults,
    errors: enErrors,
  },
} as const;

