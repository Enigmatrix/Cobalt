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
import type { Period, Target, TimeFrame } from "@/lib/entities";
import { DateTime } from "luxon";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { Tabs, TabsContent } from "@/components/ui/tabs";
import { TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Label } from "@/components/ui/label";
import { DateRangePicker } from "@/components/time/date-range-picker";
import { useTargetApps } from "@/hooks/use-refresh";
import type { UseFormReturn } from "react-hook-form";
import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { useFieldArray } from "react-hook-form";

type FormValues = z.infer<typeof alertSchema>;

export default function CreateAlerts() {
  const form = useZodForm({
    schema: alertSchema,
    defaultValues: {
      reminders: [],
    },
  });
  const createAlert = useAppState((state) => state.createAlert);
  const navigate = useNavigate();
  const onSubmit = useCallback(
    async (values: FormValues) => {
      await createAlert(values);
      navigate("/alerts");
    },
    [createAlert, navigate],
  );
  const target = form.watch("target");
  const usageLimit = form.watch("usage_limit");
  const timeFrame = form.watch("time_frame");

  return (
    <>
      <main className="grid grid-cols-[360px_minmax(0,1fr)] h-full ">
        <CreateAlertForm onSubmit={onSubmit} form={form} />
        <div className="flex flex-col p-4">
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
}: {
  onSubmit: (values: FormValues) => void;
  form: UseFormReturn<FormValues>;
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
            <div key={field.id} className="flex gap-2 items-end">
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
                        className="w-24"
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
    },
  ];

  return (
    <>
      <Form {...form}>
        <form
          onSubmit={form.handleSubmit(onSubmit)}
          className="w-[360px] space-y-4 px-8 m-auto py-8"
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

    const toDuration = (timeFrame: TimeFrame) => {
      switch (timeFrame) {
        case "Daily":
          return 24;
        case "Weekly":
          return 24 * 7;
        case "Monthly":
          return 24 * 30;
      }
    };

    const perHourUsageLimit = usageLimit / toDuration(timeFrame);
    switch (period) {
      case "hour":
        return perHourUsageLimit;
      case "day":
        return perHourUsageLimit * 24;
      case "week":
        return perHourUsageLimit * 24 * 7;
      case "month":
        return perHourUsageLimit * 24 * 30;
      case "year":
        return perHourUsageLimit * 24 * 365;
    }
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
            hightlightedAppIds={targetApps?.map((app) => app.id) ?? undefined}
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
