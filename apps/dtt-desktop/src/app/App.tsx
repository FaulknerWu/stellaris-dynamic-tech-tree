import { ErrorAlert } from "@/components/ErrorAlert";
import { currentLocale, listenForSystemLocale, setLocalePreference, useLocale } from "@/i18n/controller";
import type { AppLocale } from "@/i18n/locale";
import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  cancelGeneration,
  getBootstrapData,
  inspectSave,
  openOutputDirectory,
  resolveEnvironment,
  runGeneration,
  scanSaveLibrary,
} from "@/ipc/commands";
import type {
  GenerationProgressDto,
  InspectSaveRequestDto,
  SaveIndexDto,
  ScanSaveLibraryRequestDto,
} from "@/ipc/bindings";
import { GenerationPage } from "@/features/generation/GenerationPage";
import { GenerationErrorDialog } from "@/features/generation/GenerationErrorDialog";
import { ResultsPage } from "@/features/results/ResultsPage";
import { SaveAndEmpirePage } from "@/features/saves/SaveAndEmpirePage";
import { EnvironmentAndSettingsPage } from "@/features/settings/EnvironmentAndSettingsPage";
import { AppShell } from "./AppShell";
import {
  INITIAL_SESSION_STATE,
  canStartGeneration,
  sessionReducer,
  type SessionState,
  type WizardStep,
} from "./session";
import {
  type PersistedData,
  savePersistedData,
  type ThemePreference,

} from "./store";
import { applyTheme } from "./theme";

export function App({ persisted, initialSaveFailed }: { persisted: PersistedData; initialSaveFailed: boolean }) {
  const localeState = useLocale();
  useEffect(listenForSystemLocale, []);
  const { t } = useTranslation();
  const [state, dispatch] = useReducer(sessionReducer, sessionReducer(INITIAL_SESSION_STATE, { type: "LOAD_PERSISTED", data: persisted }));
  const [currentTheme, setCurrentTheme] = useState<ThemePreference>("system");

  const scanSeqRef = useRef(0);
  const inspectSeqRef = useRef(0);
  const envSeqRef = useRef(0);
  const selectionSeqRef = useRef(0);
  const selectedSaveRef = useRef(state.selectedSave);
  const saveSourceRef = useRef(state.saveSource);
  const resolveDebounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleInspectSave = useCallback(
    async (save: SaveIndexDto, countryId?: number) => {
      const seq = ++inspectSeqRef.current;
      dispatch({ type: "INSPECT_SAVE_START", seq });

      const request: InspectSaveRequestDto = {
        saveFile: save.path,
      };
      if (countryId !== undefined) {
        request.countryId = countryId;
      }

      try {
        const inspected = await inspectSave(request);
        if (seq !== inspectSeqRef.current) return;
        dispatch({ type: "INSPECT_SAVE_SUCCESS", inspection: inspected, seq });
      } catch (err: any) {
        if (seq !== inspectSeqRef.current) return;
        dispatch({ type: "INSPECT_SAVE_FAIL", error: err, seq });
      }
    },
    [],
  );

  const handleSelectSave = useCallback(
    (save: SaveIndexDto) => {
      ++selectionSeqRef.current;
      ++inspectSeqRef.current;
      selectedSaveRef.current = save;
      dispatch({ type: "SELECT_SAVE", save });
      if (save.state === "text") {
        handleInspectSave(save);
      }
    },
    [handleInspectSave],
  );

  const handleScanLibrary = useCallback(
    async (
      sourceOverride?: "local" | "steam_cloud",
      docsDirOverride?: string,
      steamLibsOverride?: string[],
    ) => {
      const source = sourceOverride ?? saveSourceRef.current;
      saveSourceRef.current = source;
      const seq = ++scanSeqRef.current;
      const selectionSeq = selectionSeqRef.current;
      dispatch({ type: "SCAN_LIBRARY_START", seq });

      const docsDir =
        docsDirOverride ??
        state.overrides.documentsDir ??
        state.environment?.documentsDir ??
        (state.bootstrap.status === "ready"
          ? state.bootstrap.data.environment.documentsDir
          : undefined);

      const steamLibs =
        steamLibsOverride ??
        state.environment?.steamLibraries ??
        (state.bootstrap.status === "ready"
          ? state.bootstrap.data.environment.steamLibraries
          : []);

      const request: ScanSaveLibraryRequestDto = {
        source,
        steamLibraries: steamLibs,
      };
      if (docsDir) request.documentsDir = docsDir;

      try {
        const lib = await scanSaveLibrary(request);
        if (seq !== scanSeqRef.current) return;
        dispatch({ type: "SCAN_LIBRARY_SUCCESS", library: lib, seq });
        savePersistedData({ saveSource: source });

        if (
          selectionSeq === selectionSeqRef.current &&
          !selectedSaveRef.current && lib.accounts.length > 0
        ) {
          const firstSave = lib.accounts[0]?.campaigns[0]?.saves[0];
          if (firstSave && firstSave.state === "text") {
            handleSelectSave(firstSave);
          }
        }
      } catch (err: any) {
        if (seq !== scanSeqRef.current) return;
        dispatch({ type: "SCAN_LIBRARY_FAIL", error: err, seq });
      }
    },
    [state.overrides.documentsDir, state.environment, state.bootstrap, handleSelectSave],
  );

  const handleResolveEnvironment = useCallback(
    (overrides: SessionState["overrides"]) => {
      if (resolveDebounceTimerRef.current) {
        clearTimeout(resolveDebounceTimerRef.current);
      }

      const seq = ++envSeqRef.current;
      ++scanSeqRef.current;
      dispatch({ type: "RESOLVE_ENV_START", seq });
      resolveDebounceTimerRef.current = setTimeout(async () => {
        try {
          const request: {
            gameRoot?: string;
            documentsDir?: string;
            launcherDb?: string;
          } = {};
          if (overrides.gameRoot) request.gameRoot = overrides.gameRoot;
          if (overrides.documentsDir) request.documentsDir = overrides.documentsDir;
          if (overrides.launcherDb) request.launcherDb = overrides.launcherDb;

          const env = await resolveEnvironment(request);
          if (seq !== envSeqRef.current) return;

          dispatch({ type: "RESOLVE_ENV_SUCCESS", environment: env, seq });
          savePersistedData({ overrides });

          handleScanLibrary(
            saveSourceRef.current,
            env.documentsDir,
            env.steamLibraries,
          );
        } catch (err: any) {
          if (seq !== envSeqRef.current) return;
          dispatch({ type: "RESOLVE_ENV_FAIL", error: err, seq });
        }
      }, 300);
    },
    [handleScanLibrary],
  );

  useEffect(() => {
    let mounted = true;
    const envSeq = envSeqRef.current;
    const scanSeq = scanSeqRef.current;

    async function init() {
      dispatch({ type: "BOOTSTRAP_START" });

      if (!mounted) return;

      setCurrentTheme(persisted.uiTheme);
      applyTheme(persisted.uiTheme);

      try {
        const bootstrap = await getBootstrapData();
        if (!mounted) return;
        dispatch({ type: "BOOTSTRAP_SUCCESS", data: bootstrap });
        if (envSeq !== envSeqRef.current) return;

        const hasOverrides =
          persisted.overrides.gameRoot !== null ||
          persisted.overrides.documentsDir !== null ||
          persisted.overrides.launcherDb !== null;

        if (hasOverrides) {
          handleResolveEnvironment(persisted.overrides);
        } else if (!bootstrap.environmentError && scanSeq === scanSeqRef.current) {
          handleScanLibrary(
            saveSourceRef.current,
            bootstrap.environment.documentsDir,
            bootstrap.environment.steamLibraries,
          );
        }
      } catch (err: any) {
        if (!mounted) return;
        dispatch({ type: "BOOTSTRAP_FAIL", error: err });
      }
    }

    init();

    return () => {
      mounted = false;
      ++envSeqRef.current;
      ++scanSeqRef.current;
      ++inspectSeqRef.current;
      if (resolveDebounceTimerRef.current) {
        clearTimeout(resolveDebounceTimerRef.current);
      }
    };
  }, []);

  const handleInspectWithCountry = useCallback(
    (countryId: number) => {
      if (state.selectedSave) {
        handleInspectSave(state.selectedSave, countryId);
      }
    },
    [state.selectedSave, handleInspectSave],
  );

  const handleStartGeneration = useCallback(async () => {
    if (!canStartGeneration(state) || !state.selectedSavePath || !state.environment) {
      return;
    }

    dispatch({ type: "START_GENERATION" });

    const request: {
      reportLocale: AppLocale;
      saveFile: string;
      countryId?: number;
      settings: {
        stellarisRoot: string;
        launcherDb: string;
        languages: typeof state.settings.languages;
        unknownStrategy: typeof state.settings.unknownStrategy;
        swapUnknownStrategy: typeof state.settings.swapUnknownStrategy;
      };
    } = {
      reportLocale: currentLocale(),
      saveFile: state.selectedSavePath,
      settings: {
        stellarisRoot: state.environment.gameRoot,
        launcherDb: state.environment.launcherDb,
        languages: state.settings.languages,
        unknownStrategy: state.settings.unknownStrategy,
        swapUnknownStrategy: state.settings.swapUnknownStrategy,
      },
    };
    if (state.selectedCountryId !== null) {
      request.countryId = state.selectedCountryId;
    }

    const handleProgress = (progress: GenerationProgressDto) => {
      dispatch({ type: "GENERATION_PROGRESS", progress });
    };

    try {
      const result = await runGeneration(request, handleProgress);
      dispatch({ type: "GENERATION_DONE", result });
    } catch (err: any) {
      if (err.code === "GENERATION_CANCELLED") {
        dispatch({ type: "GENERATION_CANCELLED" });
      } else {
        dispatch({ type: "GENERATION_FAIL", error: err });
      }
    }
  }, [state]);

  const handleCancelGeneration = useCallback(async () => {
    dispatch({ type: "CANCEL_GENERATION_START" });
    try {
      await cancelGeneration();
    } catch (err) {
      console.error("Cancel generation failed:", err);
    }
  }, []);

  const handleChangeTheme = (theme: ThemePreference) => {
    setCurrentTheme(theme);
    applyTheme(theme);
    savePersistedData({ uiTheme: theme });
  };

  useEffect(() => {
    savePersistedData({
      languages: state.settings.languages,
      unknownStrategy: state.settings.unknownStrategy,
      swapUnknownStrategy: state.settings.swapUnknownStrategy,
    });
  }, [state.settings]);

  return (
    <AppShell
      state={state}
      onNavigateStep={(step: WizardStep) => dispatch({ type: "NAVIGATE_STEP", step })}
      onStartGeneration={handleStartGeneration}
      onCancelGeneration={handleCancelGeneration}
      onOpenOutputDirectory={openOutputDirectory}
      onChangeSave={() => {
        ++selectionSeqRef.current;
        ++inspectSeqRef.current;
        selectedSaveRef.current = null;
        dispatch({ type: "RESET_SAVE_SELECTION" });
      }}
      onChangeLocale={setLocalePreference}
      onChangeTheme={handleChangeTheme}
      currentTheme={currentTheme}
    >
      {state.bootstrap.status === "failed" && <ErrorAlert error={state.bootstrap.error} />}
      {(localeState.saveFailed || initialSaveFailed) && <p role="alert">{t($ => $.settings.localeSaveFailed)}</p>}
      {state.step === "settings" && (
        <EnvironmentAndSettingsPage
          state={state}
          dispatch={dispatch}
          onResolveEnvironment={handleResolveEnvironment}
          onRescanEnvironment={() => {
            if (state.bootstrap.status === "ready") {
              handleResolveEnvironment(state.overrides);
            }
          }}
        />
      )}

      {state.step === "saves" && (
        <SaveAndEmpirePage
          state={state}
          dispatch={dispatch}
          onScanLibrary={handleScanLibrary}
          onSelectSave={handleSelectSave}
          onInspectWithCountry={handleInspectWithCountry}
        />
      )}

      {state.step === "generate" && (
        state.run.status === "failed" ? (
          <GenerationErrorDialog
            error={state.run.error}
            onReturn={() => dispatch({ type: "DISMISS_GENERATION_ERROR" })}
          />
        ) : <GenerationPage state={state} />
      )}

      {state.step === "results" && <ResultsPage state={state} />}
    </AppShell>
  );
}
