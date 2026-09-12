import React from "react";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

interface IdBadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string;
  variant?: "default" | "secondary" | "outline" | "destructive";
}

export function IdBadge({
  value,
  variant = "secondary",
  className,
  ...props
}: IdBadgeProps) {
  return (
    <Badge
      variant={variant}
      className={cn(
        "inline-flex items-center gap-1 font-mono text-meta font-normal leading-none tracking-tight",
        className,
      )}
      {...props}
    >
      <span>{value}</span>
    </Badge>
  );
}
