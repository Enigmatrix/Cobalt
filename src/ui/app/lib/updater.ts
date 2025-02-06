import { check } from "@tauri-apps/plugin-updater";

export function checkForUpdatesBackground() {
  // this can be awaited, but we don't need to since we want it to run in the background
  checkForUpdatesAsync().catch(console.error); // TODO log error
}

export async function checkForUpdatesAsync() {
  const update = await check();
  if (update) {
    await update.downloadAndInstall((event) => {
      switch (event.event) {
        // TODO: use Sonner to show a notification
        case "Started":
          // contentLength = event.data.contentLength;
          // console.log(`started downloading ${event.data.contentLength} bytes`);
          break;
        case "Progress":
          // downloaded += event.data.chunkLength;
          // console.log(`downloaded ${downloaded} from ${contentLength}`);
          break;
        case "Finished":
          // console.log('download finished');
          break;
      }
    });
    // don't need to bother doing this on Windows
    // await relaunch();
  }
}
