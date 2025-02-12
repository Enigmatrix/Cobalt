import type { App, Ref, WithGroupedDuration } from "@/lib/entities";
import _ from "lodash";
import { useCallback, useMemo, useState } from "react";
import {
  Bar,
  BarChart,
  CartesianGrid,
  LabelList,
  XAxis,
  YAxis,
} from "recharts";
import {
  type ChartConfig,
  ChartContainer,
  ChartTooltip,
} from "@/components/ui/chart";
import { ticksToDateTime, toHumanDateTime } from "@/lib/time";
import type { ContentType, Props } from "recharts/types/component/Label";
import { toDataUrl } from "@/components/app/app-icon";
import { AppUsageChartTooltipContent } from "@/components/viz/app-usage-chart-tooltip";
import { DateTime } from "luxon";
import { useAppState, type EntityMap } from "@/lib/state";
import { useRefresh } from "@/hooks/use-refresh";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";

export interface AppUsageBarChartProps {
  data: EntityMap<App, WithGroupedDuration<App>[]>;
  singleAppId?: Ref<App>;
  periodTicks: number;
  rangeMinTicks?: number;
  rangeMaxTicks?: number;
  maxYIsPeriod?: boolean;
  hideXAxis?: boolean;
  gridVertical?: boolean;
  gridHorizontal?: boolean;
  gradientBars?: boolean;
  animationsEnabled?: boolean;
  className?: ClassValue;
  dateTimeFormatter?: (dt: DateTime) => string;
  onHover?: (data?: WithGroupedDuration<App>) => void;
}

type AppUsageBarChartData = {
  [app: Ref<App>]: number; // app => duration
  key: number; // group (timestamp)
};

export function AppUsageBarChart({
  data: unflattenedData,
  singleAppId,
  periodTicks,
  rangeMinTicks,
  rangeMaxTicks,
  maxYIsPeriod = false,
  hideXAxis = false,
  gridVertical = false,
  gridHorizontal = false,
  gradientBars = false,
  dateTimeFormatter = toHumanDateTime,
  animationsEnabled = true,
  className,
  onHover,
}: AppUsageBarChartProps) {
  const apps = useAppState((state) => state.apps);
  const { handleStaleApps } = useRefresh();

  const involvedApps = useMemo(
    () =>
      _(
        singleAppId
          ? { [singleAppId]: unflattenedData[singleAppId] }
          : unflattenedData,
      )
        .keys()
        .map((id) => apps[id as unknown as Ref<App>])
        .thru(handleStaleApps)
        .value(),
    [handleStaleApps, apps, unflattenedData, singleAppId],
  );
  // data is grouped by app, we regroup by timestamp.
  const data: AppUsageBarChartData[] = useMemo(() => {
    let ret = _(unflattenedData)
      .values()
      .thru(handleStaleApps)
      .flatten()
      .groupBy((d) => d.group)
      .mapValues((durs) => {
        return _.fromPairs([
          ...durs.map((d) => {
            return [d.id, d.duration];
          }),
          ["key", ticksToDateTime(durs[0].group).toMillis()],
        ]);
      })
      .value();

    if (rangeMinTicks !== undefined && rangeMaxTicks !== undefined) {
      // fill up gaps in the time range.
      ret = _.merge(
        ret,
        _(_.range(rangeMinTicks, rangeMaxTicks, periodTicks))
          .map((t) => {
            return [t, { key: ticksToDateTime(t).toMillis() }];
          })
          .fromPairs()
          .value(),
      );
    }

    return _(ret)
      .values()
      .flatten()
      .sortBy((d) => d.key)
      .value();
  }, [
    unflattenedData,
    handleStaleApps,
    rangeMinTicks,
    rangeMaxTicks,
    periodTicks,
  ]);

  const [hoveredAppId, setHoveredAppId] = useState<Ref<App> | null>(null);

  const config = {} satisfies ChartConfig; // TODO: generate config

  const renderCustomizedLabel: ContentType = useCallback((props: Props) => {
    const {
      x: xValue,
      y: yValue,
      width: widthValue,
      height: heightValue,
      value,
    } = props;
    const app = value as unknown as App;
    const x = Number(xValue);
    const y = Number(yValue);
    const width = Number(widthValue);
    const height = Number(heightValue);

    const radius = Math.min(Math.max(Math.min(width, height) / 2 - 4, 0), 16);

    return (
      <image
        x={x + width / 2 - radius}
        y={y + height / 2 - radius}
        width={radius * 2}
        height={radius * 2}
        href={toDataUrl(app.icon)}
        pointerEvents="none"
      />
    );
  }, []);

  return (
    <ChartContainer config={config} className={cn(className)}>
      <BarChart accessibilityLayer data={data}>
        <defs>
          {gradientBars &&
            involvedApps.map((app) => (
              <linearGradient
                key={app.id}
                id={`gradient-${app.id}`}
                gradientTransform="rotate(90)"
              >
                <stop offset="50%" stopColor={app.color} />
                <stop offset="95%" stopColor={app.color} stopOpacity={0.7} />
              </linearGradient>
            ))}
        </defs>
        <CartesianGrid vertical={gridVertical} horizontal={gridHorizontal} />
        <XAxis
          dataKey="key"
          hide={hideXAxis}
          tickMargin={10}
          tickFormatter={(value) =>
            dateTimeFormatter(DateTime.fromMillis(value))
          }
        />
        <YAxis
          type="number"
          hide
          domain={["dataMin", maxYIsPeriod ? periodTicks : "dataMax"]}
        />
        <ChartTooltip
          allowEscapeViewBox={{ x: true, y: true }}
          wrapperStyle={{ zIndex: 1000 }}
          content={
            <AppUsageChartTooltipContent
              maximumApps={10}
              hoveredAppId={hoveredAppId}
              singleAppId={singleAppId}
              hideLabel
            />
          }
        />
        {involvedApps.map((app) => (
          <Bar
            key={app.id}
            dataKey={app.id}
            stackId="a"
            fill={gradientBars ? `url(#gradient-${app.id})` : app.color}
            name={app.name}
            isAnimationActive={animationsEnabled}
            onMouseEnter={(e) => {
              setHoveredAppId(app.id);
              onHover?.({
                id: app.id,
                duration: e[app.id],
                group: e.key,
              });
            }}
            onMouseLeave={() => {
              setHoveredAppId(null);
              onHover?.(undefined);
            }}
            radius={4}
          >
            {singleAppId === undefined && (
              <LabelList dataKey={() => app} content={renderCustomizedLabel} />
            )}
          </Bar>
        ))}
      </BarChart>
    </ChartContainer>
  );
}
