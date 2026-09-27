import { diagnosticMessage } from "@/i18n/messages";
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
        {t($ => $.empty, { ns: "results" })}
      </div>
    );
  }

  const isSuccess = result.status === "success";
  const writtenCount = result.written.length;
  const removedCount = result.removed.length;
  const failedCount = result.failed.length;

  return (
    <div className="h-full min-h-0">
      <Panel title={t($ => $.title, { ns: "results" })} icon={Sparkles}>
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
                  ? t($ => $.banner.success.title, { ns: "results" })
                  : t($ => $.banner.incomplete.title, { ns: "results" })}
              </h3>
              <p className="text-body text-muted-foreground">
                {isSuccess
                  ? t($ => $.banner.success.description, { ns: "results", writtenCount })
                  : t($ => $.banner.incomplete.description, { ns: "results",
                      failedCount,
                      writtenCount,
                    })}
              </p>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
            <StatCard
              label={t($ => $.stats.sources, { ns: "results" })}
              value={result.sourceCount}
              description={t($ => $.stats.sourcesDesc, { ns: "results",
                count: result.sourceCount,
              })}
              icon={<Database className="size-4" />}
            />
            <StatCard
              label={t($ => $.stats.totalTech, { ns: "results" })}
              value={result.technologyCount}
              description={t($ => $.stats.totalTechDesc, { ns: "results" })}
              icon={<Cpu className="size-4" />}
            />
            <StatCard
              label={t($ => $.stats.eligibleTech, { ns: "results" })}
              value={result.eligibleCount}
              description={t($ => $.stats.eligibleTechDesc, { ns: "results" })}
              icon={<ListTree className="size-4" />}
            />
            <StatCard
              label={t($ => $.stats.swaps, { ns: "results" })}
              value={`${result.swapMatched} / ${result.swapNoMatch} / ${result.swapUncertain}`}
              description={t($ => $.stats.swapsDesc, { ns: "results",
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
              <span>{t($ => $.files.title, { ns: "results" })}</span>
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
                    <span>{t($ => $.files.failed, { ns: "results" })}</span>
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
                      {diagnosticMessage(file).summary}
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
                    <span>{t($ => $.files.written, { ns: "results" })}</span>
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
                    <span>{t($ => $.files.removed, { ns: "results" })}</span>
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
                <span className="text-foreground">{t($ => $.files.report, { ns: "results" })}:</span>
                <span className="max-w-md truncate">
                  {truncateMiddle(result.reportPath, 60)}
                </span>
              </div>
            )}
          </section>

          <section className="space-y-2">
            <h3 className="flex items-center gap-2 text-title font-semibold">
              <Sparkles className="size-4 text-muted-foreground" />
              <span>{t($ => $.diagnostics.title, { ns: "results" })}</span>
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
              <span>{t($ => $.guide.title, { ns: "results" })}</span>
            </h3>
            <ol className="list-inside list-decimal space-y-1.5 text-body leading-relaxed text-muted-foreground">
              <li>{t($ => $.guide.step1, { ns: "results" })}</li>
              <li>{t($ => $.guide.step2, { ns: "results" })}</li>
              <li>{t($ => $.guide.step3, { ns: "results" })}</li>
            </ol>
            <Alert className="mt-2 border-warning/40 bg-warning/10 text-body text-warning">
              <AlertTriangle className="size-4 text-warning" />
              <AlertTitle className="text-body font-semibold text-foreground">
                {t($ => $.guideLoadOrderTitle, { ns: "results" })}
              </AlertTitle>
              <AlertDescription className="text-meta text-muted-foreground">
                {t($ => $.guide.importantNote, { ns: "results" })}
              </AlertDescription>
            </Alert>
          </section>
        </div>
      </Panel>
    </div>
  );
}
