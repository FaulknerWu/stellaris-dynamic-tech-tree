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

