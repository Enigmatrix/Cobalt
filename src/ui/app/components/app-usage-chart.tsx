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
import { ticksToDateTime } from "@/lib/time";
import type { ContentType } from "recharts/types/component/Label";
import { toDataUrl } from "./app-icon";
import { AppUsageChartTooltipContent } from "@/components/app-usage-chart-tooltip";
import { DateTime } from "luxon";
import { useAppState, type EntityMap } from "@/lib/state";

export interface AppUsageBarChartProps {
  data: EntityMap<App, WithGroupedDuration<App>[]>;
  singleAppId?: Ref<App>;
  periodTicks: number;
  rangeMinTicks?: number;
  rangeMaxTicks?: number;
  maxYIsPeriod?: boolean;
  onHover: (app: Ref<App>, duration: WithGroupedDuration<App>) => void;
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
  onHover,
}: AppUsageBarChartProps) {
  const apps = useAppState((state) => state.apps);

  const involvedApps = useMemo(
    () =>
      Object.keys(unflattenedData)
        .map((id) => apps[id as unknown as Ref<App>])
        .filter((app) => app !== undefined), // stale data
    [unflattenedData]
  );
  // data is grouped by app, we regroup by timestamp.
  const data: AppUsageBarChartData[] = useMemo(() => {
    let ret = _(unflattenedData)
      .values()
      .filter((x) => x !== undefined) // stale data
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
          .value()
      );
    }

    return _(ret)
      .values()
      .flatten()
      .sortBy((d) => d.key)
      .value();
  }, [unflattenedData, rangeMinTicks, rangeMaxTicks, periodTicks]);

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
        <YAxis
          type="number"
          hide
          domain={["dataMin", maxYIsPeriod ? periodTicks : "dataMax"]}
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
