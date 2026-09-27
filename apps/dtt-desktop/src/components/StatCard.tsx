import { useLocale } from "@/i18n/controller";
import { numberFormatter } from "@/i18n/format";
import React from "react";

import { cn } from "@/lib/utils";

interface StatCardProps {
  label: string;
  value: React.ReactNode;
  description?: string;
  icon?: React.ReactNode;
  className?: string;
}

export function StatCard({
  label,
  value,
  description,
  icon,
  className,
}: StatCardProps) {
  const { resolvedLocale } = useLocale();
  return (
    <div className={cn("rounded-lg bg-surface p-4", className)}>
      <div className="flex items-center justify-between">
        <span className="text-body font-medium text-muted-foreground">{label}</span>
        {icon && <div className="text-muted-foreground">{icon}</div>}
      </div>
      <div className="mt-2 text-2xl font-semibold tracking-tight tabular-nums text-foreground">
        {typeof value === "number" ? numberFormatter(resolvedLocale).format(value) : value}
      </div>
      {description && (
        <p className="mt-1 text-meta text-muted-foreground">{description}</p>
      )}
    </div>
  );
}
