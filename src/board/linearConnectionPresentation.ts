import type {
  BoardSnapshot,
  LinearConnectionStatus,
  LinearOAuthConfiguration,
} from "./types";

export const linearLoopbackRedirectUri =
  "http://127.0.0.1:38471/linear/oauth/callback";

export type LinearBoardMode = "local_only" | "read_only" | "linked_execution";

export type LinearBoardModePresentation = Readonly<{
  label: string;
  description: string;
  mode: LinearBoardMode;
}>;

export function productManagedLinearOAuthConfiguration(
  clientId = import.meta.env.VITE_LINEAR_OAUTH_CLIENT_ID,
): LinearOAuthConfiguration | undefined {
  const normalizedClientId = clientId?.trim();
  if (!normalizedClientId) return undefined;

  return {
    clientId: normalizedClientId,
    redirectUri: linearLoopbackRedirectUri,
  };
}

export function boardLinearMode(
  snapshot: BoardSnapshot,
): LinearBoardModePresentation {
  const linearLinks = snapshot.externalLinks.filter(
    ({ connectorId }) => connectorId === "linear",
  );

  if (
    linearLinks.some(
      ({ connectionMode }) => connectionMode === "linked_execution",
    )
  ) {
    return {
      mode: "linked_execution",
      label: "Linked execution",
      description:
        "Some tasks can prepare public Linear updates. Nothing is sent until you explicitly send each update.",
    };
  }

  if (linearLinks.length > 0) {
    return {
      mode: "read_only",
      label: "Linear read-only",
      description:
        "This board can read its linked Linear work. Kanban will not send updates to Linear.",
    };
  }

  return {
    mode: "local_only",
    label: "Local-only board",
    description:
      "This board does not exchange data with Linear. You can connect Linear later in Settings.",
  };
}

export function commentsAreAuthorized(status: LinearConnectionStatus): boolean {
  return (
    status.kind === "connected" &&
    (status.scopes.includes("comments:create") ||
      status.scopes.includes("write"))
  );
}

export function connectedLinearDescription(
  status: LinearConnectionStatus,
): string {
  if (status.kind !== "connected") {
    return "Connect Linear to load issues or choose linked execution.";
  }

  if (commentsAreAuthorized(status)) {
    return "Linked execution is available. Updates stay local until you explicitly send each one.";
  }

  return "Linear is connected in read-only mode. You can load issues; Kanban will not send updates.";
}
