import { cn } from "@/lib/utils";
import icon from "../src-tauri/icons/icon.svg";

export function SplashScreen() {
  return (
    <div
      className={cn(
        "fixed inset-0 z-50 flex items-center justify-center bg-background transition-opacity duration-500",
      )}
    >
      <div className="relative h-full w-full flex items-center justify-center">
        {/* Gradient background */}
        <div className="absolute inset-0 overflow-hidden">
          <div
            className={cn(
              "absolute inset-0 bg-linear-to-br from-primary/90 via-primary to-primary/70 transition-opacity duration-500",
            )}
          />
        </div>

        {/* Logo */}
        <div className="flex flex-col items-center z-50 transition-opacity duration-500">
          <img
            src={icon}
            alt="Cobalt Logo"
            className="h-20 w-20 animate-pulse"
          />
          <div className="text-xl mt-2 font-semibold text-primary-foreground">
            Cobalt
          </div>
        </div>
      </div>
    </div>
  );
}
