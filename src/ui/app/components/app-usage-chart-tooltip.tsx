import React from "react";
import * as RechartsPrimitive from "recharts";
import { cn } from "@/lib/utils";
import { toHumanDateTime } from "@/lib/time";
import _ from "lodash";
import type { App, Ref } from "@/lib/entities";
import AppIcon from "@/components/app-icon";
import { useAppState } from "@/lib/state";
import { DateTime } from "luxon";
import { DurationText } from "@/components/duration-text";

export const AppUsageChartTooltipContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<typeof RechartsPrimitive.Tooltip> &
    React.ComponentProps<"div"> & {
      hideLabel?: boolean;
      hideIndicator?: boolean;
      nameKey?: string;
      labelKey?: string;
      maximumApps?: number;
      hoveredAppId: Ref<App> | null;
      singleAppId?: Ref<App>;
    }
>(
  (
    {
      active,
      payload,
      className,
      hideIndicator = false,
      formatter,
      hoveredAppId,
      maximumApps,
      singleAppId,
    },
    ref,
  ) => {
    const apps = useAppState((state) => state.apps);

    if (!active || !payload?.length) {
      return null;
    }

    payload = _.orderBy(payload, "value", "desc");
    // this total usage might not be accurate if there are stale apps!
    const totalUsage = _(payload[0]?.payload || {})
      .toPairs()
      .reduce((acc, [key, value]) => (key !== "key" ? acc + value : acc), 0);

    const dtMills = payload[0]?.payload.key as number | undefined;
    const dt = dtMills ? DateTime.fromMillis(dtMills) : undefined;

    if (singleAppId !== undefined) {
      const singleApp = apps[singleAppId];
      const singlePayload = payload[0];
      if (!singleApp || !singlePayload) {
        return null;
      }
      return (
        <div
          ref={ref}
          className={cn(
            "grid min-w-[8rem] items-start gap-1.5 rounded-lg border border-border bg-background px-2.5 py-1.5 text-xs shadow-xl",
            className,
          )}
        >
          <div className="flex items-center gap-2 py-2">
            <AppIcon
              buffer={singleApp.icon}
              className="w-6 h-6 shrink-0 ml-2 mr-1"
            />
            <div className="flex flex-col">
              <span className="truncate max-w-52 text-base">
                {singleApp.name}
              </span>
              {dt && (
                <div className="text-xs text-muted-foreground tracking-tighter">
                  {toHumanDateTime(dt)}
                </div>
              )}
            </div>
            <div className="flex-1"></div>
            <div className="flex flex-col items-center shrink-0 min-w-max">
              <DurationText
                className="text-muted-foreground text-sm"
                ticks={singlePayload?.payload[singleAppId] || 0}
              />
            </div>
          </div>
        </div>
      );
    }

    return (
      <div
        ref={ref}
        className={cn(
          "grid min-w-[8rem] items-start gap-1.5 rounded-lg border border-border bg-background px-2.5 py-1.5 text-xs shadow-xl",
          className,
        )}
      >
        {hoveredAppId && payload.length > 0 && (
          <div className="flex items-center gap-2 border-b border-border py-2 mb-1">
            <AppIcon
              buffer={apps[hoveredAppId]!.icon} // cannot be stale - we filter stale data out
              className="w-6 h-6 shrink-0 ml-2 mr-1"
            />
            <div className="flex flex-col">
              <span className="truncate max-w-52 text-base">
                {apps[hoveredAppId]!.name}
                {/* cannot be stale - we filter stale data out */}
              </span>
              {dt && (
                <div className="text-xs text-muted-foreground tracking-tighter">
                  {toHumanDateTime(dt)}
                </div>
              )}
            </div>
            <div className="flex-1"></div>
            <div className="flex flex-col items-end text-muted-foreground shrink-0 min-w-max">
              <DurationText
                className="font-semibold text-sm"
                ticks={payload[0]?.payload[hoveredAppId] || 0}
              />
              <span className="inline-flex items-center gap-1 text-xs">
                <p>/</p>
                <DurationText ticks={totalUsage} />
              </span>
            </div>
          </div>
        )}
        {!hoveredAppId && totalUsage && (
          <div className="flex-col border-b border-border py-2 mb-1 flex items-end">
            <span className="inline-flex items-center gap-1 font-semibold">
              <p>Total:</p>
              <DurationText ticks={totalUsage} />
            </span>

            {dt && (
              <div className="text-xs text-muted-foreground">
                {toHumanDateTime(dt)}
              </div>
            )}
          </div>
        )}
        <div className="grid gap-1.5">
          {payload.slice(0, maximumApps).map((item, index) => {
            return (
              <div
                key={item.dataKey}
                className={cn(
                  "flex w-full flex-wrap items-stretch gap-2 [&>svg]:h-2.5 [&>svg]:w-2.5 [&>svg]:text-muted-foreground",
                )}
              >
                {formatter && item?.value !== undefined && item.name ? (
                  formatter(item.value, item.name, item, index, item.payload)
                ) : (
                  <>
                    {!hideIndicator && (
                      <AppIcon
                        buffer={apps[item.dataKey as Ref<App>]!.icon} // cannot be stale - we filter stale data out
                        className="w-4 h-4 shrink-0"
                      />
                    )}
                    <div className="flex flex-1 justify-between items-center">
                      <div className="grid gap-1.5">
                        <span className="text-muted-foreground truncate max-w-52">
                          {item.name}
                        </span>
                      </div>
                      {item.value && (
                        <DurationText
                          className="font-mono font-medium tabular-nums text-foreground ml-2"
                          ticks={item.value as number}
                        />
                      )}
                    </div>
                  </>
                )}
              </div>
            );
          })}
          {maximumApps !== undefined && payload.length > maximumApps && (
            <span className="text-muted-foreground">
              + {payload.length - maximumApps} more
            </span>
          )}
        </div>
      </div>
    );
  },
);
AppUsageChartTooltipContent.displayName = "AppUsageChartTooltipContent";
