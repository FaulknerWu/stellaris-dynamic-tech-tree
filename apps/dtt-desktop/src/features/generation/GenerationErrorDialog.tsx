import { useState } from "react";
import { useTranslation } from "react-i18next";
import { AlertCircle, Copy } from "lucide-react";

import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import type { AppError } from "@/ipc/bindings";

interface GenerationErrorDialogProps {
  error: AppError;
  onReturn: () => void;
}

export function GenerationErrorDialog({ error, onReturn }: GenerationErrorDialogProps) {
  const { t } = useTranslation("generation");
  const [copyStatus, setCopyStatus] = useState<"idle" | "copying" | "copied" | "failed">("idle");
  const report = [
    t("failure.title"),
    `${t("failure.code")}: ${error.code}`,
    `${t("failure.message")}: ${error.message}`,
    error.context?.path ? `${t("failure.path")}: ${error.context.path}` : null,
    error.detail ? `${t("failure.detail")}:\n${error.detail}` : null,
  ].filter((part) => part !== null).join("\n\n");

  async function copyReport() {
    setCopyStatus("copying");
    try {
      await navigator.clipboard.writeText(report);
      setCopyStatus("copied");
    } catch {
      setCopyStatus("failed");
    }
  }

  return (
    <AlertDialog open onOpenChange={(open) => { if (!open) onReturn(); }}>
      <AlertDialogContent className="max-h-[90vh] overflow-y-auto data-[size=default]:max-w-[calc(100vw-2rem)] data-[size=default]:sm:max-w-xl">
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2">
            <AlertCircle className="size-5 text-destructive" />
            {t("failure.title")}
          </AlertDialogTitle>
          <AlertDialogDescription>{t("failure.description")}</AlertDialogDescription>
        </AlertDialogHeader>
        <pre tabIndex={0} className="max-h-[45vh] overflow-auto whitespace-pre-wrap break-words rounded border border-border-subtle bg-surface p-3 font-mono text-meta select-text">
          {report}
        </pre>
        <p role="status" aria-live="polite" className="text-body text-muted-foreground">
          {copyStatus === "copied" ? t("failure.copied") : copyStatus === "failed" ? t("failure.copyFailed") : null}
        </p>
        <AlertDialogFooter>
          <Button type="button" variant="outline" onClick={onReturn}>
            {t("failure.return")}
          </Button>
          <Button type="button" disabled={copyStatus === "copying"} onClick={copyReport}>
            <Copy className="size-4" />
            {t("failure.copy")}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
