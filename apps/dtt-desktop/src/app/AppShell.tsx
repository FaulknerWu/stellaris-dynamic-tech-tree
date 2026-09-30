import { registry, type AppLocale, type LocalePreference } from "@/i18n/locale";
import { useLocale } from "@/i18n/controller";
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
import { canStartGeneration, generationBlocker, hasValidEnvironment } from "./session";
import type { ThemePreference } from "./store";

interface AppShellProps {
  state: SessionState;
  onNavigateStep: (step: WizardStep) => void;
  onStartGeneration: () => void;
  onCancelGeneration: () => void;
  onOpenOutputDirectory: () => void;
  onChangeSave: () => void;
  onChangeLocale: (locale: LocalePreference) => void;
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
  const { t } = useTranslation(["common", "generation"]);
  const { preference } = useLocale();
  const [aboutOpen, setAboutOpen] = useState(false);

  const isGenerating =
    state.run.status === "running" || state.run.status === "cancelling";

  let disabledReason: string | null = null;
  if (state.step === "settings") {
    if (!hasValidEnvironment(state)) {
      disabledReason = t($ => $.hints.needEnvironment, { ns: "common" });
    } else if (state.settings.languages.length === 0) {
      disabledReason = t($ => $.barriers.needLanguage, { ns: "generation" });
    }
  } else if (state.step === "saves") {
    switch (generationBlocker(state)) {
      case "environment": disabledReason = t($ => $.hints.needEnvironment, { ns: "common" }); break;
      case "language": disabledReason = t($ => $.barriers.needLanguage, { ns: "generation" }); break;
      case "save": disabledReason = t($ => $.barriers.needSave, { ns: "generation" }); break;
      case "invalidSave": disabledReason = t($ => $.barriers.invalidSave, { ns: "generation" }); break;
      case "empire": disabledReason = t($ => $.barriers.needEmpire, { ns: "generation" }); break;
    }
  }

  const canProceedFromSettings =
    state.step === "settings" && hasValidEnvironment(state) && state.settings.languages.length > 0;

  const canStartFromSaves = canStartGeneration(state);

  const selectedSaveName =
    state.selectedSave?.metadata?.name ||
    state.selectedSave?.fileName ||
    t($ => $.hints.selectSave, { ns: "common" });

  let footerHint = "";
  if (state.step === "settings") {
    footerHint = t($ => $.hints.settingsAutosaved, { ns: "common" });
  } else if (state.step === "saves") {
    footerHint = t($ => $.hints.saveSelected, { ns: "common", name: selectedSaveName });
  } else if (state.step === "results") {
    footerHint = t($ => $.hints.generationReady, { ns: "common" });
  }

  return (
    <div className="grid h-screen w-screen grid-cols-[minmax(0,1fr)] grid-rows-[auto_minmax(0,1fr)_auto] overflow-hidden bg-background text-foreground select-none">
      <header className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-4 border-b border-border px-4 py-2.5 sm:px-6">
        <div className="flex items-center gap-1.5 text-body text-muted-foreground">
          <span>{t($ => $.feedback, { ns: "common", group: "1121150979" })}</span>

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
                  aria-label={t($ => $.settings.title, { ns: "common" })}
                >
                  <Settings className="size-4" />
                </Button>
              }
            />
            <DropdownMenuContent align="end" className="w-48 text-body">
              <DropdownMenuGroup>
                <DropdownMenuLabel>{t($ => $.settings.localePreference, { ns: "common" })}</DropdownMenuLabel>
                <DropdownMenuItem onClick={() => onChangeLocale("system")}>
                  {t($ => $.settings.themeSystem, { ns: "common" })} {preference === "system" ? "✓" : ""}
                </DropdownMenuItem>
                {(Object.keys(registry) as AppLocale[]).map((locale) => (
                  <DropdownMenuItem key={locale} onClick={() => onChangeLocale(locale)} className="justify-between">
                    <span>{registry[locale].nativeName}</span>{preference === locale && <span>✓</span>}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuGroup>

              <DropdownMenuSeparator />

              <DropdownMenuGroup>
                <DropdownMenuLabel>{t($ => $.settings.theme, { ns: "common" })}</DropdownMenuLabel>
                <DropdownMenuItem
                  onClick={() => onChangeTheme("light")}
                  className="justify-between"
                >
                  <div className="flex items-center gap-2">
                    <Sun className="size-3.5" />
                    <span>{t($ => $.settings.themeLight, { ns: "common" })}</span>
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
                    <span>{t($ => $.settings.themeDark, { ns: "common" })}</span>
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
                    <span>{t($ => $.settings.themeSystem, { ns: "common" })}</span>
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
                  <span>{t($ => $.settings.about, { ns: "common" })}</span>
                </div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>

      <main className="min-h-0 min-w-0 overflow-auto lg:overflow-hidden px-4 py-3.5 sm:px-6 sm:py-4">
        <div className="flex min-h-full min-w-0 flex-col gap-2 lg:h-full lg:min-h-0 [&>div:last-child]:min-h-0 [&>div:last-child]:flex-1">{children}</div>
      </main>

      <footer className="flex min-w-0 flex-wrap items-center justify-between gap-4 border-t border-border bg-card/70 px-4 py-3 sm:px-6 backdrop-blur-xs">
        <div className="flex min-w-0 items-center pr-4">
          <span className="text-body text-muted-foreground">
            {disabledReason ?? footerHint}
          </span>
        </div>

        <div className="flex min-w-0 flex-wrap items-center gap-2">
          {state.step === "settings" && (
            <Button
              type="button"
              disabled={!canProceedFromSettings}
              onClick={() => onNavigateStep("saves")}
              className="h-9 gap-2 px-5 text-body font-semibold"
            >
              <span>{t($ => $.actions.nextToSaves, { ns: "common" })}</span>
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
                <span>{t($ => $.actions.prevToSettings, { ns: "common" })}</span>
              </Button>

              <Button
                type="button"
                disabled={!canStartFromSaves}
                onClick={onStartGeneration}
                className="h-9 gap-2 px-5 text-body font-semibold"
              >
                <Play className="size-3.5 fill-current" />
                <span>{t($ => $.actions.startGeneration, { ns: "common" })}</span>
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
                  ? t($ => $.actions.cancelling, { ns: "common" })
                  : t($ => $.actions.cancel, { ns: "common" })}
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
                <span>{t($ => $.actions.changeSave, { ns: "common" })}</span>
              </Button>

              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => onNavigateStep("settings")}
                className="h-9 gap-1.5 text-body"
              >
                <span>{t($ => $.actions.modifySettings, { ns: "common" })}</span>
              </Button>

              <Button
                type="button"
                onClick={onOpenOutputDirectory}
                className="h-9 gap-1.5 px-4 text-body font-semibold"
              >
                <FolderOpen className="size-3.5" />
                <span>{t($ => $.actions.openOutputDirectory, { ns: "common" })}</span>
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
              <span>{t($ => $.settings.about, { ns: "common" })}</span>
            </AlertDialogTitle>
            <AlertDialogDescription className="pt-2 text-body leading-relaxed">
              {t($ => $.settings.aboutText, { ns: "common" })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="space-y-1 border-t border-border-subtle py-2 font-mono text-meta text-muted-foreground">
            <div>
              {t($ => $.settings.version, { ns: "common" })}: 0.1.0
            </div>
            <div>
              {t($ => $.settings.license, { ns: "common" })}: MIT / Apache-2.0
            </div>
          </div>
          <AlertDialogFooter>
            <AlertDialogAction onClick={() => setAboutOpen(false)}>
              {t($ => $.actions.close, { ns: "common" })}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
