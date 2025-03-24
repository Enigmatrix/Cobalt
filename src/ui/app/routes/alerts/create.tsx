import { useZodForm } from "@/hooks/use-form";
import { alertSchema } from "@/lib/schema";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import type { z } from "zod";
import { ChooseTarget } from "@/components/alert/choose-target";
import { DurationPicker } from "@/components/time/duration-picker";
import {
  durationToTicks,
  findIntervalPeriod,
  nextInterval,
  prevInterval,
  ticksToDuration,
  type Interval,
} from "@/lib/time";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import {
  Timeline,
  TimelineContent,
  TimelineIndicator,
  TimelineTitle,
  TimelineSeparator,
  TimelineHeader,
  TimelineItem,
} from "@/components/ui/timeline";
import { Button } from "@/components/ui/button";
import { useCallback, useMemo, useState, useRef, useEffect } from "react";
import { useAppState } from "@/lib/state";
import { useNavigate } from "react-router";
import { Duration } from "luxon";
import { useTimePeriod, useToday } from "@/hooks/use-today";
import {
  timeFrameToPeriod,
  type Period,
  type Target,
  type TimeFrame,
} from "@/lib/entities";
import { DateTime } from "luxon";
import { useAppDurations, useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { Tabs, TabsContent } from "@/components/ui/tabs";
import { TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Label } from "@/components/ui/label";
import { DateRangePicker } from "@/components/time/date-range-picker";
import { useTargetApps } from "@/hooks/use-refresh";
import type { FieldArrayWithId, UseFormReturn } from "react-hook-form";
import {
  TriangleAlertIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  InfoIcon,
  PlusIcon,
} from "lucide-react";
import { useFieldArray } from "react-hook-form";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Checkbox } from "@/components/ui/checkbox";
import type { CreateAlert } from "@/lib/repo";
import { Badge } from "@/components/ui/badge";
import { DurationText } from "@/components/time/duration-text";

type FormValues = z.infer<typeof alertSchema>;

interface TriggerInfo {
  currentUsage: number;
  alert: boolean;
  reminders: { id?: number; trigger: boolean }[];
}

export function useTriggerInfo(
  target?: Target,
  usageLimit?: number,
  timeFrame?: TimeFrame,
  reminders?: { id?: number; threshold: number; message: string }[],
): TriggerInfo {
  const interval = useTimePeriod(timeFrameToPeriod(timeFrame ?? "Daily"));

  const targetApps = useTargetApps(target);
  const { ret: appDurations } = useAppDurations({
    start: interval.start,
    end: interval.end,
  });
  const totalUsage = useMemo(() => {
    if (!targetApps) return 0;
    return targetApps.reduce(
      (acc, curr) => acc + (appDurations[curr.id]?.duration ?? 0),
      0,
    );
  }, [appDurations, targetApps]);

  const triggerInfo = useMemo(() => {
    if (!usageLimit || !timeFrame || !target)
      return { alert: false, reminders: [], currentUsage: totalUsage };

    const alert = totalUsage > usageLimit;
    const reminderTriggers = (reminders ?? []).map((reminder) => ({
      id: reminder.id,
      trigger: totalUsage > reminder.threshold * usageLimit,
    }));
    return { alert, reminders: reminderTriggers, currentUsage: totalUsage };
  }, [totalUsage, target, timeFrame, usageLimit, reminders]);

  return triggerInfo;
}

export default function CreateAlerts() {
  const form = useZodForm({
    schema: alertSchema,
    defaultValues: {
      ignore_trigger: false,
      reminders: [],
    },
  });
  const createAlert = useAppState((state) => state.createAlert);
  const navigate = useNavigate();
  const target = form.watch("target");
  const usageLimit = form.watch("usage_limit");
  const timeFrame = form.watch("time_frame");
  const reminders = form.watch("reminders");
  const { fields, append, remove, update } = useFieldArray({
    control: form.control,
    name: "reminders",
  });

  const triggerInfo = useTriggerInfo(target, usageLimit, timeFrame, reminders);

  const onSubmit = useCallback(
    async (values: FormValues) => {
      // FormValues is not the same as CreateAlert
      // the ignore_trigger in FormValues means ignore all firing alerts and reminders
      // but in CreateAlert, it's customizable for each alert and reminder.

      const object: CreateAlert = {
        ...structuredClone(values),
        ignore_trigger: false,
        reminders: values.reminders.map((reminder) => ({
          ...reminder,
          ignore_trigger: false,
        })),
      };

      if (values.ignore_trigger) {
        object.ignore_trigger = triggerInfo.alert;
        object.reminders.forEach((reminder, index) => {
          reminder.ignore_trigger = triggerInfo.reminders[index].trigger;
        });
      }

      await createAlert(object);
      navigate("/alerts");
    },
    [createAlert, navigate, triggerInfo],
  );

  const handleReminderUpdate = useCallback(
    (index: number, threshold: number) => {
      update(index, { threshold, message: fields[index].message });
    },
    [update, fields],
  );

  return (
    <>
      <main className="grid grid-cols-[360px_minmax(0,1fr)] h-full ">
        <div className="max-h-screen overflow-y-auto my-auto">
          <CreateAlertForm
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
                      usage_limit={usageLimit}
                      current_usage={triggerInfo.currentUsage}
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

export function CreateAlertForm({
  onSubmit,
  form,
  triggerInfo,
  remindersFields: fields,
  remindersAppend: append,
  remindersRemove: remove,
}: {
  onSubmit: (values: FormValues) => void;
  form: UseFormReturn<FormValues>;
  triggerInfo: TriggerInfo;
  remindersFields: FieldArrayWithId<FormValues, "reminders", "id">[];
  remindersAppend: (value: { threshold: number; message: string }) => void;
  remindersRemove: (index: number) => void;
}) {
  const items = [
    {
      id: 1,
      title: "Target",
      content: (
        <FormField
          control={form.control}
          name="target"
          render={({ field: { value, onChange, ...field } }) => (
            <FormItem>
              <FormControl>
                <ChooseTarget
                  {...field}
                  value={value}
                  onValueChanged={onChange}
                  className="w-full justify-start"
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
      ),
    },
    {
      id: 2,
      title: "Limit",
      content: (
        <>
          <FormField
            control={form.control}
            name="time_frame"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Period</FormLabel>
                <FormControl>
                  <Select
                    {...field}
                    value={field.value}
                    onValueChange={field.onChange}
                  >
                    <SelectTrigger className="hover:bg-muted w-full">
                      <SelectValue placeholder="Period" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="Daily">Daily</SelectItem>
                      <SelectItem value="Weekly">Weekly</SelectItem>
                      <SelectItem value="Monthly">Monthly</SelectItem>
                    </SelectContent>
                  </Select>
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="usage_limit"
            render={({ field: { value, onChange, ...field } }) => (
              <FormItem>
                <FormLabel>Limit</FormLabel>
                <FormControl>
                  <DurationPicker
                    showIcon={false}
                    className="w-full text-muted-foreground"
                    {...field}
                    value={value === undefined ? null : ticksToDuration(value)}
                    onValueChange={(dur) =>
                      onChange(dur === null ? undefined : durationToTicks(dur))
                    }
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
        </>
      ),
    },
    {
      id: 3,
      title: "Action",
      content: (
        <>
          <FormField
            control={form.control}
            name="trigger_action"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Select
                    {...field}
                    value={field.value?.tag}
                    onValueChange={(v) => {
                      // Reset the form when changing action type
                      if (v === "Kill") {
                        field.onChange({ tag: v });
                      } else if (v === "Dim") {
                        field.onChange({ tag: v, duration: undefined });
                      } else if (v === "Message") {
                        field.onChange({ tag: v, content: "" });
                      }
                    }}
                  >
                    <SelectTrigger className="hover:bg-muted w-full">
                      <SelectValue placeholder="Type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="Dim">Dim</SelectItem>
                      <SelectItem value="Message">Message</SelectItem>
                      <SelectItem value="Kill">Kill</SelectItem>
                    </SelectContent>
                  </Select>
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="trigger_action"
            render={({ field: { value, onChange, ...field } }) => (
              <>
                {value?.tag === "Dim" && (
                  <FormItem>
                    <FormLabel>Dim Duration</FormLabel>
                    <FormControl>
                      <DurationPicker
                        showIcon={false}
                        className="w-full text-foreground"
                        {...field}
                        value={
                          value?.duration === undefined
                            ? null
                            : ticksToDuration(value.duration)
                        }
                        onValueChange={(dur) =>
                          onChange({
                            tag: "Dim",
                            duration:
                              dur === null ? undefined : durationToTicks(dur),
                          })
                        }
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              </>
            )}
          />

          <FormField
            control={form.control}
            name="trigger_action"
            render={({ field }) => (
              <>
                {field.value?.tag === "Message" && (
                  <FormItem>
                    <FormLabel>Message Content</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        value={field.value.content ?? ""}
                        onChange={(e) =>
                          field.onChange({
                            tag: "Message",
                            content: e.target.value,
                          })
                        }
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              </>
            )}
          />
        </>
      ),
    },
    {
      id: 4,
      title: (
        <div className="flex">
          <div>Reminders</div>
          <Button
            variant="outline"
            size="sm"
            type="button"
            className="ml-auto"
            onClick={() => {
              append({ threshold: 0.5, message: "" });
            }}
          >
            Add
          </Button>
        </div>
      ),
      content: (
        <>
          {fields.map((field, index) => (
            <div key={field.id} className="flex flex-col gap-2">
              <div className="flex gap-2 items-center">
                <Badge variant="outline" className="">
                  {index + 1}
                </Badge>
                <FormField
                  control={form.control}
                  name={`reminders.${index}.threshold`}
                  render={({ field }) => (
                    <FormItem>
                      <FormControl>
                        <Input
                          type="number"
                          min={0}
                          max={1}
                          step="any"
                          placeholder="Threshold (0-1)"
                          className="w-16"
                          {...field}
                          onChange={(e) =>
                            field.onChange(parseFloat(e.target.value))
                          }
                        />
                      </FormControl>
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name={`reminders.${index}.message`}
                  render={({ field }) => (
                    <FormItem className="flex-1">
                      <FormControl>
                        <Input placeholder="Reminder message" {...field} />
                      </FormControl>
                    </FormItem>
                  )}
                />
                <Button
                  type="button"
                  variant="destructive"
                  size="icon"
                  onClick={() => remove(index)}
                >
                  <span className="sr-only">Delete reminder</span>×
                </Button>
              </div>

              <div className="flex flex-col">
                <FormField
                  control={form.control}
                  name={`reminders.${index}.threshold`}
                  render={() => <FormMessage />}
                />
                <FormField
                  control={form.control}
                  name={`reminders.${index}.message`}
                  render={() => <FormMessage />}
                />
              </div>
            </div>
          ))}
        </>
      ),
    },
    {
      id: 5,
      title: (
        <Button type="submit" className="-mt-1">
          Submit
        </Button>
      ),
      content: (
        <>
          {(triggerInfo.alert ||
            triggerInfo.reminders.some((r) => r.trigger)) && (
            <Alert className="mt-2">
              <TriangleAlertIcon className="h-4 w-4" />
              <AlertTitle>Warning</AlertTitle>
              <AlertDescription>
                {triggerInfo.alert &&
                triggerInfo.reminders.some((r) => r.trigger) ? (
                  <div>
                    This alert and{" "}
                    {triggerInfo.reminders.filter((r) => r.trigger).length}{" "}
                    {triggerInfo.reminders.filter((r) => r.trigger).length === 1
                      ? "reminder"
                      : "reminders"}{" "}
                    will immediately trigger once submitted.
                  </div>
                ) : triggerInfo.alert &&
                  triggerInfo.reminders.every((r) => !r.trigger) ? (
                  <div>This alert will immediately trigger once submitted.</div>
                ) : triggerInfo.reminders.some((r) => r.trigger) ? (
                  <div>
                    {triggerInfo.reminders.filter((r) => r.trigger).length}{" "}
                    {triggerInfo.reminders.filter((r) => r.trigger).length === 1
                      ? "reminder"
                      : "reminders"}{" "}
                    will immediately trigger once submitted.
                  </div>
                ) : null}

                <FormField
                  control={form.control}
                  name="ignore_trigger"
                  render={({ field: { value, onChange, ...field } }) => (
                    <FormItem className="mt-4 flex gap-2 items-center">
                      <FormControl>
                        <Checkbox
                          checked={value}
                          onCheckedChange={onChange}
                          {...field}
                        />
                      </FormControl>
                      <FormLabel>Ignore for current period</FormLabel>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </AlertDescription>
            </Alert>
          )}
        </>
      ),
    },
  ];

  return (
    <>
      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSubmit)}
          className="space-y-4 p-8 m-auto"
        >
          <Timeline value={0}>
            {items.map((item) => (
              <TimelineItem key={item.id} step={item.id}>
                <TimelineHeader>
                  <TimelineSeparator className="bg-primary/60 mt-2.5 group-data-[orientation=vertical]/timeline:h-[calc(100%-2rem-0.25rem)]" />
                  <TimelineTitle className="-mt-1/2 text-base">
                    {item.title}
                  </TimelineTitle>
                  <TimelineIndicator className="border-primary/80 group-data-completed/timeline-item:bg-primary group-data-completed/timeline-item:text-primary-foreground flex size-6 items-center justify-center group-data-completed/timeline-item:border-none group-data-[orientation=vertical]/timeline:-left-6" />
                </TimelineHeader>
                {item.content && (
                  <TimelineContent className="mt-2 space-y-4">
                    {item.content}
                  </TimelineContent>
                )}
              </TimelineItem>
            ))}
          </Timeline>

          <div className="flex justify-end mt-4"></div>
        </form>
      </Form>
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
  const today = useToday();
  const week = useTimePeriod("week");
  const [interval, setInterval] = useState<Interval | null>(week);
  const [period, setPeriod] = useState<Period>("day");

  // TODO commonolize this with usage-card.tsx if there are more usages!
  const guessedIntervalPeriod = useMemo(() => {
    if (!interval) return undefined;
    return findIntervalPeriod(interval);
  }, [interval]);

  const canGoPrevInterval = useMemo(() => {
    return !!guessedIntervalPeriod; // can always go back as long as we have an interval
  }, [guessedIntervalPeriod]);

  const canGoNextInterval = useMemo(() => {
    if (!interval || !guessedIntervalPeriod) return false;
    const nextStart = nextInterval(interval, guessedIntervalPeriod);
    return today.plus({ day: 1 }) > nextStart.start;
  }, [guessedIntervalPeriod, interval, today]);

  const goPrevInterval = useCallback(() => {
    if (!interval || !guessedIntervalPeriod) return;
    const newInterval = prevInterval(interval, guessedIntervalPeriod);
    setInterval(newInterval);
  }, [guessedIntervalPeriod, interval]);

  const goNextInterval = useCallback(() => {
    if (!interval || !guessedIntervalPeriod) return;
    const newInterval = nextInterval(interval, guessedIntervalPeriod);
    setInterval(newInterval);
  }, [guessedIntervalPeriod, interval]);

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
              disabled={!canGoPrevInterval}
              onClick={goPrevInterval}
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
              disabled={!canGoNextInterval}
              onClick={goNextInterval}
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

function TimeProgressBar({
  usage_limit,
  current_usage,
  reminders,
  circleRadius,
  onReminderAdd,
  onReminderUpdate,
}: {
  usage_limit: number;
  current_usage: number;
  reminders: { id?: number; threshold: number; message: string }[];
  circleRadius: number;
  onReminderAdd?: (params: { threshold: number; message: string }) => void;
  onReminderUpdate?: (index: number, threshold: number) => void;
}) {
  const percentage = (current_usage / usage_limit) * 100;
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
        <Tooltip key={index}>
          <TooltipTrigger asChild>
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
          </TooltipTrigger>
          <TooltipContent>
            <p>{`${(reminder.threshold * 100).toFixed(0)}% - ${reminder.message}`}</p>
          </TooltipContent>
        </Tooltip>
      );
    },
    [circleRadius, dragState.index, handleMouseDown],
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
