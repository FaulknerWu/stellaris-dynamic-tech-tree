import type {
  AppError,
  BootstrapDataDto,
  EnvironmentDto,
  GenerationProgressDto,
  GenerationResultDto,
  GenerationStageDto,
  InspectedSaveDto,
  SaveIndexDto,
  SaveLibraryDto,
  SupportedLanguageDto,
  SwapUnknownStrategyDto,
  UnknownStrategyDto,
} from "@/ipc/bindings";
import type { PersistedData } from "./store";

export type WizardStep = "settings" | "saves" | "generate" | "results";

export type SessionState = {
  step: WizardStep;

  bootstrap:
    | { status: "loading" }
    | { status: "ready"; data: BootstrapDataDto }
    | { status: "failed"; error: AppError };

  overrides: {
    gameRoot: string | null;
    documentsDir: string | null;
    launcherDb: string | null;
  };
  environment: EnvironmentDto | null;
  environmentError: AppError | null;

  saveSource: "local" | "steam_cloud";
  library: SaveLibraryDto | null;
  libraryStatus: "idle" | "scanning" | "ready" | "failed";
  libraryError: AppError | null;
  manualSave: SaveIndexDto | null;

  selectedSavePath: string | null;
  selectedSave: SaveIndexDto | null;
  inspection: InspectedSaveDto | null;
  inspectionStatus: "idle" | "inspecting" | "ready" | "failed";
  inspectionError: AppError | null;
  selectedCountryId: number | null;

  settings: {
    languages: SupportedLanguageDto[];
    unknownStrategy: UnknownStrategyDto;
    swapUnknownStrategy: SwapUnknownStrategyDto;
  };

  run:
    | { status: "idle" }
    | { status: "running"; stage: GenerationStageDto; percent: number; startTime: number }
    | { status: "cancelling"; stage: GenerationStageDto; percent: number; startTime: number }
    | { status: "done"; result: GenerationResultDto }
    | { status: "cancelled" }
    | { status: "failed"; error: AppError };

  scanSeq: number;
  inspectSeq: number;
  envSeq: number;
};

export const INITIAL_SESSION_STATE: SessionState = {
  step: "settings",
  bootstrap: { status: "loading" },
  overrides: {
    gameRoot: null,
    documentsDir: null,
    launcherDb: null,
  },
  environment: null,
  environmentError: null,
  saveSource: "local",
  library: null,
  libraryStatus: "idle",
  libraryError: null,
  manualSave: null,
  selectedSavePath: null,
  selectedSave: null,
  inspection: null,
  inspectionStatus: "idle",
  inspectionError: null,
  selectedCountryId: null,
  settings: {
    languages: ["simp_chinese"],
    unknownStrategy: "include_flagged",
    swapUnknownStrategy: "keep_base",
  },
  run: { status: "idle" },
  scanSeq: 0,
  inspectSeq: 0,
  envSeq: 0,
};

export type SessionAction =
  | { type: "BOOTSTRAP_START" }
  | { type: "BOOTSTRAP_SUCCESS"; data: BootstrapDataDto }
  | { type: "BOOTSTRAP_FAIL"; error: AppError }
  | { type: "LOAD_PERSISTED"; data: PersistedData }
  | { type: "SET_OVERRIDE"; key: "gameRoot" | "documentsDir" | "launcherDb"; value: string | null }
  | { type: "RESOLVE_ENV_START"; seq: number }
  | { type: "RESOLVE_ENV_SUCCESS"; environment: EnvironmentDto; seq: number }
  | { type: "RESOLVE_ENV_FAIL"; error: AppError; seq: number }
  | { type: "SET_SAVE_SOURCE"; source: "local" | "steam_cloud" }
  | { type: "SCAN_LIBRARY_START"; seq: number }
  | { type: "SCAN_LIBRARY_SUCCESS"; library: SaveLibraryDto; seq: number }
  | { type: "SCAN_LIBRARY_FAIL"; error: AppError; seq: number }
  | { type: "SELECT_SAVE"; save: SaveIndexDto | null }
  | { type: "SET_MANUAL_SAVE"; save: SaveIndexDto }
  | { type: "INSPECT_SAVE_START"; seq: number }
  | { type: "INSPECT_SAVE_SUCCESS"; inspection: InspectedSaveDto; seq: number }
  | { type: "INSPECT_SAVE_FAIL"; error: AppError; seq: number }
  | { type: "SELECT_COUNTRY_ID"; countryId: number | null }
  | { type: "SET_LANGUAGES"; languages: SupportedLanguageDto[] }
  | { type: "SET_UNKNOWN_STRATEGY"; strategy: UnknownStrategyDto }
  | { type: "SET_SWAP_UNKNOWN_STRATEGY"; strategy: SwapUnknownStrategyDto }
  | { type: "START_GENERATION" }
  | { type: "GENERATION_PROGRESS"; progress: GenerationProgressDto }
  | { type: "CANCEL_GENERATION_START" }
  | { type: "GENERATION_DONE"; result: GenerationResultDto }
  | { type: "GENERATION_CANCELLED" }
  | { type: "GENERATION_FAIL"; error: AppError }
  | { type: "NAVIGATE_STEP"; step: WizardStep }
  | { type: "RESET_SAVE_SELECTION" };

export function sessionReducer(state: SessionState, action: SessionAction): SessionState {
  switch (action.type) {
    case "BOOTSTRAP_START":
      return {
        ...state,
        bootstrap: { status: "loading" },
      };

    case "BOOTSTRAP_SUCCESS": {
      const hasDirectError = !!action.data.environmentError;
      const initialEnv: EnvironmentDto | null =
        !hasDirectError &&
        action.data.environment.gameRoot &&
        action.data.environment.documentsDir &&
        action.data.environment.launcherDb
          ? {
              gameRoot: action.data.environment.gameRoot,
              documentsDir: action.data.environment.documentsDir,
              launcherDb: action.data.environment.launcherDb,
              steamLibraries: action.data.environment.steamLibraries,
            }
          : null;

      return {
        ...state,
        bootstrap: { status: "ready", data: action.data },
        environment: state.environment ?? initialEnv,
        environmentError: state.environmentError ?? (action.data.environmentError || null),
      };
    }

    case "BOOTSTRAP_FAIL":
      return {
        ...state,
        bootstrap: { status: "failed", error: action.error },
      };

    case "LOAD_PERSISTED":
      return {
        ...state,
        overrides: action.data.overrides,
        saveSource: action.data.saveSource,
        settings: {
          languages: action.data.languages.length > 0 ? action.data.languages : state.settings.languages,
          unknownStrategy: action.data.unknownStrategy,
          swapUnknownStrategy: action.data.swapUnknownStrategy,
        },
      };

    case "SET_OVERRIDE":
      return {
        ...state,
        overrides: {
          ...state.overrides,
          [action.key]: action.value,
        },
      };

    case "RESOLVE_ENV_START":
      return {
        ...state,
        envSeq: action.seq,
      };

    case "RESOLVE_ENV_SUCCESS":
      if (action.seq < state.envSeq) return state;
      return {
        ...state,
        environment: action.environment,
        environmentError: null,
      };

    case "RESOLVE_ENV_FAIL":
      if (action.seq < state.envSeq) return state;
      return {
        ...state,
        environment: null,
        environmentError: action.error,
      };

    case "SET_SAVE_SOURCE":
      return {
        ...state,
        saveSource: action.source,
      };

    case "SCAN_LIBRARY_START":
      return {
        ...state,
        libraryStatus: "scanning",
        libraryError: null,
        scanSeq: action.seq,
      };

    case "SCAN_LIBRARY_SUCCESS": {
      if (action.seq < state.scanSeq) return state;

      return {
        ...state,
        libraryStatus: "ready",
        library: action.library,
        libraryError: null,
      };
    }

    case "SCAN_LIBRARY_FAIL":
      if (action.seq < state.scanSeq) return state;
      return {
        ...state,
        libraryStatus: "failed",
        libraryError: action.error,
      };

    case "SET_MANUAL_SAVE":
      return {
        ...state,
        manualSave: action.save,
        selectedSavePath: action.save.path,
        selectedSave: action.save,
        selectedCountryId: null,
        inspection: null,
        inspectionStatus: "idle",
        inspectionError: null,
      };

    case "SELECT_SAVE":
      return {
        ...state,
        selectedSavePath: action.save ? action.save.path : null,
        selectedSave: action.save,
        selectedCountryId: null,
        inspection: null,
        inspectionStatus: "idle",
        inspectionError: null,
      };

    case "INSPECT_SAVE_START":
      return {
        ...state,
        inspectionStatus: "inspecting",
        inspectionError: null,
        inspectSeq: action.seq,
      };

    case "INSPECT_SAVE_SUCCESS":
      if (action.seq < state.inspectSeq) return state;
      return {
        ...state,
        inspectionStatus: "ready",
        inspection: action.inspection,
        inspectionError: null,
        selectedCountryId:
          action.inspection.playerCountries.length === 1
            ? action.inspection.playerCountries[0]?.countryId ?? null
            : state.selectedCountryId,
      };

    case "INSPECT_SAVE_FAIL":
      if (action.seq < state.inspectSeq) return state;
      return {
        ...state,
        inspectionStatus: "failed",
        inspectionError: action.error,
        inspection: null,
      };

    case "SELECT_COUNTRY_ID":
      return {
        ...state,
        selectedCountryId: action.countryId,
      };

    case "SET_LANGUAGES":
      if (action.languages.length === 0) return state;
      return {
        ...state,
        settings: {
          ...state.settings,
          languages: action.languages,
        },
      };

    case "SET_UNKNOWN_STRATEGY":
      return {
        ...state,
        settings: {
          ...state.settings,
          unknownStrategy: action.strategy,
        },
      };

    case "SET_SWAP_UNKNOWN_STRATEGY":
      return {
        ...state,
        settings: {
          ...state.settings,
          swapUnknownStrategy: action.strategy,
        },
      };

    case "START_GENERATION":
      return {
        ...state,
        step: "generate",
        run: {
          status: "running",
          stage: "SAVE_PARSE",
          percent: 5,
          startTime: Date.now(),
        },
      };

    case "GENERATION_PROGRESS":
      if (state.run.status !== "running" && state.run.status !== "cancelling") {
        return state;
      }
      return {
        ...state,
        run: {
          ...state.run,
          stage: action.progress.stage,
          percent: action.progress.percent,
        },
      };

    case "CANCEL_GENERATION_START":
      if (state.run.status !== "running") return state;
      return {
        ...state,
        run: {
          ...state.run,
          status: "cancelling",
        },
      };

    case "GENERATION_DONE":
      return {
        ...state,
        step: "results",
        run: {
          status: "done",
          result: action.result,
        },
      };

    case "GENERATION_CANCELLED":
      return {
        ...state,
        step: "saves",
        run: { status: "cancelled" },
      };

    case "GENERATION_FAIL":
      return {
        ...state,
        step: "saves",
        run: { status: "failed", error: action.error },
      };

    case "NAVIGATE_STEP":
      if (state.run.status === "running" || state.run.status === "cancelling") {
        return state;
      }
      return {
        ...state,
        step: action.step,
      };

    case "RESET_SAVE_SELECTION":
      return {
        ...state,
        step: "saves",
        selectedSavePath: null,
        selectedSave: null,
        manualSave: null,
        inspection: null,
        inspectionStatus: "idle",
        inspectionError: null,
        selectedCountryId: null,
        run: { status: "idle" },
      };

    default:
      return state;
  }
}
