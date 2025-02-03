import React from "react";
import * as RechartsPrimitive from "recharts";
import { cn } from "@/lib/utils";
import { toHumanDuration } from "@/lib/time";
import _ from "lodash";
import type { App, Ref } from "@/lib/entities";
import AppIcon from "./app-icon";
import { useAppState } from "@/lib/state";

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
    ref
  ) => {
    const apps = useAppState((state) => state.apps);

    if (!active || !payload?.length) {
      return null;
    }

    payload = _.orderBy(payload, "value", "desc");
    const totalUsage = _(payload[0]?.payload || {})
      .toPairs()
      .reduce((acc, [key, value]) => (key !== "key" ? acc + value : acc), 0);

    if (singleAppId !== undefined) {
      const singleApp = apps[singleAppId];
      if (!singleApp) {
        return null;
      }
      return (
        <div
          ref={ref}
          className={cn(
            "grid min-w-[8rem] items-start gap-1.5 rounded-lg border border-border/50 bg-background px-2.5 py-1.5 text-xs shadow-xl",
            className
          )}
        >
          <div className="flex items-center gap-2 py-2 mb-1">
            <AppIcon
              buffer={singleApp.icon}
              className="w-6 h-6 shrink-0 ml-2 mr-1"
            />
            <span className="text-muted-foreground truncate max-w-52 text-base">
              {singleApp.name}
            </span>
            <div className="flex-1"></div>
            <div className="flex flex-col items-center">
              <span className=" text-sm">
                {toHumanDuration(payload[0]?.payload[singleAppId] || 0)}
              </span>
            </div>
          </div>
        </div>
      );
    }

    return (
      <div
        ref={ref}
        className={cn(
          "grid min-w-[8rem] items-start gap-1.5 rounded-lg border border-border/50 bg-background px-2.5 py-1.5 text-xs shadow-xl",
          className
        )}
      >
        {hoveredAppId && payload.length > 0 && (
          <div className="flex items-center gap-2 border-b border-secondary-foreground/50 py-2 mb-1">
            <AppIcon
              buffer={apps[hoveredAppId]!.icon} // cannot be stale - we filter stale data out
              className="w-6 h-6 shrink-0 ml-2 mr-1"
            />
            <span className="text-muted-foreground truncate max-w-52 text-base">
              {apps[hoveredAppId]!.name}{" "}
              {/* cannot be stale - we filter stale data out */}
            </span>
            <div className="flex-1"></div>
            <div className="flex flex-col items-center">
              <span className="font-semibold text-sm">
                {toHumanDuration(payload[0]?.payload[hoveredAppId] || 0)}
              </span>
              <span className="text-xs">/ {toHumanDuration(totalUsage)}</span>
            </div>
          </div>
        )}
        {!hoveredAppId && totalUsage && (
          <div className="border-b border-secondary-foreground/50 py-2 mb-1 flex justify-end">
            <span className="font-semibold">
              Total: {toHumanDuration(totalUsage)}
            </span>
          </div>
        )}
        <div className="grid gap-1.5">
          {payload.slice(0, maximumApps).map((item, index) => {
            return (
              <div
                key={item.dataKey}
                className={cn(
                  "flex w-full flex-wrap items-stretch gap-2 [&>svg]:h-2.5 [&>svg]:w-2.5 [&>svg]:text-muted-foreground"
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
                        <span className="font-mono font-medium tabular-nums text-foreground ml-2">
                          {toHumanDuration(item.value as number)}
                        </span>
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
  }
);
AppUsageChartTooltipContent.displayName = "AppUsageChartTooltipContent";
