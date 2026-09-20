import {
  AlertTriangle,
  BookOpen,
  CheckCircle2,
  Cpu,
  Database,
  FileCheck,
  FileText,
  FileWarning,
  FileX,
  GitCompare,
  ListTree,
  Sparkles,
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { DiagnosticSection } from "@/components/DiagnosticSection";
import { Panel } from "@/components/Panel";
import { StatCard } from "@/components/StatCard";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import type { SessionState } from "@/app/session";
import type { GenerationResultDto } from "@/ipc/bindings";
import { truncateMiddle } from "@/lib/format";
import { cn } from "@/lib/utils";

interface ResultsPageProps {
  state: SessionState;
}

export function ResultsPage({ state }: ResultsPageProps) {
  const { t } = useTranslation(["results", "common"]);

  const result: GenerationResultDto | null =
    state.run.status === "done" ? state.run.result : null;

  const [writtenOpen, setWrittenOpen] = useState(true);
  const [removedOpen, setRemovedOpen] = useState(false);
  const [failedOpen, setFailedOpen] = useState(true);

  if (!result) {
    return (
      <div className="flex h-full min-h-0 items-center justify-center text-body text-muted-foreground">
        {t("results:empty")}
      </div>
    );
  }

  const isSuccess = result.status === "success";
  const writtenCount = result.written.length;
  const removedCount = result.removed.length;
  const failedCount = result.failed.length;

  return (
    <div className="h-full min-h-0">
      <Panel title={t("results:title")} icon={Sparkles}>
        <div className="space-y-5">
          <div
            className={cn(
              "flex items-start gap-3 rounded-lg border p-4 transition-all",
              isSuccess
                ? "border-success/30 bg-success/10 text-success"
                : "border-warning/30 bg-warning/10 text-warning",
            )}
          >
            <div className="mt-0.5 shrink-0">
              {isSuccess ? (
                <CheckCircle2 className="size-5 text-success" />
              ) : (
                <AlertTriangle className="size-5 text-warning" />
              )}
            </div>
            <div className="space-y-0.5">
              <h3 className="text-title font-semibold text-foreground">
                {isSuccess
                  ? t("results:banner.success.title")
                  : t("results:banner.incomplete.title")}
              </h3>
              <p className="text-body text-muted-foreground">
                {isSuccess
                  ? t("results:banner.success.description", { writtenCount })
                  : t("results:banner.incomplete.description", {
                      failedCount,
                      writtenCount,
                    })}
              </p>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
            <StatCard
              label={t("results:stats.sources")}
              value={result.sourceCount}
              description={t("results:stats.sourcesDesc", {
                count: result.sourceCount,
              })}
              icon={<Database className="size-4" />}
            />
            <StatCard
              label={t("results:stats.totalTech")}
              value={result.technologyCount}
              description={t("results:stats.totalTechDesc")}
              icon={<Cpu className="size-4" />}
            />
            <StatCard
              label={t("results:stats.eligibleTech")}
              value={result.eligibleCount}
              description={t("results:stats.eligibleTechDesc")}
              icon={<ListTree className="size-4" />}
            />
            <StatCard
              label={t("results:stats.swaps")}
              value={`${result.swapMatched} / ${result.swapNoMatch} / ${result.swapUncertain}`}
              description={t("results:stats.swapsDesc", {
                matched: result.swapMatched,
                noMatch: result.swapNoMatch,
                uncertain: result.swapUncertain,
              })}
              icon={<GitCompare className="size-4" />}
            />
          </div>

          <section className="space-y-3">
            <h3 className="flex items-center gap-2 text-title font-semibold">
              <FileText className="size-4 text-muted-foreground" />
              <span>{t("results:files.title")}</span>
            </h3>

            {failedCount > 0 && (
              <Collapsible
                open={failedOpen}
                onOpenChange={setFailedOpen}
                className="rounded-md border border-destructive/40 bg-destructive/5 p-2.5 text-body"
              >
                <CollapsibleTrigger className="flex w-full items-center justify-between font-medium text-destructive">
                  <div className="flex items-center gap-2">
                    <FileWarning className="size-3.5" />
                    <span>{t("results:files.failed")}</span>
                  </div>
                  <Badge
                    variant="destructive"
                    className="h-4 px-1.5 text-meta tabular-nums"
                  >
                    {failedCount}
                  </Badge>
                </CollapsibleTrigger>
                <CollapsibleContent className="mt-2 space-y-1 pt-1 font-mono text-meta text-destructive">
                  {result.failed.map((file, i) => (
                    <div key={i} className="truncate">
                      {file}
                    </div>
                  ))}
                </CollapsibleContent>
              </Collapsible>
            )}

            {writtenCount > 0 && (
              <Collapsible
                open={writtenOpen}
                onOpenChange={setWrittenOpen}
                className="rounded-md bg-surface p-2.5 text-body"
              >
                <CollapsibleTrigger className="flex w-full items-center justify-between font-medium text-foreground">
                  <div className="flex items-center gap-2">
                    <FileCheck className="size-3.5 text-success" />
                    <span>{t("results:files.written")}</span>
                  </div>
                  <Badge
                    variant="secondary"
                    className="h-4 px-1.5 text-meta tabular-nums"
                  >
                    {writtenCount}
                  </Badge>
                </CollapsibleTrigger>
                <CollapsibleContent className="mt-2 max-h-40 overflow-y-auto pt-1 font-mono text-meta text-muted-foreground">
                  <div className="space-y-1 pr-2">
                    {result.written.map((file, i) => (
                      <div key={i} className="truncate">
                        {file}
                      </div>
                    ))}
                  </div>
                </CollapsibleContent>
              </Collapsible>
            )}

            {removedCount > 0 && (
              <Collapsible
                open={removedOpen}
                onOpenChange={setRemovedOpen}
                className="rounded-md bg-surface p-2.5 text-body"
              >
                <CollapsibleTrigger className="flex w-full items-center justify-between font-medium text-muted-foreground hover:text-foreground">
                  <div className="flex items-center gap-2">
                    <FileX className="size-3.5 text-muted-foreground" />
                    <span>{t("results:files.removed")}</span>
                  </div>
                  <Badge
                    variant="secondary"
                    className="h-4 px-1.5 text-meta tabular-nums"
                  >
                    {removedCount}
                  </Badge>
                </CollapsibleTrigger>
                <CollapsibleContent className="mt-2 space-y-1 pt-1 font-mono text-meta text-muted-foreground">
                  {result.removed.map((file, i) => (
                    <div key={i} className="truncate">
                      {file}
                    </div>
                  ))}
                </CollapsibleContent>
              </Collapsible>
            )}

            {result.reportPath && (
              <div className="flex items-center justify-between rounded-md bg-surface p-2.5 font-mono text-meta text-muted-foreground">
                <span className="text-foreground">{t("results:files.report")}:</span>
                <span className="max-w-md truncate">
                  {truncateMiddle(result.reportPath, 60)}
                </span>
              </div>
            )}
          </section>

          <section className="space-y-2">
            <h3 className="flex items-center gap-2 text-title font-semibold">
              <Sparkles className="size-4 text-muted-foreground" />
              <span>{t("results:diagnostics.title")}</span>
            </h3>
            <DiagnosticSection
              categoryKey="unknownConditions"
              items={result.diagnostics.unknownConditions}
            />
            <DiagnosticSection
              categoryKey="loadOrder"
              items={result.diagnostics.loadOrder}
            />
            <DiagnosticSection
              categoryKey="deferredConditions"
              items={result.diagnostics.deferredConditions}
            />
            <DiagnosticSection
              categoryKey="gameData"
              items={result.diagnostics.gameData}
            />
            <DiagnosticSection
              categoryKey="unhandledDefinitions"
              items={result.diagnostics.unhandledDefinitions}
            />
            <DiagnosticSection
              categoryKey="localisation"
              items={result.diagnostics.localisation}
            />
            <DiagnosticSection
              categoryKey="cycles"
              items={result.diagnostics.cycles}
            />
            <DiagnosticSection
              categoryKey="writeFailures"
              items={result.diagnostics.writeFailures}
            />
          </section>

          <section className="space-y-3">
            <h3 className="flex items-center gap-2 text-title font-semibold">
              <BookOpen className="size-4 text-muted-foreground" />
              <span>{t("results:guide.title")}</span>
            </h3>
            <ol className="list-inside list-decimal space-y-1.5 text-body leading-relaxed text-muted-foreground">
              <li>{t("results:guide.step1")}</li>
              <li>{t("results:guide.step2")}</li>
              <li>{t("results:guide.step3")}</li>
            </ol>
            <Alert className="mt-2 border-warning/40 bg-warning/10 text-body text-warning">
              <AlertTriangle className="size-4 text-warning" />
              <AlertTitle className="text-body font-semibold text-foreground">
                {t("results:guideLoadOrderTitle")}
              </AlertTitle>
              <AlertDescription className="text-meta text-muted-foreground">
                {t("results:guide.importantNote")}
              </AlertDescription>
            </Alert>
          </section>
        </div>
      </Panel>
    </div>
  );
}
