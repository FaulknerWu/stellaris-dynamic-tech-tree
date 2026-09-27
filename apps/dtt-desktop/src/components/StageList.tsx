import { Check, Circle, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { GenerationStageDto } from "@/ipc/bindings";
import { cn } from "@/lib/utils";

interface StageListProps {
  currentStage: GenerationStageDto;
  isCancelling?: boolean;
}

const STAGES: GenerationStageDto[] = [
  "SAVE_PARSE",
  "LOAD_ORDER",
  "INGEST_TECH",
  "RELATIONS",
  "INGEST_L10N",
  "RENDER",
  "CYCLES",
  "WRITE_OUTPUT",
  "DONE",
];

export function StageList({ currentStage, isCancelling }: StageListProps) {
  const { t } = useTranslation("generation");

  const currentIndex = STAGES.indexOf(currentStage);

  return (
    <div className="space-y-2 py-2">
      {STAGES.map((stage, idx) => {
        const isDone = idx < currentIndex || currentStage === "DONE";
        const isCurrent = idx === currentIndex && currentStage !== "DONE";

        return (
          <div
            key={stage}
            className={cn(
              "flex items-center gap-2.5 text-body transition-colors",
              isCurrent && "font-medium text-foreground",
              isDone && "text-muted-foreground",
              !isDone && !isCurrent && "text-muted-foreground",
            )}
          >
            <div className="flex size-4 shrink-0 items-center justify-center">
              {isDone ? (
                <Check className="size-3.5 text-success stroke-[2.5]" />
              ) : isCurrent ? (
                <Loader2
                  className={cn(
                    "size-3.5 text-primary",
                    !isCancelling && "animate-spin",
                  )}
                />
              ) : (
                <Circle className="size-2.5 text-muted-foreground stroke-[2]" />
              )}
            </div>
            <span>{t($ => $.stages[stage])}</span>
          </div>
        );
      })}
    </div>
  );
}
