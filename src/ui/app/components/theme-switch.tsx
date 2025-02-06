import * as React from "react";
import * as SwitchPrimitives from "@radix-ui/react-switch";

import { cn } from "@/lib/utils";
import { Moon, Sun } from "lucide-react";
import {
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Tooltip } from "@radix-ui/react-tooltip";
import type { Theme } from "@/components/theme-provider";

type ThemeSwitchProps = Omit<
  React.ComponentProps<typeof SwitchPrimitives.Root>,
  "value" | "onValueChange" | "checked" | "onCheckedChange"
> & {
  value: Theme;
  onValueChange: (value: Theme) => void;
};

const ThemeSwitch = React.forwardRef<
  React.ElementRef<typeof SwitchPrimitives.Root>,
  ThemeSwitchProps
>(({ className, value, onValueChange, ...props }, ref) => (
  <TooltipProvider>
    <Tooltip>
      <TooltipTrigger asChild>
        <SwitchPrimitives.Root
          className={cn(
            "peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 bg-input",
            className,
          )}
          checked={value === "dark"}
          onCheckedChange={(checked) =>
            onValueChange(checked ? "dark" : "light")
          }
          {...props}
          ref={ref}
        >
          <SwitchPrimitives.Thumb
            className={cn(
              "pointer-events-none flex h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0",
            )}
          >
            {value === "dark" ? (
              <Moon size={16} className="m-auto" />
            ) : (
              <Sun size={16} className="m-auto" />
            )}
          </SwitchPrimitives.Thumb>
        </SwitchPrimitives.Root>
      </TooltipTrigger>
      <TooltipContent>
        Currently: {value === "dark" ? "Dark" : "Light"} Mode
      </TooltipContent>
    </Tooltip>
  </TooltipProvider>
));
ThemeSwitch.displayName = "ThemeSwitch";

export { ThemeSwitch };
