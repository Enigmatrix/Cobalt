import {
  useState,
  useMemo,
  useCallback,
  type ComponentProps,
  useRef,
} from "react";
import {
  ChevronRight,
  ChevronDown,
  LaptopIcon,
  Loader2Icon,
  KeyboardIcon,
  MouseIcon,
} from "lucide-react";
import { DateTime } from "luxon";
import {
  systemEventToString,
  type App,
  type InteractionPeriod,
  type Ref,
  type Session,
  type SystemEvent,
  type Usage,
} from "@/lib/entities";
import { ticksToDateTime } from "@/lib/time";
import type { AppSessionUsages } from "@/lib/repo";
import { Text } from "@/components/ui/text";
import { useApps } from "@/hooks/use-refresh";
import AppIcon from "@/components/app/app-icon";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";
import { HScrollView } from "@/components/hscroll-view";
import { Tooltip } from "@/components/viz/tooltip";
import { DateTimeText } from "@/components/time/time-text";
import { DurationText } from "@/components/time/duration-text";

interface HoverData {
  session?: Session;
  usage?: Usage;
}

interface GanttProps {
  usages: AppSessionUsages;
  usagesLoading?: boolean;
  interactionPeriods?: InteractionPeriod[];
  interactionPeriodsLoading?: boolean;
  systemEvents?: SystemEvent[];
  systemEventsLoading?: boolean;
  defaultExpanded?: Record<Ref<App>, boolean>;
  rangeStart: DateTime;
  rangeEnd: DateTime;
}

type TimeUnit = {
  unit: "minute" | "hour" | "day";
  step: number;
};

type SessionUsage = {
  usage: Usage;
  session: Session;
};

function getTimeUnits(start: DateTime, end: DateTime): TimeUnit {
  const diff = end.diff(start, ["days", "hours", "minutes"]).toObject();

  if (diff.days && diff.days > 2) {
    return { unit: "day", step: 1 };
  } else if (diff.hours && diff.hours > 4) {
    return { unit: "hour", step: 1 };
  }
  return { unit: "minute", step: 15 };
}

function getTimeArray(
  start: DateTime,
  end: DateTime,
  unit: "minute" | "hour" | "day",
  step: number,
): DateTime[] {
  const times: DateTime[] = [];
  let current = start;

  while (current < end) {
    times.push(current);
    current = current.plus({ [unit + "s"]: step });
  }

  return times;
}

function formatTime(date: DateTime, unit: "minute" | "hour" | "day"): string {
  switch (unit) {
    case "minute":
      return date.toLocaleString({ hour: "2-digit", minute: "2-digit" });
    case "hour":
      return date.toLocaleString({ hour: "2-digit" });
    case "day":
      return date.toLocaleString({ month: "short", day: "numeric" });
  }
}

function getPosition(
  start: DateTime,
  end: DateTime,
  rangeStart: DateTime,
  rangeEnd: DateTime,
  timeUnit: TimeUnit,
) {
  const totalDuration = rangeEnd
    .diff(rangeStart, timeUnit.unit)
    .get(timeUnit.unit);
  const startOffset = start.diff(rangeStart, timeUnit.unit).get(timeUnit.unit);
  const duration = end.diff(start, timeUnit.unit).get(timeUnit.unit);

  const left = (startOffset / totalDuration) * 100;
  const width = (duration / totalDuration) * 100;

  return { left: `${left}%`, width: `${width}%` };
}

function Bar({
  className,
  rangeStart,
  rangeEnd,
  start,
  end,
  timeUnit,
  ...props
}: ComponentProps<"div"> & {
  className?: ClassValue;
  rangeStart: DateTime;
  rangeEnd: DateTime;
  start: DateTime;
  end: DateTime;
  timeUnit: TimeUnit;
}) {
  return (
    <div
      className={cn("absolute h-6", className)}
      style={getPosition(start, end, rangeStart, rangeEnd, timeUnit)}
      {...props}
    />
  );
}

function Line({
  className,
  rangeStart,
  rangeEnd,
  timestamp,
  timeUnit,
  ...props
}: ComponentProps<"div"> & {
  className?: ClassValue;
  rangeStart: DateTime;
  rangeEnd: DateTime;
  timestamp: DateTime;
  timeUnit: TimeUnit;
}) {
  return (
    <div
      className={cn("absolute h-8 -mt-1", className)}
      style={{
        ...getPosition(timestamp, timestamp, rangeStart, rangeEnd, timeUnit),
        width: "1px",
      }}
      {...props}
    />
  );
}

function UsageBars({
  sessionUsages,
  rangeStart,
  rangeEnd,
  timeUnit,
  setHoverUsage,
}: {
  sessionUsages: SessionUsage[];
  rangeStart: DateTime;
  rangeEnd: DateTime;
  timeUnit: TimeUnit;
  setHoverUsage: (usage: HoverData | null) => void;
}) {
  return (
    <>
      {sessionUsages.map((sessionUsage, index) => (
        <Bar
          key={index}
          className="bg-primary hover:bg-card-foreground"
          start={ticksToDateTime(sessionUsage.usage.start)}
          end={ticksToDateTime(sessionUsage.usage.end)}
          rangeStart={rangeStart}
          rangeEnd={rangeEnd}
          timeUnit={timeUnit}
          onMouseOver={() => setHoverUsage(sessionUsage)}
          onMouseLeave={() => setHoverUsage(null)}
        />
      ))}
    </>
  );
}

function InteractionPeriodBars({
  interactionPeriods,
  systemEvents,
  rangeStart,
  rangeEnd,
  timeUnit,
  setHoverInteractionPeriod,
  setHoverSystemEvent,
}: {
  interactionPeriods: InteractionPeriod[];
  systemEvents: SystemEvent[];
  rangeStart: DateTime;
  rangeEnd: DateTime;
  timeUnit: TimeUnit;
  setHoverInteractionPeriod: (ip: InteractionPeriod | null) => void;
  setHoverSystemEvent: (se: SystemEvent | null) => void;
}) {
  return (
    <>
      {interactionPeriods.map((interactionPeriod, index) => (
        <Bar
          key={index}
          className="bg-primary/50 hover:bg-card-foreground"
          start={ticksToDateTime(interactionPeriod.start)}
          end={ticksToDateTime(interactionPeriod.end)}
          rangeStart={rangeStart}
          rangeEnd={rangeEnd}
          timeUnit={timeUnit}
          onMouseOver={() => setHoverInteractionPeriod(interactionPeriod)}
          onMouseLeave={() => setHoverInteractionPeriod(null)}
        />
      ))}
      {systemEvents.map((systemEvent, index) => (
        <Line
          key={index}
          className="bg-orange-500 hover:bg-card-foreground"
          timestamp={ticksToDateTime(systemEvent.timestamp)}
          rangeStart={rangeStart}
          rangeEnd={rangeEnd}
          timeUnit={timeUnit}
          onMouseOver={() => setHoverSystemEvent(systemEvent)}
          onMouseLeave={() => setHoverSystemEvent(null)}
        />
      ))}
    </>
  );
}

function AppBars({
  app,
  expanded,
  usages,
  rangeStart,
  rangeEnd,
  timeUnit,
  setHoverUsage,
}: {
  app: App;
  expanded: boolean;
  usages: AppSessionUsages;
  rangeStart: DateTime;
  rangeEnd: DateTime;
  timeUnit: TimeUnit;
  setHoverUsage: (usage: HoverData | null) => void;
}) {
  const allSessionUsagesFlat = useMemo(
    () =>
      Object.values(usages[app.id]).flatMap((session) =>
        session.usages.map((usage) => ({ usage, session })),
      ),
    [app, usages],
  );
  const allSessionUsages = useMemo(
    () =>
      Object.values(usages[app.id]).map((session) => ({
        session,
        sessionUsages: session.usages.map((usage) => ({ usage, session })),
      })),
    [app, usages],
  );

  return (
    <div className="border-b">
      {/* App header */}
      <div className="h-[52px] bg-muted/60 relative">
        <div className="absolute inset-x-0 top-4">
          <UsageBars
            sessionUsages={allSessionUsagesFlat}
            rangeStart={rangeStart}
            rangeEnd={rangeEnd}
            timeUnit={timeUnit}
            setHoverUsage={setHoverUsage}
          />
        </div>
      </div>

      {/* Sessions */}
      {expanded &&
        allSessionUsages.map(({ session, sessionUsages }) => (
          <div key={session.id} className="relative h-[68px] border-t">
            <div className="absolute inset-x-0 top-6">
              {/* Base task bar */}
              <Bar
                className="bg-primary/20"
                start={ticksToDateTime(session.start)}
                end={ticksToDateTime(session.end)}
                rangeStart={rangeStart}
                rangeEnd={rangeEnd}
                timeUnit={timeUnit}
                onMouseOver={() => setHoverUsage({ session })}
                onMouseLeave={() => setHoverUsage(null)}
              />
              {/* Usage periods */}
              <UsageBars
                sessionUsages={sessionUsages}
                rangeStart={rangeStart}
                rangeEnd={rangeEnd}
                timeUnit={timeUnit}
                setHoverUsage={setHoverUsage}
              />
            </div>
          </div>
        ))}
    </div>
  );
}

export function Gantt({
  usages,
  usagesLoading,
  interactionPeriods,
  interactionPeriodsLoading,
  systemEvents,
  systemEventsLoading,
  defaultExpanded,
  rangeStart,
  rangeEnd,
}: GanttProps) {
  const [expanded, setExpanded] = useState<Record<Ref<App>, boolean>>(
    defaultExpanded ?? {},
  );
  const [hoverUsage, setHoverUsage] = useState<HoverData | null>(null);
  const [hoverInteractionPeriod, setHoverInteractionPeriod] =
    useState<InteractionPeriod | null>(null);
  const [hoverSystemEvent, setHoverSystemEvent] = useState<SystemEvent | null>(
    null,
  );

  const timeUnit = useMemo(
    () => getTimeUnits(rangeStart, rangeEnd),
    [rangeStart, rangeEnd],
  );
  const timeArray = useMemo(
    () => getTimeArray(rangeStart, rangeEnd, timeUnit.unit, timeUnit.step),
    [rangeStart, rangeEnd, timeUnit],
  );

  const involvedApps = useMemo(
    () => Object.keys(usages).map((id) => +id as Ref<App>),
    [usages],
  );
  const apps = useApps(involvedApps);

  const toggleApp = useCallback(
    (appId: Ref<App>) => {
      setExpanded((prev) => ({
        ...prev,
        [appId]: !prev[appId],
      }));
    },
    [setExpanded],
  );

  const hideTimeline =
    (apps.length === 0 || usagesLoading) &&
    interactionPeriods === undefined &&
    systemEvents === undefined;

  const ref = useRef<HTMLDivElement | null>(null);

  return (
    <div className="flex flex-col" ref={ref}>
      <Tooltip show={hoverUsage !== null} targetRef={ref}>
        <div className="max-w-[800px]">
          {hoverUsage?.session && (
            <div className="flex flex-col">
              <Text>{hoverUsage.session.title}</Text>
              <div className="flex items-center text-muted-foreground gap-1 text-xs">
                <DateTimeText ticks={hoverUsage.session.start} /> -
                <DateTimeText ticks={hoverUsage.session.end} />
                (
                <DurationText
                  ticks={hoverUsage.session.end - hoverUsage.session.start}
                />
                )
              </div>
            </div>
          )}
          {hoverUsage?.usage && (
            <div
              className={cn("flex flex-col", {
                "border-t mt-2 border-border": hoverUsage.session,
              })}
            >
              <Text className="text-muted-foreground">Usage</Text>
              <div className="flex items-center text-muted-foreground gap-1 text-xs">
                <DateTimeText ticks={hoverUsage.usage.start} /> -
                <DateTimeText ticks={hoverUsage.usage.end} />
                (
                <DurationText
                  ticks={hoverUsage.usage.end - hoverUsage.usage.start}
                />
                )
              </div>
            </div>
          )}
        </div>
      </Tooltip>

      <Tooltip show={hoverInteractionPeriod !== null} targetRef={ref}>
        <div className="max-w-[800px]">
          {hoverInteractionPeriod && (
            <div className={cn("flex flex-col")}>
              <div className="flex items-center gap-1 text-sm">
                <Text className="text-base">Interaction</Text>
                <KeyboardIcon size={16} className="ml-4" />
                <div>{hoverInteractionPeriod.key_strokes}</div>
                <MouseIcon size={16} className="ml-2" />
                <div>{hoverInteractionPeriod.mouse_clicks}</div>
              </div>
              <div className="flex items-center text-muted-foreground gap-1 text-xs">
                <DateTimeText ticks={hoverInteractionPeriod.start} /> -
                <DateTimeText ticks={hoverInteractionPeriod.end} />
                (
                <DurationText
                  ticks={
                    hoverInteractionPeriod.end - hoverInteractionPeriod.start
                  }
                />
                )
              </div>
            </div>
          )}
        </div>
      </Tooltip>

      <Tooltip show={hoverSystemEvent !== null} targetRef={ref}>
        <div className="max-w-[800px]">
          {hoverSystemEvent && (
            <div className={cn("flex flex-col")}>
              <div className="flex items-center gap-1 text-sm">
                <Text className="text-base">
                  {systemEventToString(hoverSystemEvent.event)}
                </Text>
              </div>
              <div className="flex items-center text-muted-foreground gap-1 text-xs">
                <DateTimeText ticks={hoverSystemEvent.timestamp} />
              </div>
            </div>
          )}
        </div>
      </Tooltip>

      <div className="bg-transparent overflow-hidden flex text-muted-foreground">
        {/* Fixed left column */}
        <div className="w-[300px] flex-shrink-0">
          <div
            className={cn("h-14 border-r border-r-transparent flex", {
              "border-b": !hideTimeline,
            })}
          >
            <h2 className="font-semibold text-xl text-card-foreground my-auto mx-4">
              Sessions
            </h2>
          </div>

          {/* Interaction Periods' + System Events header */}
          {(interactionPeriods || systemEvents) && (
            <div className="border-b">
              <div className="flex items-center p-4 border-r h-[52px]">
                <LaptopIcon className="w-6 h-6 ml-6" />
                <Text className="font-semibold ml-4">Interactions</Text>
                {/* BUG: this loading is as long as the usage loading ..????? */}
                {(interactionPeriodsLoading || systemEventsLoading) && (
                  <Loader2Icon className="animate-spin ml-4" />
                )}
              </div>
            </div>
          )}

          {/* App headers and sessions */}
          {!usagesLoading &&
            apps.map((app) => (
              <div key={app.id} className="border-b">
                <div
                  className="flex items-center p-4 bg-muted/80 cursor-pointer hover:bg-muted/60 border-r h-[52px]"
                  onClick={() => toggleApp(app.id)}
                >
                  {expanded[app.id] ? (
                    <ChevronDown size={20} />
                  ) : (
                    <ChevronRight size={20} />
                  )}
                  <AppIcon
                    buffer={app.icon}
                    className="ml-2 w-6 h-6 shrink-0"
                  />
                  <Text className="font-semibold ml-4">{app.name}</Text>
                </div>

                {expanded[app.id] &&
                  Object.values(usages[app.id]).map((session) => (
                    <div
                      key={session.id}
                      className="p-4 border-t border-r h-[68px]"
                    >
                      <Text className="text-sm">{session.title}</Text>
                      <div className="text-xs text-muted-foreground">
                        {formatTime(
                          ticksToDateTime(session.start),
                          timeUnit.unit,
                        )}{" "}
                        -{" "}
                        {formatTime(
                          ticksToDateTime(session.end),
                          timeUnit.unit,
                        )}
                      </div>
                    </div>
                  ))}
              </div>
            ))}
        </div>

        {/* Scrollable timeline container */}
        <HScrollView
          className="flex-grow overflow-auto"
          innerClassName="group relative"
          onMouseMove={(e) => {
            const container = e.currentTarget;
            if (container) {
              const rect = container.getBoundingClientRect();
              const x = e.clientX - rect.left + container.scrollLeft;
              container.style.setProperty("--mouse-x", `${x}px`);
            }
          }}
        >
          {/* Vertical line that follows mouse */}
          <div
            className="absolute top-0 bottom-0 w-px border-l border-dashed border-muted-foreground z-[999] pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity"
            style={{
              left: "var(--mouse-x)",
              transform: "translateX(-50%)",
            }}
          />
          <div className="flex flex-col w-fit">
            {/* Timeline header */}
            {!hideTimeline && (
              <div className="h-14 w-fit flex py-4 border-b">
                {timeArray.map(
                  (time, i) =>
                    i % (timeUnit.unit === "minute" ? 4 : 1) === 0 && (
                      <div
                        key={time.toISO()}
                        className="text-sm text-muted-foreground flex-shrink-0 border-l pl-1 border-muted-foreground"
                        style={{ width: "100px" }}
                      >
                        {formatTime(time, timeUnit.unit)}
                      </div>
                    ),
                )}
              </div>
            )}

            {(interactionPeriods || systemEvents) && (
              <div className="border-b">
                {/* Interaction Periods */}
                <div className="h-[52px] relative">
                  <div className="absolute inset-x-0 top-4">
                    <InteractionPeriodBars
                      setHoverInteractionPeriod={setHoverInteractionPeriod}
                      setHoverSystemEvent={setHoverSystemEvent}
                      interactionPeriods={interactionPeriods ?? []}
                      systemEvents={systemEvents ?? []}
                      rangeStart={rangeStart}
                      rangeEnd={rangeEnd}
                      timeUnit={timeUnit}
                    />
                  </div>
                </div>
              </div>
            )}

            {/* Task bars */}
            {!usagesLoading &&
              apps.map((app) => (
                <AppBars
                  key={app.id}
                  app={app}
                  expanded={expanded[app.id]}
                  usages={usages}
                  rangeStart={rangeStart}
                  rangeEnd={rangeEnd}
                  timeUnit={timeUnit}
                  setHoverUsage={setHoverUsage}
                />
              ))}
          </div>
        </HScrollView>
      </div>

      {/* Session Empty State Indicator */}
      {!usagesLoading && apps.length === 0 && (
        <div className="flex flex-col items-center justify-center p-8 text-muted-foreground">
          <LaptopIcon className="w-12 h-12 mb-4" />
          <Text className="text-lg font-semibold mb-2">No Activity</Text>
          <Text className="text-sm">
            No application usage data available for this time period
          </Text>
        </div>
      )}

      {/* Session Loading Indicator */}
      {usagesLoading && (
        <div className="flex flex-col items-center justify-center p-8 text-muted-foreground">
          <LaptopIcon className="w-12 h-12 mb-4 animate-pulse" />
          <Text className="text-lg font-semibold mb-2">Loading...</Text>
          <Text className="text-sm animate-pulse">
            Fetching application usage data
          </Text>
        </div>
      )}
    </div>
  );
}
