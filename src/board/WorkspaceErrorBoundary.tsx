import { Component, type ReactNode } from "react";
import { AlertCircleIcon, RefreshCwIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";

type WorkspaceErrorBoundaryProps = Readonly<{
  children: ReactNode;
}>;

type WorkspaceErrorBoundaryState = Readonly<{
  hasError: boolean;
}>;

export class WorkspaceErrorBoundary extends Component<
  WorkspaceErrorBoundaryProps,
  WorkspaceErrorBoundaryState
> {
  state: WorkspaceErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): WorkspaceErrorBoundaryState {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return (
        <Empty className="board-library-loading" role="alert">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <AlertCircleIcon />
            </EmptyMedia>
            <EmptyTitle aria-level={2} role="heading">
              Kanban needs to try again
            </EmptyTitle>
            <EmptyDescription>
              Your saved boards and work have not been changed.
            </EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button
              onClick={() => this.setState({ hasError: false })}
              type="button"
            >
              <RefreshCwIcon data-icon="inline-start" />
              Try again
            </Button>
          </EmptyContent>
        </Empty>
      );
    }

    return this.props.children;
  }
}
