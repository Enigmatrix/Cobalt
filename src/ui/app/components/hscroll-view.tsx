import * as React from "react";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";

interface HScrollViewProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
  innerClassName?: ClassValue;
}

const adjustPadding = 3;

export function HScrollView({
  children,
  className,
  innerClassName,
  ...props
}: HScrollViewProps) {
  const scrollRef = React.useRef<HTMLDivElement>(null);
  const [showLeftShadow, setShowLeftShadow] = React.useState(false);
  const [showRightShadow, setShowRightShadow] = React.useState(false);

  const checkScroll = () => {
    if (scrollRef.current) {
      const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current;
      setShowLeftShadow(scrollLeft > adjustPadding);
      setShowRightShadow(
        scrollLeft < scrollWidth - clientWidth - adjustPadding,
      );
    }
  };

  React.useEffect(() => {
    checkScroll();
    window.addEventListener("resize", checkScroll);
    return () => window.removeEventListener("resize", checkScroll);
  }, []);

  return (
    <div
      className={cn(
        "relative",
        'before:pointer-events-none before:absolute before:inset-y-0 before:left-0 before:w-[20px] before:bg-linear-to-r before:from-black/30 before:to-transparent before:opacity-0 before:transition-opacity before:duration-200 before:content-[""]',
        'after:pointer-events-none after:absolute after:inset-y-0 after:right-0 after:w-[20px] after:bg-linear-to-l after:from-black/30 after:to-transparent after:opacity-0 after:transition-opacity after:duration-200 after:content-[""]',
        showLeftShadow && "before:opacity-100",
        showRightShadow && "after:opacity-100",
        className,
      )}
    >
      <div
        ref={scrollRef}
        className={cn("overflow-x-auto", innerClassName)}
        onScroll={checkScroll}
        {...props}
      >
        {children}
      </div>
    </div>
  );
}
