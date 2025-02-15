import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { CheckIcon, ChevronDown, XIcon } from "lucide-react";

import { cn } from "@/lib/utils";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import { Text } from "@/components/ui/text";

/**
 * Variants for the multi-select component to handle different styles.
 * Uses class-variance-authority (cva) to define different styles based on "variant" prop.
 */
const multiSelectVariants = cva("min-w-0 m-1 font-normal", {
  variants: {
    variant: {
      default: "border-foreground/10 text-foreground bg-card hover:bg-card/80",
      secondary:
        "border-foreground/10 bg-secondary text-secondary-foreground hover:bg-secondary/80",
      destructive:
        "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
      inverted: "inverted",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

/**
 * Props for MultiSelect component
 */
interface MultiSelectProps<T>
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "defaultValue">,
    VariantProps<typeof multiSelectVariants> {
  /**
   * An array of option objects to be displayed in the multi-select component.
   * Each option object has a label, value, and an optional icon.
   */
  options: {
    /** The text to display for the option. */
    label: string;
    /** The unique value associated with the option. */
    id: T;
    /** Optional icon component to display alongside the option. */
    icon?: React.ComponentType<{ className?: string }>;
    /** Optional render function to customize how the option is displayed. */
    render?: React.ComponentType<{ id: T; remove: () => void }>;
  }[];

  /** Values after filtering. Set to undefined to use default filtering */
  filteredValues?: T[];

  /**
   * Callback function triggered when the selected values change.
   * Receives an array of the new selected values.
   */
  onValueChange: (value: T[]) => void;

  /** The default selected values when the component mounts. */
  defaultValue?: T[];

  /** Search query value */
  search?: string;
  /** Set the search query value */
  onSearchChanged?: (search: string) => void;

  /**
   * Placeholder text to be displayed when no values are selected.
   * Optional, defaults to "Select options".
   */
  placeholder?: string;

  /**
   * Maximum number of items to display. Extra selected items will be summarized.
   * Optional, defaults to 3.
   */
  maxCount?: number;

  /**
   * The modality of the popover. When set to true, interaction with outside elements
   * will be disabled and only popover content will be visible to screen readers.
   * Optional, defaults to false.
   */
  modalPopover?: boolean;

  /**
   * If true, renders the multi-select component as a child of another component.
   * Optional, defaults to false.
   */
  asChild?: boolean;

  /**
   * Additional class names to apply custom styles to the multi-select component.
   * Optional, can be used to add custom styles.
   */
  className?: string;

  /**
   * Show clear button and X icon.
   */
  showClear?: boolean;

  /**
   * Show close button.
   */
  showClose?: boolean;

  /**
   * Show toggle select all.
   */
  showToggleSelectAll?: boolean;

  /**
   * Hook backspace key to remove last selected value.
   */
  hookBackspace?: boolean;

  /*
   * Right icon to display in the multi-select component.
   */
  rightIcon?: (className?: string) => React.ReactNode;
}

export function MultiSelect<T>({
  options,
  filteredValues,
  onValueChange,
  variant,
  defaultValue = [],
  placeholder = "Select options",
  search,
  onSearchChanged,
  maxCount = 3,
  modalPopover = false,
  asChild = false,
  className,
  showClear = false,
  showClose = false,
  showToggleSelectAll = false,
  hookBackspace = false,
  rightIcon: rightIconInner,
  ...props
}: MultiSelectProps<T>) {
  const [selectedValues, setSelectedValues] = React.useState<T[]>(defaultValue);
  const [isPopoverOpen, setIsPopoverOpen] = React.useState(false);
  const rightIcon =
    rightIconInner ?? ((className) => <ChevronDown className={className} />);

  const handleInputKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      setIsPopoverOpen(true);
    } else if (
      hookBackspace &&
      event.key === "Backspace" &&
      !event.currentTarget.value
    ) {
      const newSelectedValues = [...selectedValues];
      newSelectedValues.pop();
      setSelectedValues(newSelectedValues);
      onValueChange(newSelectedValues);
    }
  };

  const toggleOption = (option: T) => {
    const newSelectedValues = selectedValues.includes(option)
      ? selectedValues.filter((value) => value !== option)
      : [...selectedValues, option];
    setSelectedValues(newSelectedValues);
    onValueChange(newSelectedValues);
  };

  const handleClear = () => {
    setSelectedValues([]);
    onValueChange([]);
  };

  const handleTogglePopover = () => {
    setIsPopoverOpen((prev) => !prev);
  };

  const clearExtraOptions = () => {
    const newSelectedValues = selectedValues.slice(0, maxCount);
    setSelectedValues(newSelectedValues);
    onValueChange(newSelectedValues);
  };

  const toggleAll = () => {
    if (selectedValues.length === options.length) {
      handleClear();
    } else {
      const allValues = options.map((option) => option.id);
      setSelectedValues(allValues);
      onValueChange(allValues);
    }
  };

  const filteredOptions = React.useMemo(
    () =>
      filteredValues
        ? filteredValues.map((v) => options.find((o) => o.id === v)!)
        : options,
    [filteredValues, options],
  );

  return (
    <Popover
      open={isPopoverOpen}
      onOpenChange={setIsPopoverOpen}
      modal={modalPopover}
    >
      <PopoverTrigger asChild={asChild}>
        <Button
          {...props}
          onClick={handleTogglePopover}
          className={cn(
            "flex w-full p-1 rounded-md border min-h-10 h-auto items-center justify-between bg-inherit hover:bg-inherit [&_svg]:pointer-events-auto",
            className,
          )}
        >
          {selectedValues.length > 0 ? (
            <div className="flex justify-between items-center w-full">
              <div className="flex flex-wrap items-center min-w-0">
                {selectedValues.slice(0, maxCount).map((value) => {
                  const option = options.find((o) => o.id === value);
                  const IconComponent = option?.icon;
                  if (option?.render)
                    return (
                      <option.render
                        id={option.id}
                        remove={() => toggleOption(value)}
                        key={JSON.stringify(value)}
                      />
                    );
                  return (
                    <Badge
                      key={JSON.stringify(value)}
                      className={cn(multiSelectVariants({ variant }))}
                    >
                      {IconComponent && (
                        <IconComponent className="h-4 w-4 mr-2" />
                      )}
                      <Text>{option?.label ?? ""}</Text>
                      <XIcon
                        className="ml-2 h-4 w-4 cursor-pointer"
                        onClick={(event) => {
                          event.stopPropagation();
                          toggleOption(value);
                        }}
                      />
                    </Badge>
                  );
                })}
                {selectedValues.length > maxCount && (
                  <Badge
                    className={cn(
                      "bg-transparent text-foreground border-foreground/1 hover:bg-transparent",
                      multiSelectVariants({ variant }),
                    )}
                  >
                    {`+ ${selectedValues.length - maxCount} more`}
                    <XIcon
                      className="ml-2 h-4 w-4 cursor-pointer"
                      onClick={(event) => {
                        event.stopPropagation();
                        clearExtraOptions();
                      }}
                    />
                  </Badge>
                )}
              </div>
              <div className="flex items-center justify-between">
                {showClear && (
                  <XIcon
                    className="h-4 mx-2 cursor-pointer text-muted-foreground"
                    onClick={(event) => {
                      event.stopPropagation();
                      handleClear();
                    }}
                  />
                )}
                <Separator
                  orientation="vertical"
                  className="flex min-h-6 h-full"
                />
                {rightIcon("h-4 mx-2 cursor-pointer text-muted-foreground")}
              </div>
            </div>
          ) : (
            <div className="flex items-center justify-between w-full mx-auto">
              <span className="text-sm text-muted-foreground mx-3">
                {placeholder}
              </span>
              {rightIcon("h-4 cursor-pointer text-muted-foreground mx-2")}
            </div>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className="p-0 w-[--radix-popover-trigger-width]"
        align="start"
        onEscapeKeyDown={() => setIsPopoverOpen(false)}
      >
        <Command shouldFilter={!filteredValues}>
          <CommandInput
            placeholder="Search..."
            onKeyDown={handleInputKeyDown}
            value={search}
            onValueChange={onSearchChanged}
          />
          <CommandList>
            <CommandEmpty>No results found.</CommandEmpty>
            <CommandGroup>
              {showToggleSelectAll && (
                <CommandItem
                  key="all"
                  onSelect={toggleAll}
                  className="cursor-pointer"
                >
                  <div
                    className={cn(
                      "mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary",
                      selectedValues.length === options.length
                        ? "bg-primary text-primary-foreground"
                        : "opacity-50 [&_svg]:invisible",
                    )}
                  >
                    <CheckIcon className="h-4 w-4" />
                  </div>
                  <span>(Select All)</span>
                </CommandItem>
              )}
              {/* Hidden option eats up select - fixes bug where scroll of searched item is random and annoying.
                  source of bug ref: https://github.com/pacocoursey/cmdk/issues/317 
                  this solution (not even the main solution lmao) ref: https://github.com/pacocoursey/cmdk/issues/171
                  another fix ref: https://github.com/pacocoursey/cmdk/issues/233
               */}
              {filteredOptions.length !== 0 && (
                <CommandItem value="-" className="hidden" />
              )}
              {filteredOptions.map((option) => {
                const isSelected = selectedValues.includes(option.id);
                return (
                  <CommandItem
                    key={JSON.stringify(option.id)}
                    value={JSON.stringify(option.id)}
                    onSelect={() => toggleOption(option.id)}
                    className="cursor-pointer"
                  >
                    <div
                      className={cn(
                        "mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary",
                        isSelected
                          ? "bg-primary text-primary-foreground"
                          : "opacity-50 [&_svg]:invisible",
                      )}
                    >
                      <CheckIcon className="h-4 w-4" />
                    </div>
                    {option.icon && (
                      <option.icon className="mr-2 h-4 w-4 text-muted-foreground" />
                    )}
                    <Text>{option.label}</Text>
                  </CommandItem>
                );
              })}
            </CommandGroup>
            <CommandSeparator />
            {(showClear || showClose) && (
              <CommandGroup>
                <div className="flex items-center justify-between">
                  {selectedValues.length > 0 && showClear && (
                    <>
                      <CommandItem
                        onSelect={handleClear}
                        className="flex-1 justify-center cursor-pointer"
                      >
                        Clear
                      </CommandItem>
                      <Separator
                        orientation="vertical"
                        className="flex min-h-6 h-full"
                      />
                    </>
                  )}
                  {showClose && (
                    <CommandItem
                      onSelect={() => setIsPopoverOpen(false)}
                      className="flex-1 justify-center cursor-pointer max-w-full"
                    >
                      Close
                    </CommandItem>
                  )}
                </div>
              </CommandGroup>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

MultiSelect.displayName = "MultiSelect";
