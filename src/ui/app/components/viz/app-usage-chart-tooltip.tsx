import React, { useMemo } from "react";
import { cn } from "@/lib/utils";
import _ from "lodash";
import type { App, Ref } from "@/lib/entities";
import AppIcon from "@/components/app/app-icon";
import { type EntityMap } from "@/lib/state";
import { DateTime } from "luxon";
import { DurationText } from "@/components/time/duration-text";
import { DateTimeText } from "@/components/time/time-text";
import { useApps } from "@/hooks/use-refresh";
import { Text } from "@/components/ui/text";

function HoverCard({
  app,
  usageTicks,
  totalUsageTicks,
  at,
}: {
  app: App;
  usageTicks: number;
  totalUsageTicks?: number;
  at: DateTime;
}) {
  return (
    <div className="flex items-center gap-2 py-2">
      <AppIcon buffer={app.icon} className="w-6 h-6 shrink-0 ml-2 mr-1" />
      <div className="flex flex-col">
        <Text className="text-base max-w-52">{app.name}</Text>
        <DateTimeText className="text-xs text-muted-foreground" datetime={at} />
      </div>

      <div className="flex-1"></div>
      <div className="flex flex-col items-end text-muted-foreground shrink-0 min-w-max">
        <DurationText className="font-semibold text-sm" ticks={usageTicks} />
        {totalUsageTicks && (
          <span className="inline-flex items-center gap-1 text-xs">
            <p>/</p>
            <DurationText ticks={totalUsageTicks} />
          </span>
        )}
      </div>
    </div>
  );
}

export const AppUsageChartTooltipContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div"> & {
    payload: EntityMap<App, number>;
    dt: DateTime;
    hideIndicator?: boolean;
    maximumApps?: number;
    hoveredAppId: Ref<App> | null;
    singleAppId?: Ref<App>;
    highlightedAppIds?: Ref<App>[];
  }
>(
  (
    {
      payload,
      dt,
      className,
      hideIndicator = false,
      hoveredAppId,
      maximumApps,
      singleAppId,
      highlightedAppIds,
    },
    ref,
  ) => {
    const involvedAppIds = useMemo(
      () => Object.keys(payload).map((id) => +id as Ref<App>),
      [payload],
    );
    const totalUsageTicks = useMemo(
      () => (singleAppId ? undefined : _(payload).values().sum()),
      [singleAppId, payload],
    );
    const involvedApps = useApps(involvedAppIds);
    const involvedAppSorted = useMemo(
      () =>
        _(involvedApps)
          .map((app) => ({ app, usageTicks: payload[app.id]! }))
          .filter((v) => v.usageTicks > 0)
          .orderBy(["usageTicks"], ["desc"])
          .orderBy((v) => highlightedAppIds?.indexOf(v.app.id) ?? -1, "desc")
          .value(),
      [involvedApps, payload, highlightedAppIds],
    );

    const highlightedAppId = singleAppId || hoveredAppId;
    const highlightedApp = involvedApps.find(
      (app) => app.id === highlightedAppId,
    );
    const highlightedAppUsageTicks = highlightedAppId
      ? (payload[highlightedAppId] ?? 0)
      : 0;

    return (
      <div
        className={cn("grid max-w-80 items-start gap-1.5 text-xs", className)}
        ref={ref}
      >
        {highlightedApp ? (
          <HoverCard
            app={highlightedApp}
            usageTicks={highlightedAppUsageTicks}
            totalUsageTicks={totalUsageTicks}
            at={dt}
          />
        ) : (
          dt && (
            <div className="flex text-muted-foreground">
              <DateTimeText
                className="text-xs text-muted-foreground"
                datetime={dt}
              />
              <div className="flex-1 min-w-4"></div>
              <DurationText ticks={totalUsageTicks ?? 0} />
            </div>
          )
        )}
        {!singleAppId && Object.keys(payload).length > 0 && (
          <div className="grid gap-1.5 pt-1 border-t border-border">
            {involvedAppSorted
              .slice(0, maximumApps)
              .map(({ app, usageTicks }) => {
                return (
                  <div
                    key={app.id}
                    className={cn(
                      "flex w-full flex-wrap items-stretch gap-2 [&>svg]:h-2.5 [&>svg]:w-2.5 [&>svg]:text-muted-foreground",
                    )}
                  >
                    <>
                      {!hideIndicator && (
                        <AppIcon
                          buffer={app.icon}
                          className="w-4 h-4 shrink-0"
                        />
                      )}
                      <div className="flex flex-1 justify-between items-center">
                        <div className="grid gap-1.5">
                          <Text
                            className={cn({
                              "text-muted-foreground":
                                !highlightedAppIds?.includes(app.id),
                            })}
                          >
                            {app.name}
                          </Text>
                        </div>
                        <DurationText
                          className="font-mono font-medium tabular-nums text-foreground ml-2 shrink-0"
                          ticks={usageTicks}
                        />
                      </div>
                    </>
                  </div>
                );
              })}
            {maximumApps !== undefined &&
              involvedAppSorted.length > maximumApps && (
                <span className="text-muted-foreground">
                  + {involvedAppSorted.length - maximumApps} more
                </span>
              )}
          </div>
        )}
      </div>
    );
  },
);
AppUsageChartTooltipContent.displayName = "AppUsageChartTooltipContent";
