import { cn } from "@/lib/utils";
import {
  flip,
  offset,
  useClientPoint,
  useFloating,
  useInteractions,
} from "@floating-ui/react";
import type { ClassValue } from "clsx";
import { useEffect, useState } from "react";
import { createPortal } from "react-dom";

interface TooltipProps {
  children: React.ReactNode;
  show?: boolean;
  className?: ClassValue;
  targetRef: React.RefObject<HTMLElement | null>;
  delta?: number;
}

export function Tooltip({
  show = true,
  children,
  className,
  targetRef,
  delta = 10,
}: TooltipProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [shouldRender, setShouldRender] = useState(false);

  const { refs, floatingStyles, context } = useFloating({
    open: isOpen,
    onOpenChange: setIsOpen,
    placement: "top-start",
    middleware: [offset(delta), flip()],
  });

  const clientPoint = useClientPoint(context, {
    axis: "both",
  });

  const { getFloatingProps } = useInteractions([clientPoint]);

  useEffect(() => {
    const target = targetRef?.current;
    if (!target) return;

    const handleMouseEnter = () => {
      setIsOpen(true);
      setShouldRender(true);
    };

    const handleMouseLeave = () => {
      setIsOpen(false);
      setShouldRender(false);
    };

    target.addEventListener("mouseenter", handleMouseEnter);
    target.addEventListener("mouseleave", handleMouseLeave);

    return () => {
      target.removeEventListener("mouseenter", handleMouseEnter);
      target.removeEventListener("mouseleave", handleMouseLeave);
    };
  }, [targetRef]);

  // Set the reference element to the target
  useEffect(() => {
    if (targetRef?.current) {
      refs.setReference(targetRef.current);
    }
  }, [targetRef, refs]);

  if (!show || !shouldRender) return null;

  return createPortal(
    <div
      ref={refs.setFloating}
      className={cn(
        "rounded-lg border border-border bg-background px-2.5 py-1.5 shadow-xl",
        className,
      )}
      style={{
        ...floatingStyles,
        pointerEvents: "none",
        zIndex: 9999,
      }}
      {...getFloatingProps()}
    >
      {children}
    </div>,
    document.body,
  );
}
