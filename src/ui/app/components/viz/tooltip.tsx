import type { ClassValue } from "clsx";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { cn } from "@/lib/utils";

interface TooltipProps {
  children: React.ReactNode;
  show?: boolean;
  className?: ClassValue;
  targetRef: React.RefObject<HTMLElement | null>;
  delta?: { x: number; y: number };
}

const DELTA_X = 10;
const DELTA_Y = 10;

export function Tooltip({
  show = true,
  children,
  className,
  targetRef,
  delta,
}: TooltipProps) {
  const tooltipRef = useRef<HTMLDivElement>(null);
  const positionRef = useRef({ x: 0, y: 0 });
  const rafRef = useRef<number>(0);
  const isOverRef = useRef(false);
  const [shouldRender, setShouldRender] = useState(false);

  useEffect(() => {
    const target = targetRef?.current;
    if (!target) return;

    const updatePosition = () => {
      if (!tooltipRef.current) return;
      const rect = tooltipRef.current.getBoundingClientRect();
      const tooltip = tooltipRef.current;
      const deltaX = delta?.x || DELTA_X;
      const deltaY = delta?.y || DELTA_Y;

      let x = positionRef.current.x + deltaX;
      let y = positionRef.current.y + deltaY;

      if (x + rect.width > window.innerWidth) {
        x = Math.max(0, x - rect.width - deltaX * 2);
      }
      if (y + rect.height > window.innerHeight) {
        y = Math.max(0, y - rect.height - deltaY * 2);
      }

      tooltip.style.transform = `translate(${x}px, ${y}px)`;
      tooltip.style.visibility = "inherit";
    };

    const handleMouseMove = (e: MouseEvent) => {
      positionRef.current = { x: e.clientX, y: e.clientY };
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      rafRef.current = requestAnimationFrame(updatePosition);
    };

    const handleMouseEnter = () => {
      isOverRef.current = true;
      setShouldRender(true);
    };

    const handleMouseLeave = () => {
      isOverRef.current = false;
      setShouldRender(false);
    };

    target.addEventListener("mouseenter", handleMouseEnter);
    target.addEventListener("mouseleave", handleMouseLeave);
    target.addEventListener("mousemove", handleMouseMove);

    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      target.removeEventListener("mouseenter", handleMouseEnter);
      target.removeEventListener("mouseleave", handleMouseLeave);
      target.removeEventListener("mousemove", handleMouseMove);
    };
  }, [targetRef, delta]);

  if (!show || !shouldRender) return null;

  return createPortal(
    <div
      ref={tooltipRef}
      className={cn(
        "rounded-lg border border-border bg-background px-2.5 py-1.5 shadow-xl",
        className,
      )}
      style={{
        position: "fixed",
        left: 0,
        top: 0,
        pointerEvents: "none",
        zIndex: 9999,
        transform: "translate(0, 0)",
        visibility: "hidden",
      }}
    >
      {children}
    </div>,
    document.body,
  );
}
