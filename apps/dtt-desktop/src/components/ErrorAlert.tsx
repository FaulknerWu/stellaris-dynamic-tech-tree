import { errorTitle } from "@/i18n/messages";
import { AlertCircle, AlertTriangle, ChevronRight, Info } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import type { AppError } from "@/ipc/bindings";
import { cn } from "@/lib/utils";

interface ErrorAlertProps {
  error: AppError;
  onRemedy?: () => void;
  remedyText?: string;
  className?: string;
}

export function ErrorAlert({
  error,
  onRemedy,
  remedyText,
  className,
}: ErrorAlertProps) {
  const { t } = useTranslation("errors");
  const [detailOpen, setDetailOpen] = useState(false);

  const isWarning =
    error.code === "GENERATION_BUSY" ||
    error.code === "UNSUPPORTED_BINARY_SAVE";

  const isNeutral = error.code === "GENERATION_CANCELLED";

  const hint = t($ => $.codeHints[error.code]);

  return (
    <Alert
      variant={isNeutral ? "default" : isWarning ? "default" : "destructive"}
      className={cn(
        "text-body transition-all",
        isWarning && "border-warning/50 bg-warning/10 text-warning dark:border-warning/40",
        isNeutral && "border-border-subtle bg-muted text-foreground",
        className,
      )}
    >
      <div className="flex items-start gap-2.5">
        <div className="mt-0.5 shrink-0">
          {isNeutral ? (
            <Info className="size-4 text-muted-foreground" />
          ) : isWarning ? (
            <AlertTriangle className="size-4 text-warning" />
          ) : (
            <AlertCircle className="size-4 text-destructive" />
          )}
        </div>

        <div className="min-w-0 flex-1 space-y-1">
          <AlertTitle className="text-body font-semibold leading-none tracking-tight">
            {errorTitle(error)}
          </AlertTitle>

          {hint && (
            <AlertDescription className="text-meta leading-normal text-muted-foreground">
              {hint}
            </AlertDescription>
          )}

          {error.context?.path && (
            <div className="font-mono text-meta break-all text-muted-foreground">
              {t($ => $.path, { path: error.context.path })}
            </div>
          )}

          {onRemedy && (
            <div className="pt-1">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={onRemedy}
                className="h-7 px-2.5 text-body"
              >
                {remedyText || t($ => $.remedy.retry)}
              </Button>
            </div>
          )}

          {error.technicalDetail && (
            <Collapsible
              open={detailOpen}
              onOpenChange={setDetailOpen}
              className="pt-1.5"
            >
              <CollapsibleTrigger className="flex items-center gap-1 text-meta text-muted-foreground hover:text-foreground">
                <ChevronRight
                  className={cn(
                    "size-3 transition-transform duration-200",
                    detailOpen && "rotate-90",
                  )}
                />
                <span>{t($ => $.details)}</span>
              </CollapsibleTrigger>
              <CollapsibleContent className="mt-1">
                <pre className="max-h-36 overflow-auto whitespace-pre-wrap rounded border border-border-subtle bg-surface p-2 font-mono text-meta leading-relaxed text-muted-foreground select-text">
                  {error.technicalDetail}
                </pre>
              </CollapsibleContent>
            </Collapsible>
          )}
        </div>
      </div>
    </Alert>
  );
}
