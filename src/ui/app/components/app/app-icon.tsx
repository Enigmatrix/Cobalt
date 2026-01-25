import type { App } from "@/lib/entities";
import { cn } from "@/lib/utils";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ClassValue } from "clsx";
import { CircleHelp } from "lucide-react";
import {
  CircleHelp as CircleHelpStatic,
  Tag as TagStatic,
} from "lucide-static";
import { useEffect, useState } from "react";

export const DEFAULT_ICON_SVG_URL = "data:image/svg+xml," + CircleHelpStatic;
export const TAG_ICON_URL = "data:image/svg+xml," + TagStatic;

export function appIconUrl(app?: App) {
  if (!app?.id) return DEFAULT_ICON_SVG_URL;
  return convertFileSrc(app.id.toString(), "appicon");
}

export function appIconHtmlImgElement(app?: App): HTMLImageElement {
  const img = new Image();
  img.src = appIconUrl(app);
  img.onerror = () => {
    img.src = DEFAULT_ICON_SVG_URL;
    img.onerror = null;
  };
  return img;
}

// A little hack to make the tag icon color dynamic - relies on Lucide Static using currentColor for the color
export function tagIconUrl(color?: string) {
  return TAG_ICON_URL.replaceAll(
    "currentColor",
    color ? encodeURIComponent(color) : "currentColor",
  );
}

export default function AppIcon({
  app,
  className,
}: {
  app?: App;
  className?: ClassValue;
}) {
  const [hasError, setHasError] = useState(false);
  const [icon, setIcon] = useState<string | null>(() => appIconUrl(app));
  // We assume that this useEffect runs before the setHasError(true) call
  // i.e. this is not possible:
  // 1. initial mount <--- img is shown
  // 2. img load fails, and setHasError(true) is called <--- img error icon is flashed, then CircleHelp is shown
  // 3. useEffect runs, and setHasError(false) is called <--- img is shown, img error icon is flashed
  //
  // 4. img load fails (again), and setHasError(true) is called <--- CircleHelp is shown
  useEffect(() => {
    setHasError(false);
    setIcon(appIconUrl(app));
  }, [app]);

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
