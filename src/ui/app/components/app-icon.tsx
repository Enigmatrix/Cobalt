import { cn } from "@/lib/utils";
import { CircleHelp } from "lucide-react";
import { useMemo } from "react";
import { Buffer } from "buffer";

export default function AppIcon({
  buffer,
  className,
}: {
  buffer?: Buffer;
  className?: string;
}) {
  const icon = useMemo(
    () =>
      buffer
        ? 'url("data:;base64,' + Buffer.from(buffer).toString("base64") + '")'
        : undefined,
    [buffer]
  );

  return icon ? (
    <div
      className={cn("bg-no-repeat bg-center bg-cover", className)}
      style={{ backgroundImage: icon }}
    />
  ) : (
    <CircleHelp className={cn(className)} />
  );
}
