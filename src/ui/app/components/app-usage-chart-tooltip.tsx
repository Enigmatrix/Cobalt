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
    }
>(({ active, payload, className, hideIndicator = false, formatter }, ref) => {
  const apps = useAppState((state) => state.apps);

  if (!active || !payload?.length) {
    return null;
  }

  payload = _.orderBy(payload, "value", "desc");

  return (
    <div
      ref={ref}
      className={cn(
        "grid min-w-[8rem] items-start gap-1.5 rounded-lg border border-border/50 bg-background px-2.5 py-1.5 text-xs shadow-xl",
        className
      )}
    >
      <div className="grid gap-1.5">
        {payload.map((item, index) => {
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
                      buffer={apps[item.dataKey as Ref<App>].icon}
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
      </div>
    </div>
  );
});
AppUsageChartTooltipContent.displayName = "AppUsageChartTooltipContent";
