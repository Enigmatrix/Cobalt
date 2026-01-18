import { ChooseTarget } from "@/components/alert/choose-target";
import dimVideo from "@/components/alert/dim.mp4";
import killVideo from "@/components/alert/kill.mp4";
import messageVideo from "@/components/alert/message.mp4";
import { DateRangePicker } from "@/components/time/date-range-picker";
import { DurationPicker } from "@/components/time/duration-picker";
import { DurationText } from "@/components/time/duration-text";
import { PeriodPicker } from "@/components/time/period-picker";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
  type CarouselApi,
} from "@/components/ui/carousel";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { UsageChart } from "@/components/viz/usage-chart";
import { useTargetApps } from "@/hooks/use-refresh";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { useIntervalControlsWithDefault } from "@/hooks/use-time";
import { useTriggerInfo } from "@/hooks/use-trigger-info";
import { timeFrameToPeriod, type TriggerAction } from "@/lib/entities";
import { alertSchema } from "@/lib/schema";
import { durationToTicks, ticksToDuration, type Period } from "@/lib/time";
import { cn } from "@/lib/utils";
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  InfoIcon,
  MessageSquareIcon,
  PlusIcon,
  PointerIcon,
  SunIcon,
  TriangleAlertIcon,
  XIcon,
  ZapIcon,
} from "lucide-react";
import { Duration } from "luxon";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  FormProvider,
  useFieldArray,
  useFormContext,
  useWatch,
  type UseFormReturn,
} from "react-hook-form";
import type { z } from "zod";

export type FormValues = z.infer<typeof alertSchema>;

export function AlertFormContainer({
  onSubmit,
  form,
  submitButtonText = "Create Alert",
}: {
  onSubmit: (values: FormValues) => void;
  form: UseFormReturn<FormValues>;
  submitButtonText?: string;
}) {
  return (
    <FormProvider {...form}>
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          {/* Card 1: Target, Usage Limit, Time Period with Usage Preview */}
          <TargetAndUsageCard />

          {/* Card 2: Action with Preview */}
          <ActionCard />

          {/* Card 3: Reminders with Preview */}
          <RemindersCard />

          {/* Submit Section */}
          <SubmitSection submitButtonText={submitButtonText} />
        </form>
      </Form>
    </FormProvider>
  );
}

function TargetAndUsageCard() {
  const { control } = useFormContext<FormValues>();

  return (
    <Card>
      <CardHeader>
        <CardTitle>Target & Limits</CardTitle>
        <CardDescription>
          Choose what to target and set usage limits
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Target Selection */}
        <FormField
          control={control}
          name="target"
          render={({ field: { value, onChange, ...field } }) => (
            <FormItem>
              <FormLabel>Target</FormLabel>
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

        {/* Time Period and Usage Limit using subgrid */}
        <div className="grid grid-cols-2 grid-rows-[auto_auto_auto] gap-x-4 gap-y-2">
          <FormField
            control={control}
            name="timeFrame"
            render={({ field }) => (
              <FormItem className="row-span-3 grid grid-rows-subgrid gap-y-2">
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
                      <SelectItem value="daily">Daily</SelectItem>
                      <SelectItem value="weekly">Weekly</SelectItem>
                      <SelectItem value="monthly">Monthly</SelectItem>
                    </SelectContent>
                  </Select>
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={control}
            name="usageLimit"
            render={({ field: { value, onChange, ...field } }) => (
              <FormItem className="row-span-3 grid grid-rows-subgrid gap-y-2">
                <FormLabel>Limit</FormLabel>
                <FormControl>
                  <DurationPicker
                    showIcon={false}
                    className="w-full"
                    {...field}
                    value={
                      value === undefined || value === null
                        ? null
                        : ticksToDuration(value)
                    }
                    onValueChange={(dur) =>
                      onChange(!dur ? null : durationToTicks(dur))
                    }
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
        </div>

        {/* Usage Preview Chart */}
        <UsagePreview />
      </CardContent>
    </Card>
  );
}

const actionTypes = [
  {
    tag: "dim" as const,
    title: "Dim",
    description: "Screen gradually dims to discourage usage",
    icon: SunIcon,
    defaultAction: {
      tag: "dim" as const,
      duration: durationToTicks(Duration.fromObject({ minutes: 30 })),
    },
    videoSrc: dimVideo,
    videoMessages: [
      { start: 0.5, end: 15, message: "Websites and apps dim gradually" },
      {
        start: 16,
        end: 20,
        message: "Switching to other websites changes dim",
      },
      { start: 22, end: 26, message: "Works on newly opened websites" },
    ],
  },
  {
    tag: "message" as const,
    title: "Message",
    description: "Shows a notification with custom message",
    icon: MessageSquareIcon,
    defaultAction: { tag: "message" as const, content: "" },
    videoSrc: messageVideo,
    videoMessages: [
      { start: 1, end: 6, message: "Notification appears with sound" },
    ],
  },
  {
    tag: "kill" as const,
    title: "Kill",
    description: "Apps and websites are forcefully closed",
    icon: ZapIcon,
    defaultAction: { tag: "kill" as const },
    videoSrc: killVideo,
    videoMessages: [
      { start: 2, end: 6, message: "Apps are closed immediately" },
      { start: 6, end: 8, message: "Websites are closed when opened" },
      { start: 11, end: 14, message: "Works on newly opened websites" },
    ],
  },
];

function ActionCard() {
  const { control, setValue } = useFormContext<FormValues>();

  const setTriggerAction = useCallback(
    (action: TriggerAction) => {
      setValue("triggerAction", action);
    },
    [setValue],
  );

  return (
    <Card>
      <CardHeader>
        <CardTitle>Action</CardTitle>
        <CardDescription>
          What to do when the usage limit is reached
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Action Type Selection - Button Grid */}
        <FormField
          control={control}
          name="triggerAction"
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <div className="grid grid-cols-3 gap-3">
                  {actionTypes.map((action) => {
                    const isSelected = field.value?.tag === action.tag;
                    const Icon = action.icon;
                    return (
                      <button
                        key={action.tag}
                        type="button"
                        onClick={() => field.onChange(action.defaultAction)}
                        className={cn(
                          "flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-all text-left",
                          isSelected
                            ? "border-primary bg-primary/5"
                            : "border-border hover:border-primary/50 hover:bg-muted/50",
                        )}
                      >
                        <Icon
                          className={cn(
                            "size-6",
                            isSelected
                              ? "text-primary"
                              : "text-muted-foreground",
                          )}
                        />
                        <div className="text-center">
                          <div
                            className={cn(
                              "font-medium text-sm",
                              isSelected ? "text-primary" : "text-foreground",
                            )}
                          >
                            {action.title}
                          </div>
                          <div className="text-xs text-muted-foreground mt-0.5 line-clamp-2">
                            {action.description}
                          </div>
                        </div>
                      </button>
                    );
                  })}
                </div>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        {/* Conditional Fields based on action type */}
        <FormField
          control={control}
          name="triggerAction"
          render={({ field: { value, onChange, ...field } }) => (
            <>
              {value?.tag === "dim" && (
                <FormItem>
                  <FormLabel>Dim Duration</FormLabel>
                  <FormControl>
                    <DurationPicker
                      showIcon={false}
                      className="w-full text-muted-foreground"
                      {...field}
                      value={
                        value?.duration === undefined ||
                        value?.duration === null
                          ? null
                          : ticksToDuration(value.duration)
                      }
                      onValueChange={(dur) =>
                        onChange({
                          tag: "dim",
                          duration:
                            dur === null ? undefined : durationToTicks(dur),
                        })
                      }
                    />
                  </FormControl>
                  {/* Validation message used in a hacky way */}
                  <FormField
                    control={control}
                    name="triggerAction.duration"
                    render={() => <FormMessage />}
                  />
                </FormItem>
              )}
              {value?.tag === "message" && (
                <FormItem>
                  <FormLabel>Message Content</FormLabel>
                  <FormControl>
                    <Input
                      value={value.content ?? ""}
                      onChange={(e) =>
                        onChange({
                          tag: "message",
                          content: e.target.value,
                        })
                      }
                      placeholder="Enter the message to display..."
                    />
                  </FormControl>
                  {/* Validation message used in a hacky way */}
                  <FormField
                    control={control}
                    name="triggerAction.content"
                    render={() => <FormMessage />}
                  />
                </FormItem>
              )}
            </>
          )}
        />

        {/* Action Videos Preview */}
        <ActionsVideos setTriggerAction={setTriggerAction} />
      </CardContent>
    </Card>
  );
}

function RemindersCard() {
  const { fields, append, remove, update } = useFieldArray<
    FormValues,
    "reminders"
  >({
    name: "reminders",
  });

  const reminders = useWatch<FormValues, "reminders">({ name: "reminders" });

  const handleReminderUpdate = useCallback(
    (index: number, threshold: number) => {
      update(index, { threshold, message: reminders[index].message });
    },
    [update, reminders],
  );

  // Sort fields by threshold for display, but preserve original indices
  const sortedFields = useMemo(() => {
    return fields
      .map((field, index) => ({
        field,
        index,
        threshold: reminders[index]?.threshold ?? 0,
      }))
      .sort((a, b) => a.threshold - b.threshold);
  }, [fields, reminders]);

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <CardTitle>Reminders</CardTitle>
            <CardDescription>
              Get notified before reaching your limit
            </CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            type="button"
            className="shrink-0"
            onClick={() => append({ threshold: 0.5, message: "" })}
          >
            <PlusIcon className="size-4 mr-1" />
            Add
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Reminders List - sorted by threshold */}
        {sortedFields.length > 0 && (
          <div className="space-y-3">
            {sortedFields.map(({ field, index }, displayIndex) => (
              <ReminderItem
                key={field.id}
                index={index}
                displayIndex={displayIndex}
                remove={remove}
              />
            ))}
          </div>
        )}

        {/* Reminder Preview */}
        <ReminderPreview
          onReminderAdd={(v) => append({ ...v })}
          onReminderUpdate={handleReminderUpdate}
        />
      </CardContent>
    </Card>
  );
}

function ReminderItem({
  index,
  displayIndex,
  remove,
}: {
  index: number;
  displayIndex: number;
  remove: (index: number) => void;
}) {
  const { control } = useFormContext<FormValues>();
  const target = useWatch<FormValues, "target">({ name: "target" });
  const usageLimit = useWatch<FormValues, "usageLimit">({ name: "usageLimit" });
  const timeFrame = useWatch<FormValues, "timeFrame">({ name: "timeFrame" });
  const reminders = useWatch<FormValues, "reminders">({ name: "reminders" });

  const triggerInfo = useTriggerInfo(target, usageLimit, timeFrame, reminders);
  const willTrigger = triggerInfo.reminders[index]?.trigger;

  return (
    <div
      className={cn(
        "flex flex-col gap-3 p-3 rounded-lg border",
        willTrigger
          ? "border-amber-500/50 bg-amber-500/5"
          : "border-border bg-muted/30",
      )}
    >
      <div className="flex items-center gap-2">
        <Badge variant="outline" className="shrink-0">
          {displayIndex + 1}
        </Badge>
        <FormField
          control={control}
          name={`reminders.${index}.threshold`}
          render={({ field }) => (
            <FormItem className="flex-none">
              <FormControl>
                <div className="flex items-center gap-1">
                  <Input
                    type="number"
                    min={0}
                    max={100}
                    step="any"
                    placeholder="%"
                    className="w-16 text-center"
                    value={Math.round((field.value || 0) * 100)}
                    onChange={(e) =>
                      field.onChange(parseFloat(e.target.value) / 100)
                    }
                  />
                  <span className="text-muted-foreground text-sm">%</span>
                </div>
              </FormControl>
            </FormItem>
          )}
        />
        <FormField
          control={control}
          name={`reminders.${index}.message`}
          render={({ field }) => (
            <FormItem className="flex-1">
              <FormControl>
                <Input placeholder="Reminder message..." {...field} />
              </FormControl>
            </FormItem>
          )}
        />
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="shrink-0 text-muted-foreground hover:text-destructive"
          onClick={() => remove(index)}
        >
          <XIcon className="size-4" />
        </Button>
      </div>

      {/* Validation messages */}
      <div className="flex flex-col gap-1 empty:hidden">
        <FormField
          control={control}
          name={`reminders.${index}.threshold`}
          render={() => <FormMessage />}
        />
        <FormField
          control={control}
          name={`reminders.${index}.message`}
          render={() => <FormMessage />}
        />
      </div>
    </div>
  );
}

function SubmitSection({ submitButtonText }: { submitButtonText: string }) {
  const { control } = useFormContext<FormValues>();
  const target = useWatch<FormValues, "target">({ name: "target" });
  const usageLimit = useWatch<FormValues, "usageLimit">({ name: "usageLimit" });
  const timeFrame = useWatch<FormValues, "timeFrame">({ name: "timeFrame" });
  const reminders = useWatch<FormValues, "reminders">({ name: "reminders" });

  const triggerInfo = useTriggerInfo(target, usageLimit, timeFrame, reminders);

  const timeFrameText = useMemo(() => {
    switch (timeFrame) {
      case "daily":
        return "today";
      case "weekly":
        return "this week";
      case "monthly":
        return "this month";
    }
  }, [timeFrame]);

  const hasWarning =
    triggerInfo.alert || triggerInfo.reminders.some((r) => r.trigger);

  return (
    <Card>
      <CardContent>
        <div className="flex flex-col gap-4">
          {hasWarning && (
            <Alert>
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
                  control={control}
                  name="ignoreTrigger"
                  render={({ field: { value, onChange, ...field } }) => (
                    <FormItem className="mt-3 flex gap-2 items-center">
                      <FormControl>
                        <Checkbox
                          checked={value}
                          onCheckedChange={onChange}
                          {...field}
                        />
                      </FormControl>
                      <FormLabel>Ignore for {timeFrameText}</FormLabel>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </AlertDescription>
            </Alert>
          )}

          <Button type="submit" className="w-full" size="lg">
            {submitButtonText}
          </Button>
        </div>
      </CardContent>
    </Card>
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
    case "month":
      return 24 * 30;
    case "year":
      return 24 * 365;
  }
};

const UsagePreview = memo(() => {
  const target = useWatch<FormValues, "target">({ name: "target" });
  const usageLimit = useWatch<FormValues, "usageLimit">({ name: "usageLimit" });
  const timeFrame = useWatch<FormValues, "timeFrame">({ name: "timeFrame" });

  const [period, setPeriod] = useState<Period>("day");

  const { interval, setInterval, canGoNext, goNext, canGoPrev, goPrev } =
    useIntervalControlsWithDefault("week");

  const scaledUsageLimit = useMemo(() => {
    if (!usageLimit || !timeFrame) return undefined;

    const perHourUsageLimit =
      usageLimit / hoursInPeriod(timeFrameToPeriod(timeFrame));
    return perHourUsageLimit * hoursInPeriod(period);
  }, [period, usageLimit, timeFrame]);

  const { ret: appDurationsPerPeriod } = useAppDurationsPerPeriod({
    ...interval,
    period,
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
  }, [period]);

  const targetApps = useTargetApps(target);
  const highlightedApps = useMemo(() => {
    return Object.fromEntries(targetApps?.map((app) => [app.id, true]) ?? []);
  }, [targetApps]);

  const usageChart = useMemo(() => {
    return (
      <UsageChart
        appDurationsPerPeriod={appDurationsPerPeriod}
        highlightedApps={highlightedApps}
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
        period={period}
        start={interval.start}
        end={interval.end}
        className="w-full h-full"
        maxYIsPeriod={maxYIsPeriod}
        yAxisInterval={yAxisInterval}
        animationsEnabled={false}
        barRadius={3}
      />
    );
  }, [
    appDurationsPerPeriod,
    highlightedApps,
    scaledUsageLimit,
    period,
    interval.start,
    interval.end,
    maxYIsPeriod,
    yAxisInterval,
  ]);

  return (
    <div className="rounded-lg border border-border bg-muted/20 p-4 space-y-4">
      <div className="flex items-center justify-between gap-2 flex-wrap">
        <span className="text-sm font-medium text-muted-foreground">
          Usage Preview
        </span>
        <div className="flex items-center gap-2">
          <PeriodPicker period={period} setPeriod={setPeriod} />
          <div className="flex items-center">
            <Button
              variant="ghost"
              size="icon"
              type="button"
              disabled={!canGoPrev}
              onClick={goPrev}
              className="h-8 w-8"
            >
              <ChevronLeftIcon className="size-4" />
            </Button>
            <DateRangePicker
              value={interval}
              onChange={setInterval}
              dayGranularity={true}
              className="w-auto min-w-28"
            />
            <Button
              variant="ghost"
              size="icon"
              type="button"
              disabled={!canGoNext}
              onClick={goNext}
              className="h-8 w-8"
            >
              <ChevronRightIcon className="size-4" />
            </Button>
          </div>
        </div>
      </div>
      <div className="aspect-[2/1] min-h-[200px]">{usageChart}</div>
    </div>
  );
});

function ReminderPreview({
  onReminderAdd,
  onReminderUpdate,
}: {
  onReminderAdd?: (params: { threshold: number; message: string }) => void;
  onReminderUpdate?: (index: number, threshold: number) => void;
}) {
  const target = useWatch<FormValues, "target">({ name: "target" });
  const usageLimit = useWatch<FormValues, "usageLimit">({ name: "usageLimit" });
  const timeFrame = useWatch<FormValues, "timeFrame">({ name: "timeFrame" });
  const reminders = useWatch<FormValues, "reminders">({ name: "reminders" });

  const triggerInfo = useTriggerInfo(target, usageLimit, timeFrame, reminders);

  if (!usageLimit || !timeFrame || !target) {
    return (
      <div className="rounded-lg border border-dashed border-border p-6 text-center">
        <InfoIcon className="size-8 mx-auto mb-2 text-muted-foreground/50" />
        <p className="text-sm text-muted-foreground">
          Select target, period and usage limit to show reminder preview
        </p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-border bg-muted/20 p-4 space-y-4">
      {/* Current Usage Stats */}
      <div className="grid grid-cols-2 gap-4">
        <div className="flex flex-col">
          <span className="text-xs font-medium text-muted-foreground">
            Current Usage
          </span>
          <div className="flex items-baseline gap-2">
            <DurationText
              className="text-lg font-semibold"
              ticks={triggerInfo.currentUsage}
            />
            {triggerInfo.currentUsage !== 0 && (
              <span
                className={cn(
                  "text-sm tabular-nums",
                  triggerInfo.currentUsage / usageLimit >= 1
                    ? "text-destructive"
                    : "text-muted-foreground",
                )}
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
          <span className="text-xs font-medium text-muted-foreground">
            Usage Limit
          </span>
          <DurationText className="text-lg font-semibold" ticks={usageLimit} />
        </div>
      </div>

      {/* Interactive Progress Bar */}
      <TimeProgressBar
        usageLimit={usageLimit}
        currentUsage={triggerInfo.currentUsage}
        reminders={reminders}
        onReminderAdd={onReminderAdd}
        onReminderUpdate={onReminderUpdate}
      />
    </div>
  );
}

export function TimeProgressBar({
  usageLimit,
  currentUsage,
  reminders,
  onReminderAdd,
  onReminderUpdate,
}: {
  usageLimit: number;
  currentUsage: number;
  reminders: { id?: number; threshold: number; message: string }[];
  onReminderAdd?: (params: { threshold: number; message: string }) => void;
  onReminderUpdate?: (index: number, threshold: number) => void;
}) {
  const percentage = (currentUsage / usageLimit) * 100;
  const progressBarRef = useRef<HTMLDivElement>(null);

  const [hoverState, setHoverState] = useState<{ x: number; visible: boolean }>(
    { x: 0, visible: false },
  );
  const [dragState, setDragState] = useState<{ index: number | null }>({
    index: null,
  });
  const [hoveredTooltip, setHoveredTooltip] = useState<number | null>(null);
  const preventNextClick = useRef(false);

  const getThresholdFromEvent = useCallback((e: React.MouseEvent) => {
    if (!progressBarRef.current) return 0;
    const rect = progressBarRef.current.getBoundingClientRect();
    return Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
  }, []);

  const getPercentageFromEvent = useCallback((e: React.MouseEvent) => {
    if (!progressBarRef.current) return 0;
    const rect = progressBarRef.current.getBoundingClientRect();
    return Math.max(
      0,
      Math.min(100, ((e.clientX - rect.left) / rect.width) * 100),
    );
  }, []);

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (dragState.index !== null) {
        if (onReminderUpdate) {
          onReminderUpdate(dragState.index, getThresholdFromEvent(e));
        }
      } else {
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

  useEffect(() => {
    if (dragState.index !== null) {
      const handleGlobalMouseUp = () => handleMouseUp();
      document.addEventListener("mouseup", handleGlobalMouseUp);
      return () => document.removeEventListener("mouseup", handleGlobalMouseUp);
    }
  }, [dragState.index, handleMouseUp]);

  const isInteractive = onReminderAdd ?? onReminderUpdate;

  // Calculate z-index based on threshold (higher threshold = higher z-index)
  const getZIndex = useCallback(
    (threshold: number, index: number) => {
      const baseZ = Math.round(threshold * 100) + 10;
      // Hovered or dragged tooltip gets max z-index
      if (hoveredTooltip === index || dragState.index === index) {
        return 200;
      }
      return baseZ;
    },
    [hoveredTooltip, dragState.index],
  );

  return (
    <div
      ref={progressBarRef}
      className={cn("relative w-full bg-secondary h-3 rounded-full my-6")}
      onMouseMove={isInteractive ? handleMouseMove : undefined}
      onMouseLeave={isInteractive ? handleMouseLeave : undefined}
      onClick={isInteractive ? handleClick : undefined}
      onMouseUp={isInteractive ? handleMouseUp : undefined}
    >
      {/* Progress bar */}
      <div
        className={cn(
          "h-full transition-all",
          percentage >= 100 ? "bg-destructive/80" : "bg-primary/80",
        )}
        style={{ width: `${Math.min(100, percentage)}%` }}
      />

      {/* Hover indicator */}
      {hoverState.visible &&
        dragState.index === null &&
        !preventNextClick.current &&
        isInteractive && (
          <div
            className="absolute top-1/2 -translate-y-1/2 w-0.5 h-5 bg-primary/50 rounded-full"
            style={{ left: `${hoverState.x}%` }}
          >
            <div className="absolute -top-1 left-1/2 -translate-x-1/2">
              <PlusIcon className="size-3 text-primary" />
            </div>
          </div>
        )}

      {/* Reminder markers */}
      {reminders.map((reminder, index) => {
        const isDragging = dragState.index === index;
        const isHovered = hoveredTooltip === index;
        const zIndex = getZIndex(reminder.threshold, index);

        return (
          <div
            key={index}
            className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2"
            style={{
              left: `${reminder.threshold * 100}%`,
              zIndex,
            }}
            onMouseEnter={() => setHoveredTooltip(index)}
            onMouseLeave={() => setHoveredTooltip(null)}
          >
            {/* Thin vertical marker */}
            <div
              className={cn(
                "w-1 h-5 rounded-full transition-all",
                isDragging
                  ? "bg-primary scale-x-150 shadow-lg"
                  : isHovered
                    ? "bg-primary scale-x-125"
                    : "bg-primary/80 hover:bg-primary",
              )}
              onMouseDown={(e) => handleMouseDown(e, index)}
            />

            {/* Tooltip */}
            <div
              className={cn(
                "absolute left-1/2 -translate-x-1/2 top-full mt-1.5 transition-opacity",
                isDragging || isHovered ? "opacity-100" : "opacity-80",
              )}
            >
              <div className="relative bg-popover text-popover-foreground border border-border rounded-md shadow-lg min-w-[100px] max-w-[160px]">
                {/* Arrow */}
                <div className="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-popover border-l border-t border-border rotate-45 rounded-tl-sm" />
                {/* Content */}
                <div className="relative p-2 space-y-0.5">
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-xs font-semibold tabular-nums">
                      {((reminder.threshold || 0) * 100).toFixed(0)}%
                    </span>
                    <span className="text-xs text-muted-foreground">
                      <DurationText
                        ticks={(reminder.threshold || 0) * usageLimit}
                      />
                    </span>
                  </div>
                  {reminder.message && (
                    <p className="text-xs line-clamp-2 break-words text-muted-foreground">
                      {reminder.message}
                    </p>
                  )}
                </div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

const ActionsVideos = memo(
  ({
    setTriggerAction,
  }: {
    setTriggerAction: (action: TriggerAction) => void;
  }) => {
    const [api, setApi] = useState<CarouselApi>();
    const [current, setCurrent] = useState(0);
    const [showControls, setShowControls] = useState(false);
    const [currentMessages, setCurrentMessages] = useState<string[]>([]);
    const videoRefs = useRef<(HTMLVideoElement | null)[]>([]);

    useEffect(() => {
      if (!api) return;

      api.on("select", () => {
        setCurrent(api.selectedScrollSnap());
      });
    }, [api]);

    const handleVideoEnd = useCallback(() => {
      if (api) {
        const nextIndex = (current + 1) % actionTypes.length;
        api.scrollTo(nextIndex);
      }
    }, [api, current]);

    const handleTimeUpdate = useCallback(
      (event: React.SyntheticEvent<HTMLVideoElement>) => {
        const video = event.currentTarget;
        const currentTime = video.currentTime;
        const videoIndex = parseInt(video.dataset.videoIndex ?? "0");

        if (videoIndex === current) {
          const actionData = actionTypes[videoIndex];
          const activeMessages = actionData.videoMessages
            .filter((msg) => currentTime >= msg.start && currentTime < msg.end)
            .map((msg) => msg.message);

          setCurrentMessages(activeMessages);
        }
      },
      [current],
    );

    useEffect(() => {
      videoRefs.current.forEach((videoRef, index) => {
        if (videoRef) {
          if (index === current) {
            videoRef.play().catch(console.error);
          } else {
            videoRef.pause();
            videoRef.currentTime = 0;
          }
        }
      });

      setCurrentMessages([]);
    }, [current]);

    return (
      <div className="rounded-lg border border-border bg-muted/20 p-4 space-y-4">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium text-muted-foreground">
            Action Preview
          </span>
          <Button
            variant="outline"
            size="sm"
            type="button"
            className="text-xs gap-1.5"
            onClick={() => setTriggerAction(actionTypes[current].defaultAction)}
          >
            <PointerIcon className="size-3" />
            Use {actionTypes[current].title}
          </Button>
        </div>

        <Carousel
          setApi={setApi}
          className="w-full"
          opts={{
            align: "start",
            loop: true,
          }}
        >
          <CarouselContent>
            {actionTypes.map((action, index) => (
              <CarouselItem key={index}>
                <div
                  className="aspect-video rounded-lg overflow-hidden relative bg-black"
                  onMouseEnter={() => setShowControls(true)}
                  onMouseLeave={() => setShowControls(false)}
                >
                  <video
                    ref={(el) => {
                      videoRefs.current[index] = el;
                    }}
                    data-video-index={index}
                    className="w-full object-fill aspect-video"
                    autoPlay={index === current}
                    muted
                    loop={false}
                    onEnded={handleVideoEnd}
                    onTimeUpdate={handleTimeUpdate}
                    playsInline
                    controls={showControls}
                    controlsList="nodownload nofullscreen noplaybackrate noremoteplayback"
                    disablePictureInPicture
                    disableRemotePlayback
                  >
                    <source src={action.videoSrc} type="video/mp4" />
                    Your browser does not support the video tag.
                  </video>

                  {/* Messages overlay */}
                  {index === current && currentMessages.length > 0 && (
                    <div className="absolute top-3 right-3 z-10 space-y-2">
                      {currentMessages.map((message, msgIndex) => (
                        <div
                          key={msgIndex}
                          className="bg-background/80 backdrop-blur-sm border rounded-md px-2 py-1.5 text-xs text-foreground inline-flex items-center gap-1.5"
                        >
                          <InfoIcon className="size-3.5 shrink-0 text-blue-400" />
                          {message}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </CarouselItem>
            ))}
          </CarouselContent>
          <CarouselPrevious type="button" className="-left-3" />
          <CarouselNext type="button" className="-right-3" />
        </Carousel>

        {/* Navigation dots and info */}
        <div className="flex items-center justify-between">
          <div className="flex gap-1.5">
            {actionTypes.map((action, index) => (
              <button
                key={index}
                type="button"
                onClick={() => api?.scrollTo(index)}
                className={cn(
                  "w-2 h-2 rounded-full transition-colors",
                  index === current
                    ? "bg-primary"
                    : "bg-primary/25 hover:bg-primary/50",
                )}
                aria-label={`Go to ${action.title}`}
              />
            ))}
          </div>
          <div className="text-xs text-muted-foreground">
            <span className="font-medium">{actionTypes[current].title}</span>
            <span className="mx-1">—</span>
            <span>{actionTypes[current].description}</span>
          </div>
        </div>
      </div>
    );
  },
);
