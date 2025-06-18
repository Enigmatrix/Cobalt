import { cn } from "@/lib/utils";
import { CircleHelp } from "lucide-react";
import { useMemo } from "react";
import type { ClassValue } from "clsx";
import { CircleHelp as CircleHelpStatic } from "lucide-static";
import { convertFileSrc } from "@tauri-apps/api/core";
import { iconsDir } from "@/lib/state";

export const DEFAULT_ICON_SVG_URL = "data:image/svg+xml," + CircleHelpStatic;

export function toDataUrl(appId?: number) {
  return convertFileSrc(`${iconsDir}/${appId}`);
}

export default function AppIcon({
  id,
  className,
}: {
  id?: number;
  className?: ClassValue;
}) {
  const icon = useMemo(() => {
    const dataUrl = toDataUrl(id);
    return dataUrl ? `url(${dataUrl})` : undefined;
  }, [id]);

  return icon ? (
    <div
      className={cn("bg-no-repeat bg-center bg-cover", className)}
      style={{ backgroundImage: icon }}
    />
  ) : (
    <CircleHelp className={cn(className)} />
  );
}
