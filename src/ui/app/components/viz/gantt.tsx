import { useState, useMemo, useCallback } from "react";
import { ChevronRight, ChevronDown, LaptopIcon } from "lucide-react";
import { DateTime } from "luxon";
import type { App, InteractionPeriod, Ref, Usage } from "@/lib/entities";
import { ticksToDateTime } from "@/lib/time";
import type { AppSessionUsages } from "@/lib/repo";
import { Text } from "@/components/ui/text";
import { useApps } from "@/hooks/use-refresh";
import AppIcon from "../app/app-icon";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";

interface GanttProps {
  usages: AppSessionUsages;
  interactionPeriods?: InteractionPeriod[];
  defaultExpanded?: Record<Ref<App>, boolean>;
  rangeStart: DateTime;
  rangeEnd: DateTime;
}

type TimeUnit = {
  unit: "minute" | "hour" | "day";
  step: number;
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
}: {
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
    />
  );
}

function UsageBars({
  usages,
  rangeStart,
  rangeEnd,
  timeUnit,
}: {
  usages: Usage[];
  rangeStart: DateTime;
  rangeEnd: DateTime;
  timeUnit: TimeUnit;
}) {
  return (
    <>
      {usages.map((usage, index) => (
        <Bar
          key={index}
          className="bg-primary hover:bg-card-foreground"
          start={ticksToDateTime(usage.start)}
          end={ticksToDateTime(usage.end)}
          rangeStart={rangeStart}
          rangeEnd={rangeEnd}
          timeUnit={timeUnit}
        />
      ))}
    </>
  );
}

function InteractionPeriodBars({
  interactionPeriods,
  rangeStart,
  rangeEnd,
  timeUnit,
}: {
  interactionPeriods: InteractionPeriod[];
  rangeStart: DateTime;
  rangeEnd: DateTime;
  timeUnit: TimeUnit;
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
}: {
  app: App;
  expanded: boolean;
  usages: AppSessionUsages;
  rangeStart: DateTime;
  rangeEnd: DateTime;
  timeUnit: TimeUnit;
}) {
  const allUsages = useMemo(
    () =>
      Object.values(usages[app.id])
        .map((session) => session.usages)
        .flat(),
    [app, usages],
  );
  const sessions = useMemo(() => Object.values(usages[app.id]), [app, usages]);
  return (
    <div className="border-b">
      {/* App header */}
      <div className="h-[52px] bg-muted/60 relative">
        <div className="absolute inset-x-0 top-4">
          <UsageBars
            usages={allUsages}
            rangeStart={rangeStart}
            rangeEnd={rangeEnd}
            timeUnit={timeUnit}
          />
        </div>
      </div>

      {/* Sessions */}
      {expanded &&
        sessions.map((session) => (
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
              />
              {/* Usage periods */}
              <UsageBars
                usages={session.usages}
                rangeStart={rangeStart}
                rangeEnd={rangeEnd}
                timeUnit={timeUnit}
              />
            </div>
          </div>
        ))}
    </div>
  );
}

export function Gantt({
  usages,
  interactionPeriods,
  defaultExpanded,
  rangeStart,
  rangeEnd,
}: GanttProps) {
  const [expanded, setExpanded] = useState<Record<Ref<App>, boolean>>(
    defaultExpanded ?? {},
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

  return (
    <div className="bg-transparent overflow-hidden flex text-muted-foreground">
      {/* Fixed left column */}
      <div className="w-[300px] flex-shrink-0">
        <div className="h-14 border-r border-b border-r-transparent p-4">
          <h2 className="font-semibold text-xl text-card-foreground">
            Sessions
          </h2>
        </div>

        {/* Interaction Periods' header */}
        {interactionPeriods && (
          <div className="border-b">
            <div className="flex items-center p-4 border-r h-[52px]">
              <LaptopIcon className="w-6 h-6 ml-6" />
              <Text className="font-semibold ml-4">Interactions</Text>
            </div>
          </div>
        )}

        {/* App headers and sessions */}
        {apps.map((app) => (
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
              <AppIcon buffer={app.icon} className="ml-2 w-6 h-6 shrink-0" />
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
                    {formatTime(ticksToDateTime(session.start), timeUnit.unit)}{" "}
                    - {formatTime(ticksToDateTime(session.end), timeUnit.unit)}
                  </div>
                </div>
              ))}
          </div>
        ))}
      </div>

      {/* Scrollable timeline container */}
      <div className="flex-grow overflow-x-auto">
        <div className="flex flex-col w-fit">
          {/* Timeline header */}
          <div className="h-14 w-fit flex py-4 border-b">
            {timeArray.map(
              (time, i) =>
                i % (timeUnit.unit === "minute" ? 4 : 1) === 0 && (
                  <div
                    key={time.toISO()}
                    // NOTE: this border might take up space, making the usage timelines inaccurate.
                    className="text-sm text-muted-foreground flex-shrink-0 border-l pl-1 border-muted-foreground"
                    style={{ width: "100px" }}
                  >
                    {formatTime(time, timeUnit.unit)}
                  </div>
                ),
            )}
          </div>

          {interactionPeriods && (
            <div className="border-b">
              {/* Interaction Periods */}
              <div className="h-[52px] relative">
                <div className="absolute inset-x-0 top-4">
                  <InteractionPeriodBars
                    interactionPeriods={interactionPeriods}
                    rangeStart={rangeStart}
                    rangeEnd={rangeEnd}
                    timeUnit={timeUnit}
                  />
                </div>
              </div>
            </div>
          )}

          {/* Task bars */}
          {apps.map((app) => (
            <AppBars
              key={app.id}
              app={app}
              expanded={expanded[app.id]}
              usages={usages}
              rangeStart={rangeStart}
              rangeEnd={rangeEnd}
              timeUnit={timeUnit}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
