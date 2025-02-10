import * as React from "react";
import { Copy, Check } from "lucide-react";

import * as ColorUtils from "@/lib/color-utils";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/lib/utils";
import { useClipboard } from "@/hooks/use-clipboard";

interface ColorPickerProps {
  color?: string;
  onChange?: (value: string) => void;
  className?: string;
}

type ColorMode = "hex" | "rgba" | "hsla";
type CopyState = { [key in ColorMode]: boolean };

export function ColorPicker({
  color = "#000000",
  onChange,
  className,
}: ColorPickerProps) {
  const [currentColor, setCurrentColor] = React.useState(color);
  const [colorMode, setColorMode] = React.useState<ColorMode>("hex");
  const { copy: copyHex, hasCopied: hexCopied } = useClipboard();
  const { copy: copyRgba, hasCopied: rgbaCopied } = useClipboard();
  const { copy: copyHsla, hasCopied: hslaCopied } = useClipboard();
  const colorPlaneRef = React.useRef<HTMLDivElement>(null);
  const isDragging = React.useRef(false);

  const rgb = ColorUtils.hexToRgb(currentColor) || { r: 0, g: 0, b: 0 };
  const hsl = ColorUtils.rgbToHsl(rgb);
  const rgbaString = ColorUtils.formatRgba(rgb);
  const hslaString = ColorUtils.formatHsla(hsl);

  const handleColorChange = (newColor: string) => {
    setCurrentColor(newColor);
    onChange?.(newColor);
  };

  const updateHSL = (h: number, s: number, l: number) => {
    const rgb = ColorUtils.hslToRgb({ h, s, l });
    handleColorChange(ColorUtils.rgbToHex(rgb));
  };

  const handleMouseDown = (e: React.MouseEvent | React.TouchEvent) => {
    isDragging.current = true;
    handleColorPlaneChange(e);
  };

  const handleMouseUp = () => {
    isDragging.current = false;
  };

  const handleMouseMove = (e: React.MouseEvent | React.TouchEvent) => {
    if (isDragging.current) {
      handleColorPlaneChange(e);
    }
  };

  const handleColorPlaneChange = (e: React.MouseEvent | React.TouchEvent) => {
    if (!colorPlaneRef.current) return;

    const rect = colorPlaneRef.current.getBoundingClientRect();
    const clientX = "touches" in e ? e.touches[0].clientX : e.clientX;
    const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;

    const x = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const y = Math.max(0, Math.min(1, (clientY - rect.top) / rect.height));

    updateHSL(hsl.h, Math.round(x * 100), Math.round((1 - y) * 100));
  };

  React.useEffect(() => {
    const handleGlobalMouseUp = () => {
      isDragging.current = false;
    };

    window.addEventListener("mouseup", handleGlobalMouseUp);
    window.addEventListener("touchend", handleGlobalMouseUp);

    return () => {
      window.removeEventListener("mouseup", handleGlobalMouseUp);
      window.removeEventListener("touchend", handleGlobalMouseUp);
    };
  }, []);

  const handleHexChange = (hex: string) => {
    if (/^#[0-9A-Fa-f]{6}$/.test(hex)) {
      handleColorChange(hex);
    }
  };

  const handleRgbChange = (key: keyof typeof rgb, value: string) => {
    const numValue = Number.parseInt(value);
    if (!isNaN(numValue) && numValue >= 0 && numValue <= 255) {
      const newRgb = { ...rgb, [key]: numValue };
      handleColorChange(ColorUtils.rgbToHex(newRgb));
    }
  };

  const handleHslChange = (key: keyof typeof hsl, value: string) => {
    const numValue = Number.parseInt(value);
    if (isNaN(numValue)) return;

    const max = key === "h" ? 360 : 100;
    if (numValue >= 0 && numValue <= max) {
      const newHsl = { ...hsl, [key]: numValue };
      const newRgb = ColorUtils.hslToRgb(newHsl);
      handleColorChange(ColorUtils.rgbToHex(newRgb));
    }
  };

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className={cn(
            "w-[240px] justify-start text-left font-normal",
            className,
          )}
        >
          <div className="w-full flex items-center gap-2">
            <div
              className="h-4 w-4 rounded !bg-center !bg-cover transition-all border"
              style={{ backgroundColor: currentColor }}
            />
            <div className="truncate flex-1">{currentColor}</div>
          </div>
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-80">
        <div className="grid gap-4">
          <div
            ref={colorPlaneRef}
            className="relative w-full h-48 rounded-lg cursor-crosshair touch-none"
            style={{
              backgroundImage: `
                linear-gradient(to left, hsl(${hsl.h} 100% 50% / 1), hsl(${hsl.h} 100% 50% / 0)),
                linear-gradient(to bottom, #ffffff 0%, #000000 100%)
              `,
              backgroundBlendMode: "overlay",
            }}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            onTouchStart={handleMouseDown}
            onTouchMove={handleMouseMove}
            onTouchEnd={handleMouseUp}
          >
            <div
              className="absolute w-4 h-4 border-2 border-white rounded-full -translate-x-1/2 -translate-y-1/2 pointer-events-none shadow-md"
              style={{
                left: `${hsl.s}%`,
                top: `${100 - hsl.l}%`,
                backgroundColor: currentColor,
              }}
            />
          </div>

          <div className="grid gap-2">
            <Label>Hue</Label>
            <div className="relative">
              <Slider
                value={[hsl.h]}
                max={360}
                step={1}
                className="[&_[role=slider]]:h-4 [&_[role=slider]]:w-4 [&_.bg-primary]:bg-transparent [&_.bg-secondary]:bg-transparent"
                onValueChange={([h]) => updateHSL(h, hsl.s, hsl.l)}
                style={{
                  backgroundImage: `linear-gradient(to right, 
                    hsl(0, 100%, 50%),
                    hsl(60, 100%, 50%),
                    hsl(120, 100%, 50%),
                    hsl(180, 100%, 50%),
                    hsl(240, 100%, 50%),
                    hsl(300, 100%, 50%),
                    hsl(360, 100%, 50%)
                  )`,
                }}
              />
              <style>{`
                .slider-thumb {
                  background-color: hsl(${hsl.h}, 100%, 50%) !important;
                  border: 2px solid white !important;
                  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2) !important;
                }
              `}</style>
            </div>
          </div>

          <Tabs
            value={colorMode}
            onValueChange={(v) => setColorMode(v as ColorMode)}
          >
            <TabsList className="w-full">
              <TabsTrigger value="hex" className="flex-1">
                Hex
              </TabsTrigger>
              <TabsTrigger value="rgba" className="flex-1">
                RGBA
              </TabsTrigger>
              <TabsTrigger value="hsla" className="flex-1">
                HSLA
              </TabsTrigger>
            </TabsList>

            <TabsContent value="hex" className="mt-2">
              <div className="flex gap-2">
                <Input
                  value={currentColor}
                  onChange={(e) => handleHexChange(e.target.value)}
                  className="font-mono"
                />
                <Button
                  variant="ghost"
                  size="icon"
                  className="shrink-0"
                  onClick={() => copyHex(currentColor)}
                >
                  {hexCopied ? (
                    <Check className="h-4 w-4" />
                  ) : (
                    <Copy className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </TabsContent>

            <TabsContent value="rgba" className="mt-2">
              <div className="grid gap-4">
                <div className="flex gap-2">
                  <Input value={rgbaString} readOnly className="font-mono" />
                  <Button
                    variant="ghost"
                    size="icon"
                    className="shrink-0"
                    onClick={() => copyRgba(rgbaString)}
                  >
                    {rgbaCopied ? (
                      <Check className="h-4 w-4" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
                <div className="grid grid-cols-4 gap-2">
                  <div>
                    <Label>R</Label>
                    <Input
                      value={rgb.r}
                      onChange={(e) => handleRgbChange("r", e.target.value)}
                      className="font-mono"
                    />
                  </div>
                  <div>
                    <Label>G</Label>
                    <Input
                      value={rgb.g}
                      onChange={(e) => handleRgbChange("g", e.target.value)}
                      className="font-mono"
                    />
                  </div>
                  <div>
                    <Label>B</Label>
                    <Input
                      value={rgb.b}
                      onChange={(e) => handleRgbChange("b", e.target.value)}
                      className="font-mono"
                    />
                  </div>
                  <div>
                    <Label>A</Label>
                    <Input value="1" readOnly className="font-mono" />
                  </div>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="hsla" className="mt-2">
              <div className="grid gap-4">
                <div className="flex gap-2">
                  <Input value={hslaString} readOnly className="font-mono" />
                  <Button
                    variant="ghost"
                    size="icon"
                    className="shrink-0"
                    onClick={() => copyHsla(hslaString)}
                  >
                    {hslaCopied ? (
                      <Check className="h-4 w-4" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
                <div className="grid grid-cols-4 gap-2">
                  <div>
                    <Label>H</Label>
                    <Input
                      value={hsl.h}
                      onChange={(e) => handleHslChange("h", e.target.value)}
                      className="font-mono"
                    />
                  </div>
                  <div>
                    <Label>S</Label>
                    <Input
                      value={hsl.s}
                      onChange={(e) => handleHslChange("s", e.target.value)}
                      className="font-mono"
                    />
                  </div>
                  <div>
                    <Label>L</Label>
                    <Input
                      value={hsl.l}
                      onChange={(e) => handleHslChange("l", e.target.value)}
                      className="font-mono"
                    />
                  </div>
                  <div>
                    <Label>A</Label>
                    <Input value="1" readOnly className="font-mono" />
                  </div>
                </div>
              </div>
            </TabsContent>
          </Tabs>

          <div
            className="h-6 rounded border"
            style={{ backgroundColor: currentColor }}
          />
        </div>
      </PopoverContent>
    </Popover>
  );
}
