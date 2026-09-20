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
  sessionReducer,
  type SessionState,
  type WizardStep,
} from "./session";
import {
  DEFAULT_PERSISTED_DATA,
  loadPersistedData,
  savePersistedData,
  type ThemePreference,
  type UiLocale,
} from "./store";
import { applyTheme } from "./theme";

export function App() {
  const { i18n } = useTranslation();
  const [state, dispatch] = useReducer(sessionReducer, INITIAL_SESSION_STATE);
  const [currentTheme, setCurrentTheme] = useState<ThemePreference>("system");

  const scanSeqRef = useRef(0);
  const inspectSeqRef = useRef(0);
  const envSeqRef = useRef(0);
  const resolveDebounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    let mounted = true;

    async function init() {
      dispatch({ type: "BOOTSTRAP_START" });

      const persisted = await loadPersistedData();
      if (!mounted) return;

      dispatch({ type: "LOAD_PERSISTED", data: persisted });
      setCurrentTheme(persisted.uiTheme);
      applyTheme(persisted.uiTheme);
      if (persisted.uiLocale) {
        i18n.changeLanguage(persisted.uiLocale);
      }

      try {
        const bootstrap = await getBootstrapData();
        if (!mounted) return;
        dispatch({ type: "BOOTSTRAP_SUCCESS", data: bootstrap });

        const hasOverrides =
          persisted.overrides.gameRoot !== null ||
          persisted.overrides.documentsDir !== null ||
          persisted.overrides.launcherDb !== null;

        if (hasOverrides) {
          handleResolveEnvironment(persisted.overrides, bootstrap.environment.steamLibraries);
        } else if (!bootstrap.environmentError) {
          handleScanLibrary(
            persisted.saveSource,
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
      if (resolveDebounceTimerRef.current) {
        clearTimeout(resolveDebounceTimerRef.current);
      }
    };
  }, []);

  const handleResolveEnvironment = useCallback(
    (
      overrides: SessionState["overrides"],
      steamLibsFallback?: string[],
    ) => {
      if (resolveDebounceTimerRef.current) {
        clearTimeout(resolveDebounceTimerRef.current);
      }

      resolveDebounceTimerRef.current = setTimeout(async () => {
        const seq = ++envSeqRef.current;
        dispatch({ type: "RESOLVE_ENV_START", seq });

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

          dispatch({ type: "RESOLVE_ENV_SUCCESS", environment: env, seq });
          savePersistedData({ overrides });

          handleScanLibrary(
            state.saveSource,
            env.documentsDir,
            env.steamLibraries,
          );
        } catch (err: any) {
          dispatch({ type: "RESOLVE_ENV_FAIL", error: err, seq });
        }
      }, 300);
    },
    [state.saveSource],
  );

  const handleScanLibrary = useCallback(
    async (
      sourceOverride?: "local" | "steam_cloud",
      docsDirOverride?: string,
      steamLibsOverride?: string[],
    ) => {
      const source = sourceOverride ?? state.saveSource;
      const seq = ++scanSeqRef.current;
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
        dispatch({ type: "SCAN_LIBRARY_SUCCESS", library: lib, seq });
        savePersistedData({ saveSource: source });

        if (!state.selectedSavePath && lib.accounts.length > 0) {
          const firstSave = lib.accounts[0]?.campaigns[0]?.saves[0];
          if (firstSave && firstSave.state === "text") {
            dispatch({ type: "SELECT_SAVE", save: firstSave });
            handleInspectSave(firstSave);
          }
        }
      } catch (err: any) {
        dispatch({ type: "SCAN_LIBRARY_FAIL", error: err, seq });
      }
    },
    [state.saveSource, state.overrides.documentsDir, state.environment, state.bootstrap, state.selectedSavePath],
  );

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
        dispatch({ type: "INSPECT_SAVE_SUCCESS", inspection: inspected, seq });
      } catch (err: any) {
        dispatch({ type: "INSPECT_SAVE_FAIL", error: err, seq });
      }
    },
    [],
  );

  const handleSelectSave = useCallback(
    (save: SaveIndexDto) => {
      dispatch({ type: "SELECT_SAVE", save });
      if (save.state === "text") {
        handleInspectSave(save);
      }
    },
    [handleInspectSave],
  );

  const handleInspectWithCountry = useCallback(
    (countryId: number) => {
      if (state.selectedSave) {
        handleInspectSave(state.selectedSave, countryId);
      }
    },
    [state.selectedSave, handleInspectSave],
  );

  const handleStartGeneration = useCallback(async () => {
    if (!state.selectedSavePath || !state.environment) {
      return;
    }

    dispatch({ type: "START_GENERATION" });

    const request: {
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
      const result = await runGeneration(request as any, handleProgress);
      dispatch({ type: "GENERATION_DONE", result });
    } catch (err: any) {
      if (err.code === "GENERATION_CANCELLED") {
        dispatch({ type: "GENERATION_CANCELLED" });
      } else {
        dispatch({ type: "GENERATION_FAIL", error: err });
      }
    }
  }, [
    state.selectedSavePath,
    state.selectedCountryId,
    state.environment,
    state.settings,
  ]);

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

  const handleChangeLocale = (locale: UiLocale) => {
    i18n.changeLanguage(locale);
    savePersistedData({ uiLocale: locale });
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
        dispatch({ type: "RESET_SAVE_SELECTION" });
      }}
      onChangeLocale={handleChangeLocale}
      onChangeTheme={handleChangeTheme}
      currentTheme={currentTheme}
    >
      {state.step === "settings" && (
        <EnvironmentAndSettingsPage
          state={state}
          dispatch={dispatch}
          onResolveEnvironment={handleResolveEnvironment}
          onRescanEnvironment={() => {
            if (state.bootstrap.status === "ready") {
              handleResolveEnvironment(
                state.overrides,
                state.bootstrap.data.environment.steamLibraries,
              );
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
