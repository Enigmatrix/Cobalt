import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Period } from "@/lib/time";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { useCallback } from "react";

export function PeriodPicker({
  period,
  setPeriod: setPeriodInner,
  className,
}: {
  period: Period;
  setPeriod: (period: Period) => void;
  className?: ClassValue;
}) {
  const setPeriod = useCallback(
    (p: string) => {
      setPeriodInner(p as Period);
    },
    [setPeriodInner],
  );

  return (
    <Select value={period} onValueChange={setPeriod}>
      <SelectTrigger className={cn("min-w-32 font-medium", className)}>
        <SelectValue placeholder="Select a period" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={"hour" as Period}>Hour</SelectItem>
        <SelectItem value={"day" as Period}>Day</SelectItem>
        <SelectItem value={"week" as Period}>Week</SelectItem>
        <SelectItem value={"month" as Period}>Month</SelectItem>
      </SelectContent>
    </Select>
  );
}
