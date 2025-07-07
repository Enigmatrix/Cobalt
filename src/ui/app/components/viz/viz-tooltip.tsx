import { cn } from "@/lib/utils";
import {
  flip,
  offset,
  shift,
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

export function VizTooltip({
  show = true,
  children,
  className,
  targetRef,
  delta = 10,
}: TooltipProps) {
  const [isOpen, setIsOpen] = useState(false);

  const { refs, floatingStyles, context } = useFloating({
    open: isOpen,
    onOpenChange: setIsOpen,
    placement: "top-start",
    middleware: [offset(delta), shift(), flip()],
  });

  const clientPoint = useClientPoint(context, {
    axis: "both",
  });

  // Typically, we use the getReferenceProps() on the anchor element.
  // However, to avoid re-rendering woes, we skip that and just use the
  // the useEffect below to manually handle the mouse enter and leave events.
  // See https://floating-ui.com/docs/useinteractions#external-reference for
  // an actual example of how it should be used.
  const { getFloatingProps } = useInteractions([clientPoint]);

  useEffect(() => {
    const target = targetRef?.current;
    if (!target) return;

    const handleMouseEnter = () => {
      setIsOpen(true);
    };

    const handleMouseLeave = () => {
      setIsOpen(false);
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

  if (!show || !isOpen) return null;

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
