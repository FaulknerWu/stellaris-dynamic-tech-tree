import { open } from "@tauri-apps/plugin-dialog";
import {
  Check,
  Layers,
  Pencil,
  RefreshCw,
  RotateCcw,
  SlidersHorizontal,
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { ErrorAlert } from "@/components/ErrorAlert";
import { Panel } from "@/components/Panel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldTitle,
} from "@/components/ui/field";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import type { SessionAction, SessionState } from "@/app/session";
import type {
  SupportedLanguageDto,
  SwapUnknownStrategyDto,
  UnknownStrategyDto,
} from "@/ipc/bindings";
import { cn } from "@/lib/utils";

interface EnvironmentAndSettingsPageProps {
  state: SessionState;
  dispatch: React.Dispatch<SessionAction>;
  onResolveEnvironment: (overrides: SessionState["overrides"]) => void;
  onRescanEnvironment: () => void;
}

const ALL_LANGUAGES: SupportedLanguageDto[] = [
  "simp_chinese",
  "english",
  "french",
  "german",
  "spanish",
  "russian",
  "japanese",
  "korean",
  "polish",
  "braz_por",
];

const UNKNOWN_STRATEGY_KEYS: {
  id: UnknownStrategyDto;
  key: "includeFlagged" | "excludeStrict" | "error";
}[] = [
  { id: "include_flagged", key: "includeFlagged" },
  { id: "exclude_strict", key: "excludeStrict" },
  { id: "error", key: "error" },
];

const SWAP_STRATEGY_KEYS: {
  id: SwapUnknownStrategyDto;
  key: "keepBase" | "error";
}[] = [
  { id: "keep_base", key: "keepBase" },
  { id: "error", key: "error" },
];

export function EnvironmentAndSettingsPage({
  state,
  dispatch,
  onResolveEnvironment,
  onRescanEnvironment,
}: EnvironmentAndSettingsPageProps) {
  const { t } = useTranslation(["environment", "generation", "common"]);

  const [minLangAlert, setMinLangAlert] = useState(false);

  const detected =
    state.bootstrap.status === "ready"
      ? state.bootstrap.data.environment
      : null;

  const effectiveGameRoot =
    state.overrides.gameRoot ??
    state.environment?.gameRoot ??
    detected?.gameRoot ??
    "";

  const effectiveDocsDir =
    state.overrides.documentsDir ??
    state.environment?.documentsDir ??
    detected?.documentsDir ??
    "";

  const effectiveLauncherDb =
    state.overrides.launcherDb ??
    state.environment?.launcherDb ??
    detected?.launcherDb ??
    "";

  const outputDirectory =
    state.bootstrap.status === "ready"
      ? state.bootstrap.data.outputDirectory
      : "";

  const handleBrowsePath = async (
    key: "gameRoot" | "documentsDir" | "launcherDb",
    dialogType: "directory" | "file",
    title: string,
    filters?: { name: string; extensions: string[] }[],
  ) => {
    try {
      if (dialogType === "directory") {
        const options: Parameters<typeof open>[0] = {
          directory: true,
          multiple: false,
          title,
        };
        const currentValue =
          key === "gameRoot"
            ? effectiveGameRoot
            : key === "documentsDir"
              ? effectiveDocsDir
              : effectiveLauncherDb;
        if (currentValue) options.defaultPath = currentValue;

        const selected = await open(options);
        if (typeof selected === "string") {
          const nextOverrides = { ...state.overrides, [key]: selected };
          dispatch({ type: "SET_OVERRIDE", key, value: selected });
          onResolveEnvironment(nextOverrides);
        }
      } else {
        const options: Parameters<typeof open>[0] = {
          directory: false,
          multiple: false,
          title,
        };
        if (filters) options.filters = filters;
        if (effectiveLauncherDb) options.defaultPath = effectiveLauncherDb;

        const selected = await open(options);
        if (typeof selected === "string") {
          const nextOverrides = { ...state.overrides, [key]: selected };
          dispatch({ type: "SET_OVERRIDE", key, value: selected });
          onResolveEnvironment(nextOverrides);
        }
      }
    } catch (err) {
      console.error("Browse path error:", err);
    }
  };

  const handleResetPath = (key: "gameRoot" | "documentsDir" | "launcherDb") => {
    const nextOverrides = { ...state.overrides, [key]: null };
    dispatch({ type: "SET_OVERRIDE", key, value: null });
    onResolveEnvironment(nextOverrides);
  };

  const handleToggleLanguage = (lang: SupportedLanguageDto) => {
    const current = state.settings.languages;
    if (current.includes(lang)) {
      if (current.length <= 1) {
        setMinLangAlert(true);
        setTimeout(() => setMinLangAlert(false), 3000);
        return;
      }
      dispatch({
        type: "SET_LANGUAGES",
        languages: current.filter((l) => l !== lang),
      });
    } else {
      setMinLangAlert(false);
      dispatch({
        type: "SET_LANGUAGES",
        languages: [...current, lang],
      });
    }
  };

  const handleUnknownStrategyChange = (value: unknown) => {
    if (
      value === "include_flagged" ||
      value === "exclude_strict" ||
      value === "error"
    ) {
      dispatch({ type: "SET_UNKNOWN_STRATEGY", strategy: value });
    }
  };

  const handleSwapStrategyChange = (value: unknown) => {
    if (value === "keep_base" || value === "error") {
      dispatch({ type: "SET_SWAP_UNKNOWN_STRATEGY", strategy: value });
    }
  };

  const isGameRootReady = !!effectiveGameRoot && !state.environmentError;
  const isDocsDirReady = !!effectiveDocsDir && !state.environmentError;
  const isLauncherDbReady = !!effectiveLauncherDb && !state.environmentError;

  return (
    <div className="grid h-full min-h-0 grid-cols-1 gap-4 lg:grid-cols-2">
      <Panel
        title={t("environment:pathsTitle")}
        icon={Layers}
        action={
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={onRescanEnvironment}
            className="h-7 shrink-0 gap-1.5 text-body font-normal"
          >
            <RefreshCw className="size-3" />
            <span>{t("common:actions.rescanEnvironment")}</span>
          </Button>
        }
      >
        {state.environmentError && (
          <div className="mb-2.5">
            <ErrorAlert error={state.environmentError} />
          </div>
        )}

        <div className="space-y-2.5">
          <PathRow
            label={t("environment:paths.gameRoot")}
            value={effectiveGameRoot}
            placeholder={t("environment:placeholders.gameRoot")}
            ready={isGameRootReady}
            isOverridden={state.overrides.gameRoot !== null}
            onReset={() => handleResetPath("gameRoot")}
            onBrowse={() =>
              handleBrowsePath(
                "gameRoot",
                "directory",
                t("environment:dialog.selectGameRoot"),
              )
            }
          />
          <PathRow
            label={t("environment:paths.documentsDir")}
            value={effectiveDocsDir}
            placeholder={t("environment:placeholders.documentsDir")}
            ready={isDocsDirReady}
            isOverridden={state.overrides.documentsDir !== null}
            onReset={() => handleResetPath("documentsDir")}
            onBrowse={() =>
              handleBrowsePath(
                "documentsDir",
                "directory",
                t("environment:dialog.selectDocumentsDir"),
              )
            }
          />
          <PathRow
            label={t("environment:paths.launcherDb")}
            value={effectiveLauncherDb}
            placeholder={t("environment:placeholders.launcherDb")}
            ready={isLauncherDbReady}
            isOverridden={state.overrides.launcherDb !== null}
            onReset={() => handleResetPath("launcherDb")}
            onBrowse={() =>
              handleBrowsePath(
                "launcherDb",
                "file",
                t("environment:dialog.selectLauncherDb"),
                [
                  {
                    name: t("environment:dialog.sqliteFilter"),
                    extensions: ["sqlite", "db"],
                  },
                ],
              )
            }
          />

          <div className="space-y-1 rounded-lg bg-surface p-2.5">
            <div className="flex items-center justify-between">
              <span className="text-body font-medium text-foreground">
                {t("environment:paths.outputDir")}
              </span>
              <Badge variant="secondary" className="h-4 text-meta font-normal">
                {t("environment:outputDirBadge")}
              </Badge>
            </div>
            <Input
              readOnly
              value={outputDirectory}
              placeholder={t("environment:placeholders.outputDir")}
              title={outputDirectory || undefined}
              aria-label={t("environment:paths.outputDir")}
              className="h-7 bg-background font-mono text-meta text-info"
            />
          </div>
        </div>
      </Panel>

      <Panel title={t("environment:settingsTitle")} icon={SlidersHorizontal}>
        <div className="space-y-3">
          <Field>
            <div className="flex items-center justify-between gap-2">
              <FieldTitle className="text-body text-foreground">
                {t("generation:outputLanguages.title")}
              </FieldTitle>
              <span className="text-meta text-muted-foreground">
                {t("generation:outputLanguages.selectedCount", {
                  count: state.settings.languages.length,
                })}
              </span>
            </div>
            <FieldDescription className="text-meta">
              {t("generation:outputLanguages.description")}
            </FieldDescription>

            <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-4">
              {ALL_LANGUAGES.map((lang) => {
                const isChecked = state.settings.languages.includes(lang);
                return (
                  <label
                    key={lang}
                    className={cn(
                      "relative flex cursor-pointer items-center justify-center rounded-md border px-2.5 py-1.5 text-body transition-all select-none",
                      "focus-within:border-ring focus-within:ring-3 focus-within:ring-ring/50",
                      isChecked
                        ? "border-primary bg-muted font-medium text-foreground"
                        : "border-border-subtle bg-card text-muted-foreground hover:bg-muted",
                    )}
                  >
                    <Checkbox
                      checked={isChecked}
                      onCheckedChange={() => handleToggleLanguage(lang)}
                      className="sr-only absolute size-px overflow-hidden"
                    />
                    <span className="min-w-0 truncate text-center text-body font-medium leading-none">
                      {t(`generation:outputLanguages.items.${lang}`)}
                    </span>
                    {isChecked && (
                      <Check className="absolute right-1.5 size-3.5 shrink-0 stroke-[3] text-primary" />
                    )}
                  </label>
                );
              })}
            </div>

            {minLangAlert ? (
              <FieldError className="text-meta animate-fadeIn">
                {t("generation:outputLanguages.hint")}
              </FieldError>
            ) : null}
          </Field>

          <Field>
            <FieldTitle className="text-body text-foreground">
              {t("generation:advanced.unknownStrategy.title")}
            </FieldTitle>
            <FieldDescription className="text-meta">
              {t("generation:advanced.unknownStrategy.description")}
            </FieldDescription>
            <RadioGroup
              value={state.settings.unknownStrategy}
              onValueChange={handleUnknownStrategyChange}
              className="gap-1.5"
            >
              {UNKNOWN_STRATEGY_KEYS.map((item) => (
                <StrategyChoice
                  key={item.id}
                  value={item.id}
                  selected={state.settings.unknownStrategy === item.id}
                  label={t(
                    `generation:advanced.unknownStrategy.${item.key}.label`,
                  )}
                />
              ))}
            </RadioGroup>
          </Field>

          <Field>
            <FieldTitle className="text-body text-foreground">
              {t("generation:advanced.swapUnknownStrategy.title")}
            </FieldTitle>
            <FieldDescription className="text-meta">
              {t("generation:advanced.swapUnknownStrategy.description")}
            </FieldDescription>
            <RadioGroup
              value={state.settings.swapUnknownStrategy}
              onValueChange={handleSwapStrategyChange}
              className="gap-1.5"
            >
              {SWAP_STRATEGY_KEYS.map((item) => (
                <StrategyChoice
                  key={item.id}
                  value={item.id}
                  selected={state.settings.swapUnknownStrategy === item.id}
                  label={t(
                    `generation:advanced.swapUnknownStrategy.${item.key}.label`,
                  )}
                />
              ))}
            </RadioGroup>
          </Field>
        </div>
      </Panel>
    </div>
  );
}

function StrategyChoice({
  value,
  selected,
  label,
}: {
  value: string;
  selected: boolean;
  label: string;
}) {
  return (
    <label
      className={cn(
        "relative flex w-full cursor-pointer items-center justify-start rounded-lg border py-2 pl-3 pr-8 text-left text-body transition-all select-none",
        "has-[:focus-visible]:border-ring has-[:focus-visible]:ring-3 has-[:focus-visible]:ring-ring/50",
        selected
          ? "border-primary bg-muted font-medium"
          : "border-border-subtle bg-card hover:bg-muted",
      )}
    >
      <RadioGroupItem value={value} className="sr-only absolute" />
      <span className="min-w-0 font-medium text-foreground">{label}</span>
      {selected ? (
        <Check
          className="absolute right-3 size-3.5 shrink-0 stroke-[3] text-primary"
          aria-hidden="true"
        />
      ) : null}
    </label>
  );
}

function PathRow({
  label,
  value,
  placeholder,
  ready,
  isOverridden,
  onReset,
  onBrowse,
}: {
  label: string;
  value: string;
  placeholder: string;
  ready: boolean;
  isOverridden: boolean;
  onReset: () => void;
  onBrowse: () => void;
}) {
  const { t } = useTranslation("common");

  return (
    <div className="space-y-1 rounded-lg bg-surface p-2.5">
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2">
          <span className="text-body font-medium text-foreground">{label}</span>
          {ready ? (
            <Badge variant="success" className="h-4 px-1.5 text-meta font-normal">
              {t("status.ready")}
            </Badge>
          ) : (
            <Badge
              variant="destructive"
              className="h-4 px-1.5 text-meta font-normal"
            >
              {t("status.incomplete")}
            </Badge>
          )}
        </div>

        <div className="flex items-center gap-1">
          {isOverridden && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={onReset}
              className="h-6 gap-1 px-1.5 text-meta text-muted-foreground hover:text-foreground"
              title={t("actions.reset")}
            >
              <RotateCcw className="size-3" />
              <span>{t("actions.restore")}</span>
            </Button>
          )}
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={onBrowse}
            className="h-6 gap-1 px-2 text-meta"
          >
            <Pencil className="size-3" />
            <span>{t("actions.modify")}</span>
          </Button>
        </div>
      </div>

      <Input
        readOnly
        value={value}
        placeholder={placeholder}
        title={value || undefined}
        aria-label={label}
        className="h-7 bg-background font-mono text-meta"
      />
    </div>
  );
}
