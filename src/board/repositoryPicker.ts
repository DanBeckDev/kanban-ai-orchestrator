import { open } from "@tauri-apps/plugin-dialog";

export async function selectRepository(): Promise<string | null> {
  const selection = await open({
    directory: true,
    multiple: false,
    title: "Choose a local Git repository",
  });
  return typeof selection === "string" ? selection : null;
}
