import { Button } from "@/components/ui/button";
import { TaskForm } from "./TaskForm";
import type { CreateWorkItemRequest } from "./types";

type BoardManagementProps = Readonly<{
  boardId: string;
  busy: boolean;
  onCreateWorkItem: (request: CreateWorkItemRequest) => Promise<void>;
  onBack: () => void;
}>;

export function BoardManagement({
  boardId,
  busy,
  onCreateWorkItem,
  onBack,
}: BoardManagementProps) {
  return (
    <section aria-labelledby="create-task-title" className="workspace-surface">
      <SurfaceHeader
        description="Create a task yourself. Use Dependencies to explain or add relationships between tasks."
        headingId="create-task-title"
        onBack={onBack}
        title="Create task"
      />
      <TaskForm boardId={boardId} busy={busy} onCreate={onCreateWorkItem} />
    </section>
  );
}

export function SurfaceHeader({
  backLabel = "Back to Tickets",
  description,
  headingId,
  onBack,
  title,
}: Readonly<{
  backLabel?: string;
  description: string;
  headingId?: string;
  onBack: () => void;
  title: string;
}>) {
  return (
    <header className="workspace-surface-header">
      <div>
        <p className="eyebrow">Board workspace</p>
        <h2 id={headingId}>{title}</h2>
        <p>{description}</p>
      </div>
      <Button onClick={onBack} type="button" variant="outline">
        {backLabel}
      </Button>
    </header>
  );
}
