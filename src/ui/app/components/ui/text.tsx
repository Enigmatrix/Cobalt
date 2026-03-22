import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import type { ComponentProps } from "react";

export function Text({
  children,
  className,
  ...rest
}: {
  children: string;
  className?: ClassValue;
} & Omit<ComponentProps<"div">, "children" | "className" | "title">) {
  return (
    <div className={cn("truncate", className)} title={children} {...rest}>
      {children || <span className="opacity-50 font-mono">Empty</span>}
    </div>
  );
}
