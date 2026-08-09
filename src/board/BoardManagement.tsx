import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

import { DependencyForm } from "./DependencyForm";
import { TaskForm } from "./TaskForm";
import type {
  AddDependencyRequest,
  CreateWorkItemRequest,
  WorkItem,
} from "./types";

type BoardManagementProps = Readonly<{
  boardId: string;
  busy: boolean;
  defaultTab: "task" | "dependencies";
  workItems: readonly WorkItem[];
  onAddDependency: (request: AddDependencyRequest) => Promise<void>;
  onCreateWorkItem: (request: CreateWorkItemRequest) => Promise<void>;
  onBack: () => void;
}>;

export function BoardManagement({
  boardId,
  busy,
  defaultTab,
  workItems,
  onAddDependency,
  onCreateWorkItem,
  onBack,
}: BoardManagementProps) {
  return (
    <section
      aria-labelledby="organise-work-title"
      className="workspace-surface"
    >
      <SurfaceHeader
        description="Create a standalone task or explain how two tasks depend on one another."
        headingId="organise-work-title"
        onBack={onBack}
        title="Organise work"
      />
      <Tabs defaultValue={defaultTab}>
        <TabsList aria-label="Organise work">
          <TabsTrigger value="task">New task</TabsTrigger>
          <TabsTrigger value="dependencies">Dependencies</TabsTrigger>
        </TabsList>
        <TabsContent value="task">
          <TaskForm boardId={boardId} busy={busy} onCreate={onCreateWorkItem} />
        </TabsContent>
        <TabsContent value="dependencies">
          <DependencyForm
            busy={busy}
            workItems={workItems}
            onCreate={onAddDependency}
          />
        </TabsContent>
      </Tabs>
    </section>
  );
}

export function SurfaceHeader({
  description,
  headingId,
  onBack,
  title,
}: Readonly<{
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
        Back to board
      </Button>
    </header>
  );
}
