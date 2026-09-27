import { Loader2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Panel } from "@/components/Panel";
import { StageList } from "@/components/StageList";
import { Progress } from "@/components/ui/progress";
import type { SessionState } from "@/app/session";
import type { GenerationStageDto } from "@/ipc/bindings";
import { cn } from "@/lib/utils";

interface GenerationPageProps {
  state: SessionState;
}

export function GenerationPage({ state }: GenerationPageProps) {
  const { t } = useTranslation(["generation", "common"]);

  const run = state.run;
  const isCancelling = run.status === "cancelling";
  const stage: GenerationStageDto =
    run.status === "running" || run.status === "cancelling"
      ? run.stage
      : "SAVE_PARSE";
  const percent: number =
    run.status === "running" || run.status === "cancelling" ? run.percent : 0;

  const [stageDuration, setStageDuration] = useState(0);

  useEffect(() => {
    setStageDuration(0);
    const interval = setInterval(() => {
      setStageDuration((prev) => prev + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, [stage]);

  const isLongWait = stageDuration >= 8;

  return (
    <div className="h-full min-h-0">
      <Panel
        title={t($ => $.running.title, { ns: "generation" })}
        icon={Loader2}
        bodyClassName="flex flex-col items-center justify-center"
      >
        <div className="w-full max-w-md space-y-6 text-center">
          <div className="space-y-2">
            <div className="inline-flex items-center gap-2 text-title font-semibold text-foreground">
              <Loader2
                className={cn(
                  "size-4 text-primary",
                  !isCancelling && "animate-spin",
                )}
              />
              <span>
                {isCancelling
                  ? t($ => $.running.cancelling, { ns: "generation" })
                  : t($ => $.stages[stage], { ns: "generation" })}
              </span>
            </div>
            <p className="text-body text-muted-foreground">
              {t($ => $.running.title, { ns: "generation" })}
            </p>
          </div>

          <div className="space-y-1.5">
            <Progress
              value={percent}
              className={cn(isCancelling && "animate-pulse opacity-70")}
            />
            <div className="flex justify-between font-mono text-meta text-muted-foreground">
              <span>{t($ => $.stages[stage], { ns: "generation" })}</span>
              <span className="font-medium tabular-nums">{percent}%</span>
            </div>
          </div>

          <div className="rounded-lg bg-surface p-4 text-left">
            <StageList currentStage={stage} isCancelling={isCancelling} />
          </div>

          {isLongWait && !isCancelling && (
            <p className="text-body text-muted-foreground animate-fadeIn">
              {t($ => $.running.longWaitHint, { ns: "generation" })}
            </p>
          )}
        </div>
      </Panel>
    </div>
  );
}
