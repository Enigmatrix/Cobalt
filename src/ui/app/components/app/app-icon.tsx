import { cn } from "@/lib/utils";
import { CircleHelp } from "lucide-react";
import { useMemo, useState } from "react";
import type { ClassValue } from "clsx";
import { CircleHelp as CircleHelpStatic } from "lucide-static";
import { convertFileSrc } from "@tauri-apps/api/core";
import { iconsDir } from "@/lib/state";

export const DEFAULT_ICON_SVG_URL = "data:image/svg+xml," + CircleHelpStatic;

function appIconUrl(appIcon: string) {
  return convertFileSrc(`${iconsDir}/${appIcon}`);
}

export function htmlImgElement(appIcon: string): HTMLImageElement {
  const img = new Image();
  img.src = appIconUrl(appIcon);
  img.onerror = () => {
    img.src = DEFAULT_ICON_SVG_URL;
    img.onerror = null;
  };
  return img;
}

export default function AppIcon({
  appIcon,
  className,
}: {
  appIcon: string;
  className?: ClassValue;
}) {
  const [hasError, setHasError] = useState(false);
  const icon = useMemo(() => {
    return appIconUrl(appIcon);
  }, [appIcon]);

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
