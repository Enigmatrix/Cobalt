import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { Check, Edit2 } from "lucide-react";
import { useState } from "react";

export function EditableText({
  text,
  className,
  buttonClassName,
  onSubmit,
}: {
  text: string;
  className?: ClassValue;
  buttonClassName?: ClassValue;
  onSubmit: (text: string) => Promise<void>;
}) {
  const [isEditing, setIsEditing] = useState(false);
  const [value, setValue] = useState(text);

  const handleSubmit = async () => {
    if (value !== text) {
      await onSubmit(value);
    }
    setIsEditing(false);
  };

  const handleKeyDown = async (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      await handleSubmit();
      setIsEditing(false);
    }
    if (e.key === "Escape") {
      setValue(text);
      setIsEditing(false);
    }
  };

  if (isEditing) {
    return (
      <div className={cn("flex items-center gap-2 min-w-0 w-full", className)}>
        <input
          type="text"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={handleSubmit}
          className="bg-transparent outline-hidden min-w-0"
          style={{ width: `${value.length + 2}ch` }}
          autoFocus
        />
        <Button
          variant="ghost"
          size="icon"
          onClick={handleSubmit}
          className={cn(
            "shrink-0 p-0 w-auto h-auto text-muted-foreground",
            buttonClassName,
          )}
        >
          <Check />
        </Button>
      </div>
    );
  }

  return (
    <div
      className={cn("flex items-center gap-2 cursor-pointer", className)}
      onClick={() => setIsEditing(true)}
    >
      <div className="truncate flex-1" title={text}>
        {text}
      </div>
      <Button
        variant="ghost"
        size="icon"
        className={cn(
          "shrink-0 p-0 w-auto h-auto text-muted-foreground",
          buttonClassName,
        )}
      >
        <Edit2 className="size-4" />
      </Button>
    </div>
  );
}
