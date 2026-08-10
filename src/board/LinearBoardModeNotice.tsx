import { Link2Icon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

import { boardLinearMode } from "./linearConnectionPresentation";
import type { BoardSnapshot } from "./types";

type LinearBoardModeNoticeProps = Readonly<{
  snapshot: BoardSnapshot;
}>;

export function LinearBoardModeNotice({
  snapshot,
}: LinearBoardModeNoticeProps) {
  const mode = boardLinearMode(snapshot);

  return (
    <section aria-labelledby="linear-board-mode-title">
      <Alert>
        <Link2Icon aria-hidden="true" />
        <AlertTitle id="linear-board-mode-title">{mode.label}</AlertTitle>
        <AlertDescription>{settingsDescription(mode.mode)}</AlertDescription>
      </Alert>
    </section>
  );
}

function settingsDescription(mode: ReturnType<typeof boardLinearMode>["mode"]) {
  switch (mode) {
    case "local_only":
      return "No task is linked to Linear. Connect an account to load issues, then choose how each link should work.";
    case "read_only":
      return "Every Linear link on this board is read-only. You can inspect and refresh shared fields, but Kanban cannot send an update.";
    case "linked_execution":
      return "At least one task can prepare a public Linear update. Every update remains local until you explicitly send it.";
  }
}
