type RGB = { r: number; g: number; b: number };
import { useTheme } from "@/components/theme-provider";
import α from "color-alpha";
import { useEffect, useState } from "react";

// ref: https://github.com/catppuccin/catppuccin
// Mocha colors
export const COLORS = [
  "#f5e0dc",
  "#f2cdcd",
  "#f5c2e7",
  "#cba6f7",
  "#f38ba8",
  "#eba0ac",
  "#fab387",
  "#f9e2af",
  "#a6e3a1",
  "#94e2d5",
  "#89dceb",
  "#74c7ec",
  "#89b4fa",
  "#b4befe",
  "#cdd6f4", // Text
];

export const randomColor = () => {
  return COLORS[Math.floor(Math.random() * COLORS.length)];
};

export const hexToRgb = (hex: string): RGB | null => {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? {
        r: Number.parseInt(result[1], 16),
        g: Number.parseInt(result[2], 16),
        b: Number.parseInt(result[3], 16),
      }
    : null;
};

export const getVarColorAsHex = (varName: string, a = 1): string => {
  const varVal = getComputedStyle(document.documentElement).getPropertyValue(
    "--" + varName,
  );

  return α(varVal, a);
};

export const useVarColorAsHex = (varName: string, a = 1): string => {
  const theme = useTheme();
  const [color, setColor] = useState(getVarColorAsHex(varName, a));

  useEffect(() => {
    setColor(getVarColorAsHex(varName, a));
  }, [theme, varName, a]);

  return color;
};
