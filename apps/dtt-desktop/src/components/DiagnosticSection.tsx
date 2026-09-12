import { ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Badge } from "@/components/ui/badge";
import type { GenerationDiagnosticItemDto } from "@/ipc/bindings";
import { cn } from "@/lib/utils";

interface DiagnosticSectionProps {
  categoryKey: string;
  items: GenerationDiagnosticItemDto[];
}

export function DiagnosticSection({
  categoryKey,
  items,
}: DiagnosticSectionProps) {
  const { t } = useTranslation(["results", "empire"]);
  const [isOpen, setIsOpen] = useState(false);
  const [expandedItemIndices, setExpandedItemIndices] = useState<Set<number>>(
    new Set(),
  );

  const count = items.length;
  const isDisabled = count === 0;

  const toggleItemDetail = (index: number) => {
    setExpandedItemIndices((prev) => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  };

  const displayedItems = items.slice(0, 50);
  const truncatedCount = count - 50;

  return (
    <div
      className={cn(
        "rounded-md bg-surface transition-colors",
        isDisabled ? "opacity-50" : "hover:bg-muted",
      )}
    >
      <button
        type="button"
        disabled={isDisabled}
        onClick={() => !isDisabled && setIsOpen(!isOpen)}
        className={cn(
          "flex w-full items-center justify-between px-3.5 py-2.5 text-left text-body transition-colors",
          isDisabled ? "cursor-not-allowed" : "cursor-pointer",
        )}
      >
        <div className="flex items-center gap-2">
          <ChevronRight
            className={cn(
              "size-3.5 text-muted-foreground transition-transform duration-200",
              isOpen && "rotate-90",
            )}
          />
          <span className="font-medium text-foreground">
            {t(`results:diagnostics.categories.${categoryKey}.title`)}
          </span>
          <span className="hidden text-meta text-muted-foreground sm:inline">
            {t(`results:diagnostics.categories.${categoryKey}.description`)}
          </span>
        </div>
        <Badge variant="secondary" className="h-5 px-1.5 text-meta font-normal tabular-nums">
          {count}
        </Badge>
      </button>

      {isOpen && count > 0 && (
        <div className="space-y-2 border-t border-border-subtle px-3.5 py-3">
          {displayedItems.map((item, idx) => {
            const hasDetail = !!item.detail;
            const isDetailOpen = expandedItemIndices.has(idx);

            return (
              <div key={idx} className="rounded-md bg-surface p-2 font-mono text-body">
                <div className="flex items-start justify-between gap-2">
                  <div className="font-medium break-all text-foreground select-text">
                    {item.summary}
                  </div>
                  {hasDetail && (
                    <button
                      type="button"
                      onClick={() => toggleItemDetail(idx)}
                      className="flex shrink-0 items-center gap-0.5 text-meta text-muted-foreground hover:text-foreground"
                    >
                      <span>
                        {isDetailOpen ? t("empire:showLess") : t("empire:detail")}
                      </span>
                      <ChevronDown
                        className={cn(
                          "size-3 transition-transform duration-150",
                          isDetailOpen && "rotate-180",
                        )}
                      />
                    </button>
                  )}
                </div>

                {hasDetail && isDetailOpen && (
                  <pre className="mt-2 overflow-x-auto whitespace-pre-wrap rounded bg-surface p-2 text-meta leading-relaxed text-muted-foreground select-text">
                    {item.detail}
                  </pre>
                )}
              </div>
            );
          })}

          {truncatedCount > 0 && (
            <p className="pt-1 text-center text-meta text-muted-foreground">
              {t("results:diagnostics.truncated", { count: truncatedCount })}
            </p>
          )}
        </div>
      )}
    </div>
  );
}
