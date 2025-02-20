import React, { useState, useMemo } from "react";
import { ChevronRight, ChevronDown } from "lucide-react";
import { DateTime, Duration } from "luxon";
import type { Usage } from "@/lib/entities";
import { ticksToDateTime } from "@/lib/time";

interface Task {
  id: string;
  name: string;
  start: DateTime;
  end: DateTime;
  usages: Usage[];
  category: string;
}

interface GanttProps {
  tasks: Task[];
}

function getTimeUnits(
  start: DateTime,
  end: DateTime,
): { unit: "minute" | "hour" | "day"; step: number } {
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

  while (current <= end) {
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

export function Gantt({ tasks }: GanttProps) {
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});

  // Add padding to timeline
  const projectStart = DateTime.min(...tasks.map((t) => t.start)).minus({
    days: 1,
  });
  const projectEnd = DateTime.max(...tasks.map((t) => t.end)).plus({ days: 1 });

  const timeUnit = useMemo(
    () => getTimeUnits(projectStart, projectEnd),
    [projectStart, projectEnd],
  );
  const timeArray = useMemo(
    () => getTimeArray(projectStart, projectEnd, timeUnit.unit, timeUnit.step),
    [projectStart, projectEnd, timeUnit],
  );

  const categories = Array.from(new Set(tasks.map((t) => t.category)));

  const toggleCategory = (category: string) => {
    setExpanded((prev) => ({
      ...prev,
      [category]: !prev[category],
    }));
  };

  const getPosition = (start: DateTime, end: DateTime) => {
    const totalDuration = projectEnd
      .diff(projectStart, timeUnit.unit)
      .get(timeUnit.unit);
    const startOffset = start
      .diff(projectStart, timeUnit.unit)
      .get(timeUnit.unit);
    const duration = end.diff(start, timeUnit.unit).get(timeUnit.unit);

    const left = (startOffset / totalDuration) * 100;
    const width = (duration / totalDuration) * 100;

    return { left: `${left}%`, width: `${width}%` };
  };

  return (
    <div className="bg-card rounded-lg shadow-lg overflow-hidden">
      <h1 className="text-2xl font-bold p-6 border-b">Project Gantt Chart</h1>

      <div className="flex">
        {/* Fixed left column */}
        <div className="w-[200px] flex-shrink-0">
          <div className="h-14 border-r bg-muted p-4">
            <h2 className="font-semibold">Tasks</h2>
          </div>

          {/* Category headers and tasks */}
          {categories.map((category) => (
            <div key={category} className="border-b">
              <div
                className="flex items-center p-4 bg-muted cursor-pointer hover:bg-muted/80 border-r h-[52px]"
                onClick={() => toggleCategory(category)}
              >
                {expanded[category] ? (
                  <ChevronDown size={20} />
                ) : (
                  <ChevronRight size={20} />
                )}
                <span className="font-semibold ml-2">{category}</span>
              </div>

              {expanded[category] &&
                tasks
                  .filter((task) => task.category === category)
                  .map((task) => (
                    <div
                      key={task.id}
                      className="p-4 border-t border-r h-[68px]"
                    >
                      <div className="text-sm">{task.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {formatTime(task.start, timeUnit.unit)} -{" "}
                        {formatTime(task.end, timeUnit.unit)}
                      </div>
                    </div>
                  ))}
            </div>
          ))}
        </div>

        {/* Scrollable timeline container */}
        <div className="flex-grow overflow-x-auto">
          <div className="min-w-[800px]">
            {/* Timeline header */}
            <div className="h-14 flex p-4 border-b">
              {timeArray.map(
                (time, i) =>
                  i % (timeUnit.unit === "minute" ? 4 : 1) === 0 && (
                    <div
                      key={time.toISO()}
                      className="text-sm text-muted-foreground flex-shrink-0"
                      style={{ width: "100px" }}
                    >
                      {formatTime(time, timeUnit.unit)}
                    </div>
                  ),
              )}
            </div>

            {/* Task bars */}
            {categories.map((category) => (
              <div key={category} className="border-b">
                <div className="h-[52px] bg-muted" />{" "}
                {/* Category header spacer */}
                {expanded[category] &&
                  tasks
                    .filter((task) => task.category === category)
                    .map((task) => (
                      <div key={task.id} className="relative h-[68px] border-t">
                        <div className="absolute inset-x-4 top-1/2 -translate-y-1/2">
                          {/* Base task bar */}
                          <div
                            className="absolute h-6 rounded-full bg-blue-200"
                            style={getPosition(task.start, task.end)}
                          >
                            {/* Usage periods */}
                            {task.usages.map((usage, index) => (
                              <div
                                key={index}
                                className="absolute h-full bg-blue-500 rounded-full"
                                style={getPosition(
                                  ticksToDateTime(usage.start),
                                  ticksToDateTime(usage.end),
                                )}
                              />
                            ))}
                          </div>
                        </div>
                      </div>
                    ))}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
