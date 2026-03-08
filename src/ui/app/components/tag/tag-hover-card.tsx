import AppIcon from "@/components/app/app-icon";
import { ScoreBadge } from "@/components/tag/score";
import TagIcon from "@/components/tag/tag-icon";
import { DurationText } from "@/components/time/duration-text";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card";
import { Separator } from "@/components/ui/separator";
import { Text } from "@/components/ui/text";
import { useApps } from "@/hooks/use-refresh";
import type { Tag } from "@/lib/entities";
import { ArrowRightIcon } from "lucide-react";
import type { ReactNode } from "react";
import { NavLink } from "react-router";

const MAX_VISIBLE_APPS = 10;

export function TagHoverCard({
  tag,
  children,
}: {
  tag: Tag;
  children: ReactNode;
}) {
  return (
    <HoverCard>
      <HoverCardTrigger asChild>{children}</HoverCardTrigger>
      <HoverCardContent side="bottom" align="start" className="w-80">
        <TagHoverCardContent tag={tag} />
      </HoverCardContent>
    </HoverCard>
  );
}

function TagHoverCardContent({ tag }: { tag: Tag }) {
  const apps = useApps(tag.apps);
  const visibleApps = apps.slice(0, MAX_VISIBLE_APPS);
  const hiddenApps = apps.slice(MAX_VISIBLE_APPS);
  const remainingCount = hiddenApps.length;

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <TagIcon tag={tag} className="w-10 h-10 shrink-0" />
        <div className="flex flex-col min-w-0 gap-1">
          <Text className="font-semibold">{tag.name}</Text>
          <ScoreBadge score={tag.score} className="self-start text-xs" />
        </div>
      </div>

      {apps.length > 0 && (
        <div className="flex flex-col gap-1.5">
          <Text
            className="text-xs text-muted-foreground font-medium"
            children={`${apps.length.toString()} App${apps.length !== 1 ? "s" : ""}`}
          />
          <div className="flex flex-wrap gap-1">
            {visibleApps.map((app) => (
              <NavLink
                key={app.id}
                to={`/apps/${app.id}`}
                className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md border border-border bg-muted/30 hover:bg-muted transition-colors text-xs"
                onClick={(e) => e.stopPropagation()}
              >
                <AppIcon app={app} className="w-3.5 h-3.5" />
                <Text className="max-w-24">{app.name}</Text>
              </NavLink>
            ))}
            {remainingCount > 0 && (
              <span className="inline-flex items-center px-2 py-0.5 rounded-md border border-border text-xs text-muted-foreground">
                +{remainingCount} more
              </span>
            )}
          </div>
        </div>
      )}

      <Separator />

      <div className="grid grid-cols-3 gap-2 text-center">
        <div>
          <div className="text-xs text-muted-foreground">Today</div>
          <DurationText
            ticks={tag.usages.today}
            className="text-sm font-medium"
          />
        </div>
        <div>
          <div className="text-xs text-muted-foreground">Week</div>
          <DurationText
            ticks={tag.usages.week}
            className="text-sm font-medium"
          />
        </div>
        <div>
          <div className="text-xs text-muted-foreground">Month</div>
          <DurationText
            ticks={tag.usages.month}
            className="text-sm font-medium"
          />
        </div>
      </div>

      <Separator />

      <NavLink
        to={`/tags/${tag.id}`}
        className="text-sm text-primary hover:underline inline-flex items-center gap-1 self-start"
        onClick={(e) => e.stopPropagation()}
      >
        View Details
        <ArrowRightIcon className="size-3" />
      </NavLink>
    </div>
  );
}
