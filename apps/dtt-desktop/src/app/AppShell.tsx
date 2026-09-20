import {
  ArrowLeft,
  ArrowRight,
  FolderOpen,
  Info,
  Laptop,
  Moon,
  Play,
  RotateCcw,
  Settings,
  Sparkles,
  Sun,
  X,
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { StepIndicator } from "@/components/StepIndicator";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { SessionState, WizardStep } from "./session";
import type { ThemePreference, UiLocale } from "./store";

interface AppShellProps {
  state: SessionState;
  onNavigateStep: (step: WizardStep) => void;
  onStartGeneration: () => void;
  onCancelGeneration: () => void;
  onOpenOutputDirectory: () => void;
  onChangeSave: () => void;
  onChangeLocale: (locale: UiLocale) => void;
  onChangeTheme: (theme: ThemePreference) => void;
  currentTheme: ThemePreference;
  children: React.ReactNode;
}

export function AppShell({
  state,
  onNavigateStep,
  onStartGeneration,
  onCancelGeneration,
  onOpenOutputDirectory,
  onChangeSave,
  onChangeLocale,
  onChangeTheme,
  currentTheme,
  children,
}: AppShellProps) {
  const { t, i18n } = useTranslation(["common", "generation"]);
  const [aboutOpen, setAboutOpen] = useState(false);

  const isGenerating =
    state.run.status === "running" || state.run.status === "cancelling";

  let disabledReason: string | null = null;
  if (state.step === "settings") {
    const hasDetected =
      state.bootstrap.status === "ready" &&
      !!state.bootstrap.data.environment.gameRoot &&
      !!state.bootstrap.data.environment.documentsDir;
    if (!state.environment && !hasDetected) {
      disabledReason = t("common:hints.needEnvironment");
    } else if (state.settings.languages.length === 0) {
      disabledReason = t("generation:barriers.needLanguage");
    }
  } else if (state.step === "saves") {
    if (!state.selectedSavePath) {
      disabledReason = t("generation:barriers.needSave");
    } else if (state.selectedSave?.state !== "text") {
      disabledReason = t("generation:barriers.invalidSave");
    } else if (!state.inspection?.snapshot) {
      disabledReason = t("generation:barriers.needEmpire");
    }
  }

  const canProceedFromSettings =
    state.step === "settings" && state.settings.languages.length > 0;

  const canStartFromSaves =
    state.step === "saves" &&
    !!state.selectedSavePath &&
    state.selectedSave?.state === "text" &&
    !!state.inspection?.snapshot;

  const selectedSaveName =
    state.selectedSave?.metadata?.name ||
    state.selectedSave?.fileName ||
    t("common:hints.selectSave");

  let footerHint = "";
  if (state.step === "settings") {
    footerHint = t("common:hints.settingsAutosaved");
  } else if (state.step === "saves") {
    footerHint = t("common:hints.saveSelected", { name: selectedSaveName });
  } else if (state.step === "results") {
    footerHint = t("common:hints.generationReady");
  }

  return (
    <div className="grid h-screen w-screen grid-rows-[auto_1fr_auto] overflow-hidden bg-background text-foreground select-none">
      <header className="grid grid-cols-[1fr_auto_1fr] items-center gap-4 border-b border-border px-4 py-2.5 sm:px-6">
        <div className="flex items-center gap-1.5 text-body text-muted-foreground">
          <span>交流反馈请加群：</span>
          <span className="font-semibold text-foreground">1121150979</span>
        </div>

        <div className="justify-self-center">
          <StepIndicator
            currentStep={state.step}
            isGenerating={isGenerating}
            onStepClick={onNavigateStep}
          />
        </div>

        <div className="flex items-center gap-2 justify-self-end">
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="size-8 text-muted-foreground hover:text-foreground"
                  aria-label={t("common:settings.title")}
                >
                  <Settings className="size-4" />
                </Button>
              }
            />
            <DropdownMenuContent align="end" className="w-48 text-body">
              <DropdownMenuGroup>
                <DropdownMenuLabel>{t("common:settings.uiLocale")}</DropdownMenuLabel>
                <DropdownMenuItem
                  onClick={() => onChangeLocale("zh-CN")}
                  className="justify-between"
                >
                  <span>简体中文</span>
                  {i18n.language.startsWith("zh") && (
                    <span className="font-bold text-primary">✓</span>
                  )}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() => onChangeLocale("en")}
                  className="justify-between"
                >
                  <span>English</span>
                  {i18n.language.startsWith("en") && (
                    <span className="font-bold text-primary">✓</span>
                  )}
                </DropdownMenuItem>
              </DropdownMenuGroup>

              <DropdownMenuSeparator />

              <DropdownMenuGroup>
                <DropdownMenuLabel>{t("common:settings.theme")}</DropdownMenuLabel>
                <DropdownMenuItem
                  onClick={() => onChangeTheme("light")}
                  className="justify-between"
                >
                  <div className="flex items-center gap-2">
                    <Sun className="size-3.5" />
                    <span>{t("common:settings.themeLight")}</span>
                  </div>
                  {currentTheme === "light" && (
                    <span className="font-bold text-primary">✓</span>
                  )}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() => onChangeTheme("dark")}
                  className="justify-between"
                >
                  <div className="flex items-center gap-2">
                    <Moon className="size-3.5" />
                    <span>{t("common:settings.themeDark")}</span>
                  </div>
                  {currentTheme === "dark" && (
                    <span className="font-bold text-primary">✓</span>
                  )}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={() => onChangeTheme("system")}
                  className="justify-between"
                >
                  <div className="flex items-center gap-2">
                    <Laptop className="size-3.5" />
                    <span>{t("common:settings.themeSystem")}</span>
                  </div>
                  {currentTheme === "system" && (
                    <span className="font-bold text-primary">✓</span>
                  )}
                </DropdownMenuItem>
              </DropdownMenuGroup>

              <DropdownMenuSeparator />

              <DropdownMenuItem onClick={() => setAboutOpen(true)}>
                <div className="flex items-center gap-2">
                  <Info className="size-3.5" />
                  <span>{t("common:settings.about")}</span>
                </div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>

      <main className="min-h-0 overflow-hidden px-4 py-3.5 sm:px-6 sm:py-4">
        <div className="h-full min-h-0">{children}</div>
      </main>

      <footer className="flex items-center justify-between gap-4 border-t border-border bg-card/70 px-4 py-3 sm:px-6 backdrop-blur-xs">
        <div className="flex min-w-0 items-center pr-4">
          <span className="truncate text-body text-muted-foreground">
            {disabledReason ?? footerHint}
          </span>
        </div>

        <div className="flex shrink-0 items-center gap-2">
          {state.step === "settings" && (
            <Button
              type="button"
              disabled={!canProceedFromSettings}
              onClick={() => onNavigateStep("saves")}
              className="h-9 gap-2 px-5 text-body font-semibold"
            >
              <span>{t("common:actions.nextToSaves")}</span>
              <ArrowRight className="size-3.5" />
            </Button>
          )}

          {state.step === "saves" && (
            <>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => onNavigateStep("settings")}
                className="h-9 gap-1.5 text-body"
              >
                <ArrowLeft className="size-3.5" />
                <span>{t("common:actions.prevToSettings")}</span>
              </Button>

              <Button
                type="button"
                disabled={!canStartFromSaves}
                onClick={onStartGeneration}
                className="h-9 gap-2 px-5 text-body font-semibold"
              >
                <Play className="size-3.5 fill-current" />
                <span>{t("common:actions.startGeneration")}</span>
              </Button>
            </>
          )}

          {state.step === "generate" && isGenerating && (
            <Button
              type="button"
              variant="outline"
              disabled={state.run.status === "cancelling"}
              onClick={onCancelGeneration}
              className="h-9 gap-2 px-4 text-body"
            >
              <X className="size-3.5" />
              <span>
                {state.run.status === "cancelling"
                  ? t("common:actions.cancelling")
                  : t("common:actions.cancel")}
              </span>
            </Button>
          )}

          {state.step === "results" && (
            <>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={onChangeSave}
                className="h-9 gap-1.5 text-body text-muted-foreground hover:text-foreground"
              >
                <RotateCcw className="size-3.5" />
                <span>{t("common:actions.changeSave")}</span>
              </Button>

              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => onNavigateStep("settings")}
                className="h-9 gap-1.5 text-body"
              >
                <span>{t("common:actions.modifySettings")}</span>
              </Button>

              <Button
                type="button"
                onClick={onOpenOutputDirectory}
                className="h-9 gap-1.5 px-4 text-body font-semibold"
              >
                <FolderOpen className="size-3.5" />
                <span>{t("common:actions.openOutputDirectory")}</span>
              </Button>
            </>
          )}
        </div>
      </footer>

      <AlertDialog open={aboutOpen} onOpenChange={setAboutOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2 text-title font-semibold">
              <Sparkles className="size-4 text-primary" />
              <span>{t("common:settings.about")}</span>
            </AlertDialogTitle>
            <AlertDialogDescription className="pt-2 text-body leading-relaxed">
              {t("common:settings.aboutText")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="space-y-1 border-t border-border-subtle py-2 font-mono text-meta text-muted-foreground">
            <div>
              {t("common:settings.version")}: 0.1.0
            </div>
            <div>
              {t("common:settings.license")}: MIT / Apache-2.0
            </div>
          </div>
          <AlertDialogFooter>
            <AlertDialogAction onClick={() => setAboutOpen(false)}>
              {t("common:actions.close")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
