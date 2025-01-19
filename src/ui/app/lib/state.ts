import { invoke } from "@tauri-apps/api/core";

export async function initState() {
  await invoke("init_state");
}
