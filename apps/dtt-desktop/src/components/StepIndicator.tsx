import { Check, Loader2 } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import type { WizardStep } from "@/app/session";
import { cn } from "@/lib/utils";

interface StepIndicatorProps {
  currentStep: WizardStep;
  isGenerating: boolean;
  onStepClick?: (step: WizardStep) => void;
}

const STEPS: { id: WizardStep; number: number }[] = [
  { id: "settings", number: 1 },
  { id: "saves", number: 2 },
  { id: "generate", number: 3 },
  { id: "results", number: 4 },
];

export function StepIndicator({
  currentStep,
  isGenerating,
  onStepClick,
}: StepIndicatorProps) {
  const { t } = useTranslation("common");

  const stepOrder: Record<WizardStep, number> = {
    settings: 1,
    saves: 2,
    generate: 3,
    results: 4,
  };

  const currentOrder = stepOrder[currentStep];

  const getStepStatus = (stepId: WizardStep) => {
    const order = stepOrder[stepId];
    if (stepId === currentStep) {
      return "current";
    }
    if (order < currentOrder) {
      return "completed";
    }
    return "pending";
  };

  const isClickable = (stepId: WizardStep) => {
    if (isGenerating) {
      return false;
    }
    if (stepId === "generate") {
      return false;
    }
    if (currentStep === "results") {
      return stepId === "settings" || stepId === "saves" || stepId === "results";
    }
    if (currentStep === "saves") {
      return stepId === "settings" || stepId === "saves";
    }
    if (currentStep === "settings") {
      return stepId === "settings";
    }
    return false;
  };

  return (
    <nav aria-label={t($ => $.wizard.ariaLabel)} className="flex items-center gap-1.5 sm:gap-2.5">
      {STEPS.map((step, index) => {
        const status = getStepStatus(step.id);
        const clickable = isClickable(step.id);

        return (
          <React.Fragment key={step.id}>
            {index > 0 && (
              <div
                className={cn(
                  "h-0.5 w-4 sm:w-6 rounded-full transition-colors duration-300",
                  status === "completed" || (status === "current" && index <= currentOrder - 1)
                    ? "bg-primary"
                    : "bg-muted",
                )}
                aria-hidden="true"
              />
            )}
            <button
              type="button"
              disabled={!clickable}
              onClick={() => clickable && onStepClick?.(step.id)}
              className={cn(
                "group flex items-center gap-1.5 rounded-full px-1.5 py-1 text-body font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                clickable ? "cursor-pointer hover:opacity-80" : "cursor-default",
              )}
              aria-current={status === "current" ? "step" : undefined}
              aria-label={t($ => $.steps[step.id])}
            >
              <div
                className={cn(
                  "relative flex size-6 shrink-0 items-center justify-center rounded-full text-meta font-semibold transition-all",
                  status === "completed" && "bg-primary text-primary-foreground",
                  status === "current" && "border-2 border-primary text-primary ring-2 ring-ring/30",
                  status === "pending" && "border border-border bg-muted text-muted-foreground",
                )}
              >
                {status === "completed" ? (
                  <Check className="size-3 stroke-[3]" />
                ) : status === "current" && isGenerating && step.id === "generate" ? (
                  <Loader2 className="size-3 animate-spin" />
                ) : (
                  <span>{step.number}</span>
                )}
              </div>
              <span
                className={cn(
                  "hidden text-body xl:inline whitespace-nowrap",
                  status === "current" ? "font-medium text-foreground" : "text-muted-foreground",
                )}
              >
                {t($ => $.steps[step.id])}
              </span>
            </button>
          </React.Fragment>
        );
      })}
    </nav>
  );
}
