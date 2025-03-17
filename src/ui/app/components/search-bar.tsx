import * as React from "react";

import { cn } from "@/lib/utils";
import { Search } from "lucide-react";

const SearchBar = React.forwardRef<
  HTMLInputElement,
  React.ComponentProps<"input">
>(({ className, ...props }) => {
  return (
    <div
      className={cn(
        "flex items-center pl-3 rounded-md border border-input bg-background w-full h-9",
        "placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input",
        "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]",
        "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
        className,
      )}
    >
      <Search size={16} />
      <input
        type="search"
        className={cn(
          " flex h-9 w-full min-w-0 border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm border-none",
        )}
        {...props}
      />
    </div>
  );
});
SearchBar.displayName = "SearchBar";

export { SearchBar };
