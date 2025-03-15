import * as React from "react";

import { cn } from "@/lib/utils";
import { Search } from "lucide-react";

const SearchBar = (
  {
    ref,
    className,
    ...props
  }: React.ComponentProps<"input"> & {
    ref: React.RefObject<HTMLInputElement>;
  }
) => {
  return (
    <div
      className={cn(
        "flex items-center pl-3 rounded-md border border-input bg-background w-full h-10",
        className,
      )}
    >
      <Search size={16} />
      <input
        type="search"
        className="text-base bg-transparent px-2 py-2 w-full placeholder:text-muted-foreground outline-hidden disabled:cursor-not-allowed disabled:opacity-50 md:text-sm"
        ref={ref}
        {...props}
      />
    </div>
  );
};
SearchBar.displayName = "SearchBar";

export { SearchBar };
