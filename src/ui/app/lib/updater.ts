import { error } from "@/lib/log";
import { check } from "@tauri-apps/plugin-updater";
import { toast } from "sonner";

export function checkForUpdatesBackground() {
  // this can be awaited, but we don't need to since we want it to run in the background
  checkForUpdatesAsync().catch(error);
}

export async function checkForUpdatesAsync() {
  const update = await check();
  if (update) {
    let ignore = false;
    const toastId = toast.loading("Update downloading ...", {
      id: "update-download",
      onDismiss: () => {
        ignore = true;
        update.close().catch(error);
      },
      dismissible: true,
      closeButton: true,
      duration: Infinity,
    });
    let downloaded = 0;
    let contentLength = 0;
    await update.download((event) => {
      if (ignore) return;
      switch (event.event) {
        case "Started":
          if (event.data.contentLength) {
            contentLength = event.data.contentLength;
          }
          // toast.loading("Starting update download ...", { id: toastId });
          break;
        case "Progress": {
          downloaded += event.data.chunkLength;
          const progress =
            contentLength > 0
              ? Math.round((downloaded / contentLength) * 100)
              : 0;
          toast.loading(`Update downloading: ${progress}%`, { id: toastId });
          break;
        }
        case "Finished":
          toast.success("Update ready", {
            id: toastId,
            action: {
              label: "Install",
              onClick: () => {
                // this will close the app and install the update
                update.install().catch(error);
              },
            },
          });
          break;
      }
    });
    // don't need to bother doing this on Windows
    // await relaunch();
  }
}
