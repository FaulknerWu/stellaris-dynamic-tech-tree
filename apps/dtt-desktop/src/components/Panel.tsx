import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface PanelProps {
  title: string;
  icon?: LucideIcon;
  action?: ReactNode;
  description?: string;
  children: ReactNode;
  className?: string;
  bodyClassName?: string;
}

export function Panel({
  title,
  icon: Icon,
  action,
  description,
  children,
  className,
  bodyClassName,
}: PanelProps) {
  return (
    <section
      className={cn(
        "flex h-full min-h-0 min-w-0 flex-col overflow-hidden rounded-lg border border-border bg-card",
        className,
      )}
    >
      <header className="flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-border-subtle bg-panel-header px-4 py-2.5">
        <div className="min-w-0">
          <h2 className="flex items-center gap-2 text-title font-semibold text-foreground">
            {Icon ? <Icon className="size-4 shrink-0 text-primary" /> : null}
            <span className="break-words">{title}</span>
          </h2>
          {description ? (
            <p className="mt-0.5 truncate text-meta text-muted-foreground">
              {description}
            </p>
          ) : null}
        </div>
        {action ? <div className="flex shrink-0 items-center gap-2">{action}</div> : null}
      </header>
      <div className={cn("min-h-0 flex-1 overflow-y-auto p-3.5", bodyClassName)}>
        {children}
      </div>
    </section>
  );
}
