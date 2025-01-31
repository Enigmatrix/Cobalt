import * as React from "react";

import { cn } from "@/lib/utils";
import { Search } from "lucide-react";

const SearchBar = React.forwardRef<
  HTMLInputElement,
  React.ComponentProps<"input">
>(({ className, ...props }, ref) => {
  return (
    <div
      className={cn(
        "flex items-center pl-3 rounded-md border border-input bg-background w-full h-10",
        className
      )}
    >
      <Search size={16} />
      <input
        type="search"
        className="text-base bg-transparent px-2 py-2 w-full placeholder:text-muted-foreground outline-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm"
        ref={ref}
        {...props}
      />
    </div>
  );
});
SearchBar.displayName = "SearchBar";

export { SearchBar };
