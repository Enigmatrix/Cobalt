import { error } from "@/lib/log";
import { useCallback, useState } from "react";
import { toast } from "sonner";

interface UseClipboardOptions {
  timeout?: number;
}

export function useClipboard(options: UseClipboardOptions = {}) {
  const { timeout = 2000 } = options;
  const [hasCopied, setHasCopied] = useState(false);

  const copy = useCallback(
    async (text: string) => {
      if (!navigator?.clipboard) {
        toast.error("Clipboard not supported");
        return false;
      }

      try {
        await navigator.clipboard.writeText(text);
        setHasCopied(true);
        toast.success("Copied");

        setTimeout(() => {
          setHasCopied(false);
        }, timeout);

        return true;
      } catch (err) {
        error("Failed to copy text", err);
        setHasCopied(false);
        return false;
      }
    },
    [timeout],
  );

  return { copy, hasCopied };
}
