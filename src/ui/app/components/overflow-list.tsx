import { useWidth } from "@/hooks/use-width";
import { cn } from "@/lib/utils";
import {
  TooltipProvider,
  TooltipTrigger,
  TooltipContent,
  Tooltip,
} from "@/components/ui/tooltip";
import type { ClassValue } from "clsx";
import _ from "lodash";
import {
  type ReactNode,
  useRef,
  useLayoutEffect,
  useState,
  useMemo,
} from "react";

// Assumes rendered items are same height. Height of this container
// must be manually set to height of the rendered item.
export function HorizontalOverflowList<T>({
  className,
  items,
  renderItem,
  renderOverflowItem,
  renderOverflowSign,
}: {
  className: ClassValue;
  items: T[];
  renderItem: (item: T) => ReactNode;
  renderOverflowItem: (item: T) => ReactNode;
  renderOverflowSign: (overflowItems: T[]) => ReactNode;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const width = useWidth(ref);
  useLayoutEffect(() => {
    if (!ref.current) return;
    const items = ref.current.children;
    const firstItem = items.item(0) as HTMLElement;
    const offsetLeft = firstItem?.offsetLeft;
    const offsetTop = firstItem?.offsetTop;
    const overflowIndex = _.findIndex(
      items,
      (item) =>
        (item as HTMLElement).offsetLeft === offsetLeft &&
        (item as HTMLElement).offsetTop !== offsetTop,
    );
    setOverflowIndex(overflowIndex);
  }, [width]);
  const [overflowIndex, setOverflowIndex] = useState(0);
  const overflowItems = useMemo(
    () => items.slice(overflowIndex),
    [items, overflowIndex],
  );

  return (
    <div
      ref={ref}
      className={cn("flex flex-wrap items-center overflow-hidden", className)}
    >
      {items.map((item, index) => (
        <div className="flex items-center" key={index}>
          {renderItem(item)}

          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger
                asChild
                className={cn({
                  hidden: index !== overflowIndex - 1,
                })}
              >
                {renderOverflowSign(overflowItems)}
              </TooltipTrigger>
              <TooltipContent>
                <div className="flex flex-col gap-2 py-2">
                  {overflowItems.map(renderOverflowItem)}
                </div>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      ))}
    </div>
  );
}
