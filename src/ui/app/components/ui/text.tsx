import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";

export function Text({
  children,
  className,
}: {
  children: string;
  className?: ClassValue;
}) {
  return (
    <div className={cn("truncate", className)} title={children}>
      {children || <span className="opacity-50 font-mono">Empty</span>}
    </div>
  );
}
