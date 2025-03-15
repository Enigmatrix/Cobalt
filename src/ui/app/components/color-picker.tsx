import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { COLORS } from "@/lib/color-utils";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { HexColorInput, HexColorPicker } from "react-colorful";

interface ColorPickerProps {
  color?: string;
  onChange?: (value: string) => void;
  className?: ClassValue;
}

export function ColorPicker({
  color = "#000000",
  onChange,
  className,
}: ColorPickerProps) {
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className={cn(
            "w-[240px] justify-start text-left font-normal",
            className,
          )}
        >
          <div className="w-full flex items-center gap-2">
            <div
              className="h-4 w-4 rounded bg-center! bg-cover! transition-all border"
              style={{ backgroundColor: color }}
            />
            <div className="truncate flex-1">{color}</div>
          </div>
        </Button>
      </PopoverTrigger>
      <PopoverContent className="flex flex-col gap-4 w-[232px]">
        <HexColorPicker color={color} onChange={onChange} className="mx-auto" />
        <HexColorInput
          prefixed
          color={color}
          onChange={(v) => onChange?.(v.toLowerCase())}
          className={cn(
            "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-base ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
            "m-auto min-w-20 w-full",
          )}
        />
        <div className="flex flex-wrap gap-2">
          {COLORS.map((color) => (
            <div
              key={color}
              className="h-4 w-4 rounded bg-center! bg-cover! transition-all border"
              style={{ backgroundColor: color }}
              onClick={() => onChange?.(color)}
            />
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
