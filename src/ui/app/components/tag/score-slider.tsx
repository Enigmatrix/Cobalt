"use client";

import * as React from "react";
import * as SliderPrimitive from "@radix-ui/react-slider";

import { cn } from "@/lib/utils";
import { useTheme } from "@/components/theme-provider";

function ScoreSlider({
  className,
  defaultValue = 0,
  step = 1,
  value,
  min = -100,
  max = 100,
  onValueChange,
  ...props
}: Omit<
  React.ComponentProps<typeof SliderPrimitive.Root>,
  "defaultValue" | "value" | "onValueChange"
> & {
  defaultValue?: number;
  value?: number;
  onValueChange?: (value: number) => void;
}) {
  const { theme } = useTheme();
  const _values = React.useMemo(
    () =>
      value !== undefined
        ? [value]
        : defaultValue !== undefined
          ? [defaultValue]
          : [min, max],
    [value, defaultValue, min, max],
  );

  const handleValueChange = React.useCallback(
    (values: number[]) => {
      if (onValueChange) {
        onValueChange(values[0]);
      }
    },
    [onValueChange],
  );

  return (
    <SliderPrimitive.Root
      data-slot="slider"
      defaultValue={[defaultValue]}
      value={[value ?? defaultValue]}
      step={step}
      min={min}
      max={max}
      onValueChange={handleValueChange}
      className={cn(
        "relative flex w-full touch-none items-center select-none data-[disabled]:opacity-50 data-[orientation=vertical]:h-full data-[orientation=vertical]:min-h-44 data-[orientation=vertical]:w-auto data-[orientation=vertical]:flex-col",
        className,
      )}
      {...props}
    >
      <SliderPrimitive.Track
        data-slot="slider-track"
        className={cn(
          "relative grow overflow-hidden rounded-full data-[orientation=horizontal]:h-1.5 data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-1.5",
          {
            "bg-red-300": theme !== "dark",
            "bg-red-800": theme === "dark",
          },
        )}
      >
        <SliderPrimitive.Range
          data-slot="slider-range"
          className={cn(
            "absolute data-[orientation=horizontal]:h-full data-[orientation=vertical]:w-full",
            {
              "bg-green-300": theme !== "dark",
              "bg-green-800": theme === "dark",
            },
          )}
        />
      </SliderPrimitive.Track>
      {Array.from({ length: _values.length }, (_, index) => (
        <SliderPrimitive.Thumb
          data-slot="slider-thumb"
          key={index}
          className="border-foreground bg-background ring-ring/50 block size-4 shrink-0 rounded-full border shadow-sm transition-[color,box-shadow] hover:ring-4 focus-visible:ring-4 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50"
        />
      ))}
    </SliderPrimitive.Root>
  );
}

export { ScoreSlider };
