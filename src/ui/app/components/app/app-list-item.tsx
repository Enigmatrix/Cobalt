import { Badge } from "@/components/ui/badge";
import { Text } from "@/components/ui/text";
import AppIcon from "@/components/app/app-icon";
import type { App } from "@/lib/entities";
import { XIcon } from "lucide-react";

export function AppBadge({ app, remove }: { app: App; remove?: () => void }) {
  return (
    <Badge
      className="whitespace-nowrap min-w-0 font-normal border-border border rounded-md h-8 text-foreground"
      style={{
        backgroundColor: "rgba(255, 255, 255, 0.1)",
      }}
    >
      <AppIcon buffer={app.icon} className="h-5 w-5 mr-2" />
      <Text className="text-base">{app.name}</Text>
      {remove && (
        <XIcon
          className="ml-2 h-4 w-4 shrink-0"
          onClick={(event) => {
            event.stopPropagation();
            remove();
          }}
        />
      )}
    </Badge>
  );
}
