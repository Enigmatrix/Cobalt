import { cn } from "@/lib/utils";
import { CircleHelp } from "lucide-react";
import { useMemo, useState } from "react";
import type { ClassValue } from "clsx";
import { CircleHelp as CircleHelpStatic } from "lucide-static";
import { convertFileSrc } from "@tauri-apps/api/core";
import { iconsDir } from "@/lib/state";

export const DEFAULT_ICON_SVG_URL = "data:image/svg+xml," + CircleHelpStatic;

export function toDataUrl(appId: number) {
  return convertFileSrc(`${iconsDir}/${appId}`);
}

export function htmlImgElement(appId: number): HTMLImageElement {
  const img = new Image();
  img.src = toDataUrl(appId);
  img.onerror = () => {
    img.src = DEFAULT_ICON_SVG_URL;
    img.onerror = null;
  };
  return img;
}

export default function AppIcon({
  id,
  className,
}: {
  id: number;
  className?: ClassValue;
}) {
  const [hasError, setHasError] = useState(false);
  const icon = useMemo(() => {
    return toDataUrl(id);
  }, [id]);

  if (!icon || hasError) {
    return <CircleHelp className={cn(className)} />;
  }

  return (
    <img
      className={cn(className)}
      src={icon}
      onError={() => setHasError(true)}
    />
  );
}
