import { AppHoverCard } from "@/components/app/app-hover-card";
import AppIcon from "@/components/app/app-icon";
import { Badge } from "@/components/ui/badge";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card";
import { Text } from "@/components/ui/text";
import type { App } from "@/lib/entities";
import type { CSSProperties, ReactNode } from "react";
import { FixedSizeList as List } from "react-window";

// hardcoding these makes things a lot easier

const ROW_HEIGHT = 32;
const MAX_VISIBLE_ROWS = 8;

export function AppOverflowTooltip({
  apps,
  children,
}: {
  apps: App[];
  children?: ReactNode;
}) {
  const listHeight = Math.min(apps.length, MAX_VISIBLE_ROWS) * ROW_HEIGHT;
  const hasScrollbar = apps.length > MAX_VISIBLE_ROWS;

  return (
    <HoverCard>
      <HoverCardTrigger asChild>
        {children ?? (
          <Badge
            variant="outline"
            style={{ backgroundColor: "rgba(255, 255, 255, 0.2)" }}
            className="whitespace-nowrap ml-1 text-card-foreground/60 rounded-md h-6 cursor-default"
          >{`+${apps.length}`}</Badge>
        )}
      </HoverCardTrigger>
      <HoverCardContent side="bottom" align="start" className="p-1.5 w-52">
        <List
          height={listHeight}
          itemCount={apps.length}
          itemSize={ROW_HEIGHT}
          width="100%"
          itemData={apps}
        >
          {({ index, style }: { index: number; style: CSSProperties }) => {
            const app = apps[index];
            return (
              <div
                style={style}
                className={hasScrollbar ? "px-0.5 pr-1" : "px-0.5"}
              >
                <AppHoverCard app={app}>
                  <div className="flex items-center gap-2 px-2 h-7 rounded-md bg-muted/40 hover:bg-muted transition-colors cursor-default">
                    <AppIcon app={app} className="w-4 h-4 shrink-0" />
                    <Text className="text-xs">{app.name}</Text>
                  </div>
                </AppHoverCard>
              </div>
            );
          }}
        </List>
      </HoverCardContent>
    </HoverCard>
  );
}
