import { appIconHtmlImgElement } from "@/components/app/app-icon";
import {
  fullKeyToString,
  stringToFullKey,
  type FullKeyWithDuration,
} from "@/components/target-keys";
import { UsageTooltipContent } from "@/components/viz/usage-tooltip";
import { VizTooltip } from "@/components/viz/viz-tooltip";
import { useRefresh } from "@/hooks/use-refresh";
import type { App, Ref, Tag, WithDuration } from "@/lib/entities";
import { untagged, useAppState, type EntityMap } from "@/lib/state";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import * as echarts from "echarts";
import _ from "lodash";
import { useEffect, useMemo, useRef, useState } from "react";

interface GlobalModel {
  getSeriesByIndex: (index: number) => echarts.SeriesModel;
}

interface RadiusAxis {
  dataToRadius: (data: number) => number;
  polar: PolarAxis;
}

interface PolarAxis {
  getAngleAxis: () => AngleAxis;
}

interface AngleAxis {
  dataToAngle: (data: number) => number;
}
export interface AppUsagePieChartProps {
  data: EntityMap<App, WithDuration<App>>;
  // apps to highlight, order them by index in array. will be drawn left to right = top to bottom. unhighlighted apps will be drawn on top of highlighted apps.
  highlightedApps?: Record<Ref<App>, boolean>;
  highlightedTags?: Record<Ref<Tag>, boolean>;
  unhighlightedAppOpacity?: number;
  hiddenApps?: Record<Ref<App>, boolean>;
  animationsEnabled?: boolean;

  className?: ClassValue;

  onHover?: (key?: FullKeyWithDuration) => void;
}

export function AppUsagePieChart({
  data,
  highlightedApps,
  highlightedTags,
  unhighlightedAppOpacity = 0.3,
  hiddenApps,
  animationsEnabled = true,
  className,
  onHover,
}: AppUsagePieChartProps) {
  const hasAnyHighlighted = useMemo(() => {
    if (!highlightedApps && !highlightedTags) return false;
    return (
      Object.values(highlightedApps ?? {}).some((v) => v) ||
      Object.values(highlightedTags ?? {}).some((v) => v)
    );
  }, [highlightedApps, highlightedTags]);

  const [hoveredData, setHoveredData] = useState<FullKeyWithDuration | null>(
    null,
  );
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  const apps = useAppState((state) => state.apps);
  const tags = useAppState((state) => state.tags);
  const { handleStaleApps } = useRefresh();

  const totalDuration = useMemo(() => {
    return _(apps).reduce(
      (sum, app) => sum + (data[app!.id]?.duration ?? 0),
      0,
    );
  }, [apps, data]);

  const appData = useMemo(() => {
    return (
      _(Object.keys(data))
        // Map to apps
        .map((id) => apps[+id as Ref<App>])
        .thru(handleStaleApps)
        .filter((app) => !hiddenApps?.[app.id])
        .map((app) => ({
          key: "app" as const,
          app,
          duration: data[app.id]?.duration ?? 0,
        }))
        .filter((app) => app.duration > 0)
        .orderBy(["app.tagId", "duration"], ["asc", "desc"]) // "null" is last
        .value()
    );
  }, [apps, data, hiddenApps, handleStaleApps]);

  const tagData = useMemo(() => {
    return _(appData)
      .groupBy("app.tagId")
      .mapValues((apps) => _(apps).sumBy("duration"))
      .toPairs()
      .orderBy(([tagId]) => +tagId) // "null" is last
      .map(([tagId, duration]) => ({
        key: "tag" as const,
        tag: tagId === "null" ? untagged : tags[+tagId as Ref<Tag>]!,
        duration,
      }))
      .filter((tagDur) => tagDur.duration > 0 && tagDur.tag !== undefined) // TODO: use handleStaleTags
      .value();
  }, [tags, appData]);

  useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current, undefined, {});
    chartInstanceRef.current = chart;

    const tagSeries = tagData
      .filter((tag) => tag.duration)
      .map(
        (data) =>
          ({
            type: "bar",
            coordinateSystem: "polar",
            stack: "tags",
            id: fullKeyToString(data),
            data: [
              {
                id: data.tag.id,
                name: data.tag.name,
                value: data.duration,
                itemStyle: {
                  opacity:
                    (highlightedTags?.[data.tag.id] ?? !hasAnyHighlighted)
                      ? undefined
                      : unhighlightedAppOpacity,
                  color: data.tag.color,
                },
              },
            ],
          }) satisfies echarts.SeriesOption,
      );

    const appSeries = appData.map(
      (data) =>
        ({
          type: "bar",
          coordinateSystem: "polar",
          stack: "apps",
          id: fullKeyToString(data),
          labelLayout(params) {
            // eslint-disable-next-line @typescript-eslint/dot-notation
            const model = (chart["getModel"] as () => GlobalModel)();
            const series = model.getSeriesByIndex(params.seriesIndex);
            const radiusAxis = series.getBaseAxis() as unknown as RadiusAxis;
            const value = series.getRawValue(params.dataIndex!);
            const radius = radiusAxis.dataToRadius(value as number);
            const angleAxis = radiusAxis.polar.getAngleAxis();
            const angle =
              angleAxis.dataToAngle(0) - angleAxis.dataToAngle(value as number);

            // const radiusDiff = outerRadius - innerRadius;
            const radiusDiff = 0.35 * radius; // TODO use a better way

            const percent = angle;

            const angleLengthValue = (percent / 360) * 2 * Math.PI * radius;
            const maxSize =
              Math.min(radiusDiff, angleLengthValue) * Math.SQRT1_2;
            const size = Math.max(Math.min(maxSize * 0.9, 32), 0); // 10% padding

            return {
              width: size,
              height: size,
            };
          },
          data: [
            {
              id: data.app.id,
              name: data.app.name,
              value: data.duration,
              itemStyle: {
                opacity:
                  (highlightedApps?.[data.app.id] ?? !hasAnyHighlighted)
                    ? undefined
                    : unhighlightedAppOpacity,
                color: data.app.color,
              },
              label: {
                show: true,
                rotate: 0,
                position: "middle",
                backgroundColor: {
                  image: appIconHtmlImgElement(data.app.icon),
                },
                formatter: () => {
                  return `{empty|}`;
                },
                rich: {
                  empty: {},
                },
              },
            },
          ],
        }) satisfies echarts.SeriesOption,
    );

    const option: echarts.EChartsOption = {
      animation: animationsEnabled,
      // animationDuration: 300,

      polar: [
        {
          radius: ["30%", "50%"],
          center: ["50%", "50%"],
        },
        {
          radius: ["60%", "90%"],
          center: ["50%", "50%"],
        },
      ],
      angleAxis: [
        {
          max: "dataMax",
          show: false,
          polarIndex: 0,
        },
        {
          max: "dataMax",
          show: false,
          polarIndex: 1,
        },
      ],
      radiusAxis: [
        {
          type: "category",
          data: ["tags"],
          show: false,
          polarIndex: 0,
        },
        {
          type: "category",
          data: ["apps"],
          show: false,
          polarIndex: 1,
        },
      ],
      tooltip: {
        trigger: "item",
        formatter() {
          // disables echarts tooltip
          return "";
        },
      },
      series: [
        ...tagSeries.map((series) => ({
          ...series,
          polarIndex: 0,
        })),
        ...appSeries.map((series) => ({
          ...series,
          polarIndex: 1,
        })),
      ],
    } satisfies echarts.EChartsOption;

    chart.getZr().on("mousemove", (params) => {
      const pos = [params.offsetX, params.offsetY];
      type ModelFilter = Parameters<typeof chart.containPixel>[0];
      // SILLY UNDOCUMENTED BEHAVIOR polarIndex: XX just works!
      const isInTag = chart.containPixel(
        { polarIndex: 0 } as unknown as ModelFilter,
        pos,
      );
      const isInApp = chart.containPixel(
        { polarIndex: 1 } as unknown as ModelFilter,
        pos,
      );
      const isInGrid = isInApp || isInTag;
      if (!isInGrid) {
        setHoveredData(null);
        onHover?.(undefined);
      }
    });

    chart.on("mousemove", (params) => {
      const key = stringToFullKey(params.seriesId!, apps, tags);
      const value = { ...key, duration: params.value as number };
      setHoveredData(value);
      onHover?.(value);
    });

    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(() => chart.resize());
    });

    chart.on("finished", () => {
      resizeObserver.observe(chartRef.current!);
    });

    chart.setOption(option);

    return () => {
      chart.dispose();
      resizeObserver.disconnect();
    };
  }, [
    data,
    appData,
    tagData,
    onHover,
    setHoveredData,
    animationsEnabled,
    highlightedApps,
    highlightedTags,
    hasAnyHighlighted,
    unhighlightedAppOpacity,
    apps,
    tags,
  ]);

  return (
    <div ref={chartRef} className={cn("w-full h-full", className)}>
      <VizTooltip targetRef={chartRef} show={!!hoveredData}>
        {hoveredData && (
          <UsageTooltipContent
            data={hoveredData.key === "app" ? appData : tagData}
            hovered={hoveredData}
            maximum={10}
            highlightedApps={highlightedApps}
            highlightedTags={highlightedTags}
            totalDuration={totalDuration}
          />
        )}
      </VizTooltip>
    </div>
  );
}
