import { useZodForm } from "@/hooks/use-form";
import { alertSchema } from "@/lib/schema";
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
import { useCallback, useMemo, useState } from "react";
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
import type { UseFormReturn } from "react-hook-form";
import {
  TriangleAlertIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from "lucide-react";
import { useFieldArray } from "react-hook-form";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Checkbox } from "@/components/ui/checkbox";
import type { CreateAlert } from "@/lib/repo";
import { Badge } from "@/components/ui/badge";

type FormValues = z.infer<typeof alertSchema>;

interface TriggerInfo {
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
      return { alert: false, reminders: [] };

    const alert = totalUsage > usageLimit;
    const reminderTriggers = (reminders ?? []).map((reminder) => ({
      id: reminder.id,
      trigger: totalUsage > reminder.threshold * usageLimit,
    }));
    return { alert, reminders: reminderTriggers };
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

  return (
    <>
      <main className="grid grid-cols-[360px_minmax(0,1fr)] h-full ">
        <div className="max-h-screen overflow-y-auto my-auto">
          <CreateAlertForm
            onSubmit={onSubmit}
            form={form}
            triggerInfo={triggerInfo}
          />
        </div>
        <div className="flex flex-col p-8">
          <Tabs defaultValue="usage" className="flex-1 flex flex-col">
            <TabsList className="self-center">
              <TabsTrigger value="usage">Usage</TabsTrigger>
              <TabsTrigger value="actions">Actions</TabsTrigger>
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
}: {
  onSubmit: (values: FormValues) => void;
  form: UseFormReturn<FormValues>;
  triggerInfo: TriggerInfo;
}) {
  const { fields, append, remove } = useFieldArray({
    control: form.control,
    name: "reminders",
  });

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
            <div key={field.id} className="flex gap-2 items-center">
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
                        step={0.01}
                        placeholder="Threshold (0-1)"
                        className="w-16"
                        {...field}
                        onChange={(e) =>
                          field.onChange(parseFloat(e.target.value))
                        }
                      />
                    </FormControl>
                    <FormMessage />
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
                    <FormMessage />
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
