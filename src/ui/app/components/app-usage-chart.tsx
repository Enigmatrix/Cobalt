import type { App, Ref, WithGroupedDuration } from "@/lib/entities";
import _ from "lodash";
import { useMemo, useState } from "react";
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
import { ticksToDateTime, ticksToDuration } from "@/lib/time";
import type { ContentType } from "recharts/types/component/Label";
import { toDataUrl } from "./app-icon";
import { AppUsageChartTooltipContent } from "@/components/app-usage-chart-tooltip";
import { DateTime } from "luxon";
import { useAppState } from "@/lib/state";

export interface AppUsageBarChartProps {
  data: WithGroupedDuration<App>[];
  periodTicks: number;
  singleAppId?: Ref<App>;
  rangeMinTicks?: number;
  rangeMaxTicks?: number;
  onHover: (app: Ref<App>, duration: WithGroupedDuration<App>) => void;
}

type AppUsageBarChartData = {
  [app: Ref<App>]: number; // app => duration
  key: number; // group (timestamp)
};

export function AppUsageBarChart({
  data: unflattenedData,
  periodTicks,
  singleAppId,
  rangeMinTicks: minTicks,
  rangeMaxTicks: maxTicks,
  onHover,
}: AppUsageBarChartProps) {
  const apps = useAppState((state) => state.apps);

  const involvedApps = useMemo(
    () =>
      _(unflattenedData)
        .map((d) => d.id)
        .uniq()
        .map((id) => apps[id])
        .filter((app) => app !== undefined) // stale data
        .value(),
    [unflattenedData]
  );
  const data: AppUsageBarChartData[] = useMemo(() => {
    let ret = _(unflattenedData)
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

    if (minTicks !== undefined && maxTicks !== undefined) {
      // fill up gaps in the time range.
      ret = _.merge(
        ret,
        _(_.range(minTicks, maxTicks, periodTicks))
          .map((t) => {
            return [t, { key: ticksToDateTime(t).toMillis() }];
          })
          .fromPairs()
          .value()
      );
    }

    return _(ret)
      .values()
      .flatten()
      .sortBy((d) => d.key)
      .value();
  }, [unflattenedData, minTicks, maxTicks, periodTicks]);

  const [hoveredAppId, setHoveredAppId] = useState<Ref<App> | null>(null);

  const config = {} satisfies ChartConfig; // TODO: generate config

  const renderCustomizedLabel: ContentType = (props) => {
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
  };

  const period = ticksToDuration(periodTicks).toMillis();

  return (
    <ChartContainer config={config}>
      <BarChart accessibilityLayer data={data}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="key"
          tickLine={false}
          // tickMargin={10}
          axisLine={false}
          tickFormatter={(value) =>
            DateTime.fromMillis(value).toFormat("dd MMM")
          }
        />
        <ChartTooltip
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
            fill={app.color}
            name={app.name}
            onMouseEnter={() => setHoveredAppId(app.id)}
            onMouseLeave={() => setHoveredAppId(null)}
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
