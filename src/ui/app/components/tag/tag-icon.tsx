import type { Tag as TagEntity } from "@/lib/entities";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { Tag } from "lucide-react";
import type { ComponentProps } from "react";

export default function TagIcon({
  tag,
  className,
  style,
  ...rest
}: {
  tag?: TagEntity;
  className?: ClassValue;
} & Omit<ComponentProps<"div">, "children" | "className">) {
  return (
    <div
      className={cn(className)}
      style={tag ? { color: tag.color, ...style } : style}
      {...rest}
    >
      <Tag className="w-full h-full" />
    </div>
  );
}
