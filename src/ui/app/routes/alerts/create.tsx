import { useZodForm } from "@/hooks/use-form";
import { alertSchema } from "@/lib/schema";
import { FormItem } from "@/components/ui/form";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import {
  useCallback,
  useMemo,
  useState,
  useRef,
  useEffect,
  Fragment,
} from "react";
import { useAppState } from "@/lib/state";
import { useNavigate } from "react-router";
import { Duration } from "luxon";
import { useIntervalControlsWithDefault } from "@/hooks/use-time";
import { timeFrameToPeriod, type Target, type TimeFrame } from "@/lib/entities";
import { DateTime } from "luxon";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { Tabs, TabsContent } from "@/components/ui/tabs";
import { TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Label } from "@/components/ui/label";
import { DateRangePicker } from "@/components/time/date-range-picker";
import { useTargetApps } from "@/hooks/use-refresh";
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  InfoIcon,
  PlusIcon,
} from "lucide-react";
import { useFieldArray } from "react-hook-form";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import type { CreateAlert } from "@/lib/repo";
import { DurationText } from "@/components/time/duration-text";
import { useTriggerInfo } from "@/hooks/use-trigger-info";
import { AlertForm, type FormValues } from "@/components/alert/alert-form";
import type { Period } from "@/lib/time";

export default function CreateAlerts() {
  const form = useZodForm({
    schema: alertSchema,
    defaultValues: {
      ignoreTrigger: false,
      reminders: [],
    },
  });
  const createAlert = useAppState((state) => state.createAlert);
  const navigate = useNavigate();
  const target = form.watch("target");
  const usageLimit = form.watch("usageLimit");
  const timeFrame = form.watch("timeFrame");
  const reminders = form.watch("reminders");
  const { fields, append, remove, update } = useFieldArray({
    control: form.control,
    name: "reminders",
  });

  const triggerInfo = useTriggerInfo(target, usageLimit, timeFrame, reminders);

  const onSubmit = useCallback(
    async (values: FormValues) => {
      // FormValues is not the same as CreateAlert
      // the ignoreTrigger in FormValues means ignore all firing alerts and reminders
      // but in CreateAlert, it's customizable for each alert and reminder.

      const object: CreateAlert = {
        ...structuredClone(values),
        ignoreTrigger: false,
        reminders: values.reminders.map((reminder) => ({
          ...reminder,
          ignoreTrigger: false,
        })),
      };

      if (values.ignoreTrigger) {
        object.ignoreTrigger = triggerInfo.alert;
        object.reminders.forEach((reminder, index) => {
          reminder.ignoreTrigger = triggerInfo.reminders[index].trigger;
        });
      }

      await createAlert(object);
      await navigate("/alerts");
    },
    [createAlert, navigate, triggerInfo],
  );

  const handleReminderUpdate = useCallback(
    (index: number, threshold: number) => {
      update(index, { threshold, message: reminders[index].message });
    },
    [update, reminders],
  );

  return (
    <>
      <main className="grid grid-cols-[360px_minmax(0,1fr)] h-full ">
        <div className="max-h-screen overflow-y-auto my-auto">
          <AlertForm
            onSubmit={onSubmit}
            form={form}
            triggerInfo={triggerInfo}
            remindersFields={fields}
            remindersAppend={append}
            remindersRemove={remove}
          />
        </div>
        <div className="flex flex-col p-8">
          <Tabs defaultValue="usage" className="flex-1 flex flex-col">
            <TabsList className="self-center">
              <TabsTrigger value="usage">Usage</TabsTrigger>
              <TabsTrigger value="actions">Actions</TabsTrigger>
              <TabsTrigger value="reminders">Reminders</TabsTrigger>
            </TabsList>
            <TabsContent value="usage" className="flex-1 flex flex-col">
              <AppUsageBarChartView
                target={target}
                usageLimit={usageLimit}
                timeFrame={timeFrame}
              />
            </TabsContent>
            <TabsContent value="actions">
              <div>TODO show action video</div>
            </TabsContent>
            <TabsContent value="reminders">
              <div className="flex h-full">
                {!usageLimit || !timeFrame || !target ? (
                  <Alert className="m-auto">
                    <InfoIcon className="size-4" />
                    <AlertTitle>Choose options first</AlertTitle>
                    <AlertDescription>
                      Select target, period and usage limit to show reminder and
                      usage progress.
                    </AlertDescription>
                  </Alert>
                ) : (
                  <div className="flex-1 flex flex-col gap-2 my-auto">
                    <div className="grid grid-cols-2 gap-4 mb-1">
                      <div className="flex flex-col">
                        <span className="text-sm font-medium text-muted-foreground">
                          Current Usage
                        </span>
                        <div className="flex items-baseline gap-2">
                          <DurationText
                            className="text-lg font-semibold pr-1"
                            ticks={triggerInfo.currentUsage}
                          />
                          {triggerInfo.currentUsage !== 0 && (
                            <span
                              className={`text-sm tabular-nums ${
                                triggerInfo.currentUsage / usageLimit >= 1
                                  ? "text-destructive"
                                  : "text-muted-foreground"
                              }`}
                            >
                              {Math.min(
                                100,
                                (triggerInfo.currentUsage / usageLimit) * 100,
                              ).toFixed(0)}
                              %
                            </span>
                          )}
                        </div>
                      </div>
                      <div className="flex flex-col items-end">
                        <span className="text-sm font-medium text-muted-foreground">
                          Usage Limit
                        </span>
                        <DurationText
                          className="text-lg font-semibold pl-1"
                          ticks={usageLimit}
                        />
                      </div>
                    </div>
                    <TimeProgressBar
                      usageLimit={usageLimit}
                      currentUsage={triggerInfo.currentUsage}
                      reminders={reminders}
                      circleRadius={12}
                      onReminderAdd={(v) => append({ ...v })}
                      onReminderUpdate={handleReminderUpdate}
                    />
                  </div>
                )}
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </main>
    </>
  );
}

const hoursInPeriod = (period: Period) => {
  switch (period) {
    case "hour":
      return 1;
    case "day":
      return 24;
    case "week":
      return 24 * 7;
    // assume 30 days per month
    case "month":
      return 24 * 30;
    // assume 365 days per year
    case "year":
      return 24 * 365;
  }
};

export function AppUsageBarChartView({
  target,
  usageLimit,
  timeFrame,
}: {
  target?: Target;
  usageLimit: number;
  timeFrame: TimeFrame;
}) {
  const [period, setPeriod] = useState<Period>("day");

  const { interval, setInterval, canGoNext, goNext, canGoPrev, goPrev } =
    useIntervalControlsWithDefault("week");

  const scaledUsageLimit = useMemo(() => {
    if (!usageLimit || !timeFrame) return undefined;

    const perHourUsageLimit =
      usageLimit / hoursInPeriod(timeFrameToPeriod(timeFrame));
    return perHourUsageLimit * hoursInPeriod(period);
  }, [period, usageLimit, timeFrame]);

  const {
    usages: appUsages,
    period: loadPeriod,
    start,
    end,
  } = useAppDurationsPerPeriod({
    start: interval?.start,
    end: interval?.end,
    period: period,
  });

  const [yAxisInterval, maxYIsPeriod] = useMemo(() => {
    switch (period) {
      case "hour":
        return [Duration.fromObject({ minutes: 15 }), true];
      case "day":
        return [Duration.fromObject({ hours: 2 }), false];
      case "week":
        return [Duration.fromObject({ hours: 6 }), false];
      case "month":
        return [Duration.fromObject({ days: 1 }), false];
      default:
        throw new Error(`Unknown period: ${period}`);
    }
    // this should take period as a dependency, but we only take in loadPeriod
    // which is a output of useAppDurationsPerPeriod, else we get yaxis flashes
    // with the older data's yaxis interval before the data is loading

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loadPeriod]);

  const targetApps = useTargetApps(target);

  return (
    <div className="flex flex-1">
      <div className="flex-1 my-auto space-y-4 max-w-full">
        <div className="flex gap-4 justify-end">
          <FormItem>
            <Label className="font-medium text-muted-foreground place-self-end">
              Period
            </Label>
            <Select
              value={period}
              onValueChange={(s) => setPeriod(s as Period)}
            >
              <SelectTrigger className="min-w-32 font-medium">
                <SelectValue placeholder="Select a period" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="hour">Hour</SelectItem>
                <SelectItem value="day">Day</SelectItem>
                <SelectItem value="week">Week</SelectItem>
                <SelectItem value="month">Month</SelectItem>
              </SelectContent>
            </Select>
          </FormItem>

          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="icon"
              disabled={!canGoPrev}
              onClick={goPrev}
              className="mt-6"
            >
              <ChevronLeftIcon className="size-4" />
            </Button>
            <FormItem>
              <Label className="font-medium text-muted-foreground place-self-end">
                Time Range
              </Label>
              <DateRangePicker
                value={interval}
                onChange={setInterval}
                dayGranularity={true}
                className="w-full min-w-32"
              />
            </FormItem>

            <Button
              variant="ghost"
              size="icon"
              disabled={!canGoNext}
              onClick={goNext}
              className="mt-6"
            >
              <ChevronRightIcon className="size-4" />
            </Button>
          </div>
        </div>
        <div className="aspect-video ">
          <AppUsageBarChart
            data={appUsages}
            highlightedAppIds={targetApps?.map((app) => app.id) ?? undefined}
            markerLines={
              scaledUsageLimit
                ? [
                    {
                      yAxis: scaledUsageLimit,
                      type: "dashed",
                    },
                  ]
                : undefined
            }
            period={loadPeriod ?? period}
            start={start ?? interval?.start ?? DateTime.now()}
            end={end ?? interval?.end ?? DateTime.now()}
            className="w-full h-full"
            maxYIsPeriod={maxYIsPeriod}
            interval={yAxisInterval}
            animationsEnabled={false}
            barRadius={3}
          />
        </div>
      </div>
    </div>
  );
}

export function TimeProgressBar({
  usageLimit,
  currentUsage,
  reminders,
  circleRadius,
  onReminderAdd,
  onReminderUpdate,
}: {
  usageLimit: number;
  currentUsage: number;
  reminders: { id?: number; threshold: number; message: string }[];
  circleRadius: number;
  onReminderAdd?: (params: { threshold: number; message: string }) => void;
  onReminderUpdate?: (index: number, threshold: number) => void;
}) {
  const percentage = (currentUsage / usageLimit) * 100;
  const progressBarRef = useRef<HTMLDivElement>(null);

  // State for hover and drag interactions
  const [hoverState, setHoverState] = useState<{ x: number; visible: boolean }>(
    { x: 0, visible: false },
  );
  const [dragState, setDragState] = useState<{ index: number | null }>({
    index: null,
  });
  const preventNextClick = useRef(false);

  // Convert mouse position to threshold value (0-1)
  const getThresholdFromEvent = useCallback((e: React.MouseEvent) => {
    if (!progressBarRef.current) return 0;
    const rect = progressBarRef.current.getBoundingClientRect();
    return Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
  }, []);

  // Convert mouse position to percentage for hover indicator (0-100)
  const getPercentageFromEvent = useCallback((e: React.MouseEvent) => {
    if (!progressBarRef.current) return 0;
    const rect = progressBarRef.current.getBoundingClientRect();
    return Math.max(
      0,
      Math.min(100, ((e.clientX - rect.left) / rect.width) * 100),
    );
  }, []);

  // Mouse event handlers
  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (dragState.index !== null) {
        // Handle dragging
        if (onReminderUpdate) {
          onReminderUpdate(dragState.index, getThresholdFromEvent(e));
        }
      } else {
        // Handle hover
        setHoverState({ x: getPercentageFromEvent(e), visible: true });
      }
    },
    [
      dragState.index,
      onReminderUpdate,
      getThresholdFromEvent,
      getPercentageFromEvent,
    ],
  );

  const handleMouseLeave = useCallback(() => {
    if (dragState.index === null) {
      setHoverState((prev) => ({ ...prev, visible: false }));
    }
  }, [dragState.index]);

  const handleMouseDown = useCallback((e: React.MouseEvent, index: number) => {
    e.stopPropagation();
    setDragState({ index });
  }, []);

  const handleMouseUp = useCallback(() => {
    if (dragState.index !== null) {
      preventNextClick.current = true;
      // Reset after a short delay to allow for future clicks
      setTimeout(() => {
        preventNextClick.current = false;
      }, 0);
    }
    setDragState({ index: null });
  }, [dragState.index]);

  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      if (
        !onReminderAdd ||
        dragState.index !== null ||
        preventNextClick.current
      )
        return;
      onReminderAdd({ threshold: getThresholdFromEvent(e), message: "" });
    },
    [onReminderAdd, dragState.index, getThresholdFromEvent],
  );

  // Global mouse up handler
  useEffect(() => {
    if (dragState.index !== null) {
      const handleGlobalMouseUp = () => handleMouseUp();
      document.addEventListener("mouseup", handleGlobalMouseUp);
      return () => document.removeEventListener("mouseup", handleGlobalMouseUp);
    }
  }, [dragState.index, handleMouseUp]);

  // Render reminder markers
  const renderReminder = useCallback(
    (reminder: { threshold: number; message: string }, index: number) => {
      const isDragging = dragState.index === index;
      return (
        <Fragment key={index}>
          <div
            className={`absolute top-1/2 bg-primary border border-border rounded-full cursor-move transition-shadow
              ${isDragging ? "ring-2 ring-primary shadow-lg" : "hover:ring-1 hover:ring-primary/50"}`}
            style={{
              left: `${reminder.threshold * 100}%`,
              width: circleRadius * 2,
              height: circleRadius * 2,
              transform: `translate(-${circleRadius}px, -50%)`,
            }}
            onMouseDown={(e) => handleMouseDown(e, index)}
          />
          <div
            className="absolute text-popover-foreground bg-popover border border-border rounded-md shadow-lg max-w-[200px]"
            style={{
              left: `${reminder.threshold * 100}%`,
              transform: `translate(-50%, 4px)`,
            }}
          >
            <p className="absolute border-border bg-popover fill-popover z-50 size-2.5 translate-y-[calc(-50%_-_2px)] -translate-x-1/2 origin-center left-1/2 rotate-45 rounded-[2px] border-l border-t" />
            <div className="p-2 space-y-1">
              <div className="flex items-center gap-2">
                <div className="text-xs font-medium">{`${((reminder.threshold || 0) * 100).toFixed(0)}%`}</div>
                <div className="text-xs text-muted-foreground">
                  <DurationText
                    ticks={(reminder.threshold || 0) * usageLimit}
                  />
                </div>
              </div>
              {reminder.message && (
                <div className="text-sm line-clamp-2 break-words">
                  {reminder.message}
                </div>
              )}
            </div>
          </div>
        </Fragment>
      );
    },
    [circleRadius, dragState.index, usageLimit, handleMouseDown],
  );

  return (
    <div
      ref={progressBarRef}
      className="relative w-full bg-secondary h-8 rounded-sm my-auto"
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      onClick={handleClick}
      onMouseUp={handleMouseUp}
    >
      {/* Progress bar */}
      <div
        className="h-full bg-primary/90 rounded-sm transition-all"
        style={{ width: `${Math.min(100, percentage)}%` }}
      />

      {/* Hover indicator */}
      {hoverState.visible &&
        dragState.index === null &&
        !preventNextClick.current && (
          <div
            className="absolute top-1/2 bg-primary/30 border border-primary/50 rounded-full flex items-center justify-center 
            cursor-pointer hover:bg-primary/40 transition-colors hover:scale-110"
            style={{
              left: `${hoverState.x}%`,
              width: circleRadius * 2,
              height: circleRadius * 2,
              transform: `translate(-${circleRadius}px, -50%)`,
            }}
          >
            <PlusIcon className="size-4 text-primary" />
          </div>
        )}

      {/* Reminder markers */}
      {reminders.map((reminder, index) => renderReminder(reminder, index))}
    </div>
  );
}
