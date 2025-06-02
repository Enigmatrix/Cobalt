import type { App, Tag, Ref, WithDuration } from "@/lib/entities";
import { untagged, type EntityMap } from "@/lib/state";
import type { ClassValue } from "clsx";
import { useTheme } from "@/components/theme-provider";
import { useApps, useTags } from "@/hooks/use-refresh";
import { useEffect, useMemo, useRef, useState } from "react";
import * as echarts from "echarts";
import _ from "lodash";
import { Tooltip } from "@/components/viz/tooltip";
import { AppUsageChartTooltipContent } from "./app-usage-chart-tooltip";
import { cn } from "@/lib/utils";
import { DEFAULT_ICON_SVG_URL } from "../app/app-icon";
import { toDataUrl } from "../app/app-icon";

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
  highlightedAppIds?: Ref<App>[];
  unhighlightedAppOpacity?: number;
  hideApps?: Record<Ref<App>, boolean>;
  animationsEnabled?: boolean;

  className?: ClassValue;
  onHover?: (data?: WithDuration<App>) => void;
  onTagHover?: (data?: Tag) => void;
}

export function AppUsagePieChart({
  data,
  highlightedAppIds,
  unhighlightedAppOpacity = 0.3,
  hideApps,
  animationsEnabled = true,
  className,
  onHover,
}: AppUsagePieChartProps) {
  const { theme } = useTheme();

  const [hoveredData, setHoveredData] = useState<{
    tagId?: Ref<Tag>;
    appId?: Ref<App>;
  } | null>(null);
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  const appIds = useMemo(
    () => Object.keys(data).map((id) => +id as Ref<App>),
    [data],
  );
  const apps = useApps(appIds);
  const tags = useTags();

  const totalUsage = useMemo(() => {
    return _(apps)
      .filter((app) => !hideApps?.[app.id])
      .reduce((sum, app) => sum + (data[app.id]?.duration ?? 0), 0);
  }, [apps, data, hideApps]);

  const appPayload = useMemo(() => {
    return _.mapValues(data, (duration) => duration?.duration ?? 0);
  }, [data]);

  const appData = useMemo(() => {
    return _(apps)
      .filter((app) => !hideApps?.[app.id])
      .map((app) => ({
        ...app,
        duration: data[app.id]?.duration ?? 0,
      }))
      .filter((app) => app.duration > 0)
      .orderBy(["tagId", "duration"], ["asc", "desc"])
      .value();
  }, [apps, data, hideApps]);

  const tagData = useMemo(() => {
    // Get all tagged apps
    const taggedAppIds = new Set(tags.flatMap((tag) => tag.apps));

    // Calculate untagged duration
    const untaggedDuration = apps
      .filter((app) => !taggedAppIds.has(app.id) && !hideApps?.[app.id])
      .reduce((sum, app) => sum + (data[app.id]?.duration ?? 0), 0);

    // Create tag data including untagged
    const tagData = [
      // Tags
      ...tags.map((tag) => ({
        ...tag,
        duration: _(tag.apps)
          .map((appId) => data[appId]?.duration ?? 0)
          .sum(),
      })),
      // Add untagged category if there are untagged apps
      ...(untaggedDuration > 0
        ? [
            {
              ...untagged,
              duration: untaggedDuration,
            },
          ]
        : []),
    ].filter((tag) => tag.duration > 0);

    return tagData;
  }, [tags, apps, data, hideApps]);

  useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current, undefined, {});
    chartInstanceRef.current = chart;

    const tagSeries = tagData
      .filter((tag) => tag.duration)
      .map(
        (tag) =>
          ({
            type: "bar",
            coordinateSystem: "polar",
            stack: "tags",
            id: "tag-" + tag.id,
            data: [
              {
                id: tag.id,
                name: tag.name,
                value: tag.duration,
                itemStyle: {
                  color: tag.color,
                },
              },
            ],
          }) satisfies echarts.SeriesOption,
      );

    const appSeries = appData.map(
      (app) =>
        ({
          type: "bar",
          coordinateSystem: "polar",
          stack: "apps",
          id: "app-" + app.id,
          labelLayout(params) {
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
              id: app.id,
              name: app.name,
              value: app.duration,
              itemStyle: {
                color: app.color,
              },
              label: {
                show: true,
                rotate: 0,
                position: "middle",
                backgroundColor: {
                  image: toDataUrl(app.icon) ?? DEFAULT_ICON_SVG_URL,
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
      if (params.seriesId?.startsWith("app-")) {
        const id = +params.seriesId.slice(4) as Ref<App>;
        setHoveredData({ appId: id });
      } else if (params.seriesId?.startsWith("tag-")) {
        const idStr = params.seriesId.slice(4);
        let id: Ref<Tag> = untagged.id;
        if (idStr !== untagged.id + "") {
          id = +idStr as Ref<Tag>;
        }
        setHoveredData({ tagId: id });
      }
      // else ignore event
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
    animationsEnabled,
    highlightedAppIds,
    unhighlightedAppOpacity,
    theme,
  ]);

  return (
    <div ref={chartRef} className={cn("w-full h-full asdf", className)}>
      <Tooltip targetRef={chartRef} show={!!hoveredData?.appId}>
        <AppUsageChartTooltipContent
          hoveredAppId={hoveredData?.appId ?? null}
          payload={appPayload}
          maximumApps={10}
          highlightedAppIds={highlightedAppIds}
        />
      </Tooltip>
    </div>
  );
}
