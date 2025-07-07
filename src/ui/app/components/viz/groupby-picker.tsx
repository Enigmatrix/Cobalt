import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { GroupBy } from "@/components/viz/usage-chart";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { useCallback } from "react";

export function GroupByPicker({
  groupBy,
  setGroupBy: setGroupByInner,
  className,
}: {
  groupBy: GroupBy;
  setGroupBy: (groupBy: GroupBy) => void;
  className?: ClassValue;
}) {
  const setGroupBy = useCallback(
    (p: string) => {
      setGroupByInner(p as GroupBy);
    },
    [setGroupByInner],
  );

  return (
    <Select value={groupBy} onValueChange={setGroupBy}>
      <SelectTrigger className={cn("min-w-32 font-medium", className)}>
        <SelectValue placeholder="Select a view" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={"app" as GroupBy}>Apps</SelectItem>
        <SelectItem value={"tag" as GroupBy}>Tags</SelectItem>
        <SelectItem value={"tag-show-untagged" as GroupBy}>
          Tags (untagged separate)
        </SelectItem>
      </SelectContent>
    </Select>
  );
}
