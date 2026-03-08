import AppIcon from "@/components/app/app-icon";
import { ScoreCircle } from "@/components/tag/score";
import TagIcon from "@/components/tag/tag-icon";
import { DurationText } from "@/components/time/duration-text";
import { Badge } from "@/components/ui/badge";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card";
import { Separator } from "@/components/ui/separator";
import { Text } from "@/components/ui/text";
import { useTag } from "@/hooks/use-refresh";
import type { App } from "@/lib/entities";
import { ArrowRightIcon } from "lucide-react";
import type { ReactNode } from "react";
import { NavLink } from "react-router";

export function AppHoverCard({
  app,
  children,
}: {
  app: App;
  children: ReactNode;
}) {
  return (
    <HoverCard>
      <HoverCardTrigger asChild>{children}</HoverCardTrigger>
      <HoverCardContent side="bottom" align="start" className="w-80">
        <AppHoverCardContent app={app} />
      </HoverCardContent>
    </HoverCard>
  );
}

function AppHoverCardContent({ app }: { app: App }) {
  const tag = useTag(app.tagId);

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <AppIcon app={app} className="w-10 h-10 shrink-0" />
        <div className="flex flex-col min-w-0 gap-1">
          <Text className="font-semibold">{app.name}</Text>
          {app.identity.tag !== "website" && app.company && (
            <Text className="text-sm text-muted-foreground">{app.company}</Text>
          )}
        </div>
      </div>

      {tag && (
        <NavLink
          to={`/tags/${tag.id}`}
          className="inline-flex self-start"
          onClick={(e) => e.stopPropagation()}
        >
          <Badge
            variant="outline"
            style={{
              borderColor: tag.color,
              color: tag.color,
              backgroundColor: "rgba(255, 255, 255, 0.2)",
            }}
            className="whitespace-nowrap hover:opacity-80 transition-opacity"
          >
            <TagIcon tag={tag} className="size-3 mr-1" />
            <Text className="max-w-32">{tag.name}</Text>
            <ScoreCircle score={tag.score} className="ml-1.5" />
          </Badge>
        </NavLink>
      )}

      {app.description && (
        <Text className="text-sm text-muted-foreground line-clamp-2">
          {app.description}
        </Text>
      )}

      <div className="text-xs inline-flex items-center border border-border rounded-md overflow-hidden bg-muted/30 self-start max-w-full">
        <div className="bg-muted px-2 py-1 border-r border-border font-medium shrink-0">
          {app.identity.tag === "uwp"
            ? "UWP"
            : app.identity.tag === "win32"
              ? "Win32"
              : app.identity.tag === "website"
                ? "Web"
                : "Squirrel"}
        </div>
        <Text className="font-mono px-2 py-1 text-muted-foreground">
          {app.identity.tag === "uwp"
            ? app.identity.aumid
            : app.identity.tag === "win32"
              ? app.identity.path
              : app.identity.tag === "website"
                ? app.identity.baseUrl
                : app.identity.identifier}
        </Text>
      </div>

      <Separator />

      <div className="grid grid-cols-3 gap-2 text-center">
        <div>
          <div className="text-xs text-muted-foreground">Today</div>
          <DurationText
            ticks={app.usages.today}
            className="text-sm font-medium"
          />
        </div>
        <div>
          <div className="text-xs text-muted-foreground">Week</div>
          <DurationText
            ticks={app.usages.week}
            className="text-sm font-medium"
          />
        </div>
        <div>
          <div className="text-xs text-muted-foreground">Month</div>
          <DurationText
            ticks={app.usages.month}
            className="text-sm font-medium"
          />
        </div>
      </div>

      <Separator />

      <NavLink
        to={`/apps/${app.id}`}
        className="text-sm text-primary hover:underline inline-flex items-center gap-1 self-start"
        onClick={(e) => e.stopPropagation()}
      >
        View Details
        <ArrowRightIcon className="size-3" />
      </NavLink>
    </div>
  );
}
