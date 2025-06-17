import useResizeObserver from "@react-hook/resize-observer";
import { useState, useLayoutEffect } from "react";

export function useWidth(
  target: React.RefObject<HTMLElement | null>,
  initWidth?: number,
) {
  const [width, setWidth] = useState(initWidth ?? 0);

  useLayoutEffect(() => {
    if (!target?.current) return;
    setWidth(target.current.getBoundingClientRect().width);
  }, [target]);

  // Where the magic happens
  useResizeObserver(target, (entry) => setWidth(entry.contentRect.width));
  return width;
}
