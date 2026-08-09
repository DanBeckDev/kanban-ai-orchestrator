import { open } from "@tauri-apps/plugin-dialog";

export async function selectRepository(): Promise<string | null> {
  return selectDirectory("Choose a local Git repository");
}

export async function selectCloneDestination(): Promise<string | null> {
  return selectDirectory("Choose where to clone the GitHub repository");
}

async function selectDirectory(title: string): Promise<string | null> {
  const selection = await open({
    directory: true,
    multiple: false,
    title,
  });
  return typeof selection === "string" ? selection : null;
}
