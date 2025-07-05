import { iconsDir } from "@/lib/state";
import { cn } from "@/lib/utils";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ClassValue } from "clsx";
import { CircleHelp } from "lucide-react";
import {
  CircleHelp as CircleHelpStatic,
  Tag as TagStatic,
} from "lucide-static";
import normalize from "path-normalize";
import { useMemo, useState } from "react";

export const DEFAULT_ICON_SVG_URL = "data:image/svg+xml," + CircleHelpStatic;
export const TAG_ICON_URL = "data:image/svg+xml," + TagStatic;

function appIconUrl(appIcon?: string) {
  if (!appIcon) return null;
  const fileName = normalize(appIcon);
  return convertFileSrc(`${iconsDir}/${fileName}`);
}

export function htmlImgElement(appIcon?: string): HTMLImageElement {
  const img = new Image();
  img.src = appIconUrl(appIcon) ?? DEFAULT_ICON_SVG_URL;
  img.onerror = () => {
    img.src = DEFAULT_ICON_SVG_URL;
    img.onerror = null;
  };
  return img;
}

// A little hack to make the tag icon color dynamic - relies on Lucide Static using currentColor for the color
export function tagIconUrlStatic(color?: string) {
  return TAG_ICON_URL.replaceAll(
    "currentColor",
    color ? encodeURIComponent(color) : "currentColor",
  );
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
