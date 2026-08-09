import { AlertCircleIcon, RefreshCwIcon } from "lucide-react";
import type { ComponentProps } from "react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { BoardLibrary } from "./BoardLibrary";
import { BoardSetup } from "./BoardSetup";
import { BoardView } from "./BoardView";
import type { BoardSnapshot } from "./types";

type BoardLibraryProps = ComponentProps<typeof BoardLibrary>;
type BoardSetupProps = ComponentProps<typeof BoardSetup>;
type BoardViewProps = ComponentProps<typeof BoardView>;

type BoardWorkspaceScreenProps = Readonly<{
  boardLibraryLoadFailed: boolean;
  boardLibraryLoaded: boolean;
  boardLibraryProps: BoardLibraryProps;
  boardSetupProps: BoardSetupProps;
  error?: string;
  onRetryBoardLibrary: () => void;
  showBoardSetup: boolean;
}> &
  (
    | Readonly<{
        boardViewProps: BoardViewProps;
        snapshot: BoardSnapshot;
      }>
    | Readonly<{
        boardViewProps?: undefined;
        snapshot?: undefined;
      }>
  );

export function BoardWorkspaceScreen(props: BoardWorkspaceScreenProps) {
  const {
    boardLibraryLoadFailed,
    boardLibraryLoaded,
    boardLibraryProps,
    boardSetupProps,
    error,
    onRetryBoardLibrary,
    showBoardSetup,
    snapshot,
  } = props;
  return (
    <section className="board-shell">
      {error !== undefined && !boardLibraryLoadFailed && (
        <Alert
          aria-live="polite"
          className="error-notice"
          variant="destructive"
        >
          <AlertCircleIcon aria-hidden="true" />
          <AlertTitle>Kanban could not complete that request.</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      {snapshot === undefined && boardLibraryLoadFailed ? (
        <Empty className="board-library-loading">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <AlertCircleIcon />
            </EmptyMedia>
            <EmptyTitle aria-level={2} role="heading">
              Kanban could not load your boards
            </EmptyTitle>
            <EmptyDescription>
              Try again. If it keeps happening, restart Kanban.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button onClick={onRetryBoardLibrary} type="button">
              <RefreshCwIcon data-icon="inline-start" />
              Try again
            </Button>
          </EmptyContent>
        </Empty>
      ) : snapshot === undefined && !boardLibraryLoaded ? (
        <Empty aria-live="polite" className="board-library-loading">
          <EmptyHeader>
            <EmptyTitle>Loading your local boards…</EmptyTitle>
            <EmptyDescription>
              Reading the boards stored on this device.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : snapshot === undefined && showBoardSetup ? (
        <BoardSetup {...boardSetupProps} />
      ) : snapshot === undefined ? (
        <BoardLibrary {...boardLibraryProps} />
      ) : (
        <BoardView {...props.boardViewProps} />
      )}
    </section>
  );
}
