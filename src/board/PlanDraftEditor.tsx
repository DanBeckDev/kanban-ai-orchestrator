import { useEffect, useState, type FormEvent } from "react";
import { ArrowDownIcon, ArrowUpIcon, PlusIcon, Trash2Icon } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

import { timestamp } from "./presentation";
import type {
  AgentProfile,
  AgentEffort,
  AgentModelPreference,
  BoardPlan,
  ProposePlanRequest,
  WorkItemBudget,
} from "./types";

type PlanDraftEditorProps = Readonly<{
  agentProfiles: readonly AgentProfile[];
  boardId: string;
  busy: boolean;
  defaultTicketWorkerProfileName?: string;
  plan: BoardPlan;
  onCancel: () => void;
  onSave: (request: ProposePlanRequest) => Promise<void>;
}>;

type EditableTask = Readonly<{
  id: string;
  title: string;
  description: string;
  acceptanceCriteria: readonly string[];
  budget: WorkItemBudget;
  requiresHumanReview: boolean;
  assignedAgentProfileName?: string;
  assignedAgentModel: AgentModelPreference;
  assignedAgentEffort: AgentEffort;
}>;

const noWorker = "__none__";

export function PlanDraftEditor({
  agentProfiles,
  boardId,
  busy,
  defaultTicketWorkerProfileName,
  plan,
  onCancel,
  onSave,
}: PlanDraftEditorProps) {
  const [tasks, setTasks] = useState<readonly EditableTask[]>(() =>
    tasksFromPlan(plan),
  );

  useEffect(() => {
    setTasks(tasksFromPlan(plan));
  }, [plan]);

  const taskNames = new Map(tasks.map((task) => [task.id, task.title]));
  const dependencies = plan.preview.dependencies.filter(
    (dependency) =>
      taskNames.has(dependency.upstreamWorkItemId) &&
      taskNames.has(dependency.downstreamWorkItemId),
  );

  function updateTask(id: string, changes: Partial<EditableTask>) {
    setTasks((current) =>
      current.map((task) => (task.id === id ? { ...task, ...changes } : task)),
    );
  }

  function moveTask(id: string, direction: -1 | 1) {
    setTasks((current) => {
      const index = current.findIndex((task) => task.id === id);
      const destination = index + direction;
      if (index < 0 || destination < 0 || destination >= current.length) {
        return current;
      }
      const reordered = [...current];
      [reordered[index], reordered[destination]] = [
        reordered[destination],
        reordered[index],
      ];
      return reordered;
    });
  }

  function removeTask(id: string) {
    if (tasks.length === 1) return;
    setTasks((current) => current.filter((task) => task.id !== id));
  }

  function addTask() {
    const id = `draft-task-${Date.now()}-${tasks.length + 1}`;
    setTasks((current) => [
      ...current,
      {
        id,
        title: "New task",
        description: "Describe the smallest useful outcome for this task.",
        acceptanceCriteria: ["State how this task is complete."],
        budget: {},
        requiresHumanReview: false,
        assignedAgentProfileName: defaultTicketWorkerProfileName,
        assignedAgentModel: { kind: "provider_default" },
        assignedAgentEffort: "provider_default",
      },
    ]);
  }

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSave({
      planId: plan.preview.id,
      boardId,
      proposedBy: "orchestrator",
      proposedAt: timestamp(),
      workItems: tasks.map((task) => ({
        workItemId: task.id,
        title: task.title,
        description: task.description,
        acceptanceCriteria: task.acceptanceCriteria,
        budget: task.budget,
        requiresHumanReview: task.requiresHumanReview,
        assignedAgentProfileName: task.assignedAgentProfileName,
        assignedAgentModel: task.assignedAgentModel,
        assignedAgentEffort: task.assignedAgentEffort,
      })),
      dependencies: dependencies.map((dependency) => ({
        dependencyId: dependency.id,
        upstreamWorkItemId: dependency.upstreamWorkItemId,
        downstreamWorkItemId: dependency.downstreamWorkItemId,
        kind: dependency.kind,
        reason: dependency.reason,
        owner: dependency.owner,
        nextAction: dependency.nextAction,
      })),
      unresolvedAssumptions: plan.preview.unresolvedAssumptions,
    });
  }

  return (
    <form
      aria-label="Edit plan proposal"
      className="plan-draft-editor"
      onSubmit={save}
    >
      <div>
        <h3>Edit proposed tasks</h3>
        <p className="field-hint">
          Change the scope, order, review requirement, or ticket worker. Task
          identifiers stay hidden; nothing changes on the board until you save
          this preview and confirm it.
        </p>
      </div>
      <FieldGroup>
        {tasks.map((task, index) => (
          <TaskEditor
            agentProfiles={agentProfiles}
            canMoveDown={index < tasks.length - 1}
            canMoveUp={index > 0}
            canRemove={tasks.length > 1}
            index={index}
            key={task.id}
            task={task}
            onMove={moveTask}
            onRemove={removeTask}
            onUpdate={updateTask}
          />
        ))}
      </FieldGroup>
      {plan.preview.dependencies.length !== dependencies.length && (
        <p className="field-hint" role="status">
          Removing a task also removes its{" "}
          {plan.preview.dependencies.length - dependencies.length} related
          dependency link
          {plan.preview.dependencies.length - dependencies.length === 1
            ? ""
            : "s"}
          .
        </p>
      )}
      {dependencies.length > 0 && (
        <p className="field-hint">
          {dependencies.length} dependency link
          {dependencies.length === 1 ? "" : "s"} will be kept. Review the full
          relationship map in Dependencies after confirmation.
        </p>
      )}
      <div className="form-actions">
        <Button
          disabled={busy}
          onClick={addTask}
          type="button"
          variant="outline"
        >
          <PlusIcon data-icon="inline-start" />
          Add task
        </Button>
        <Button disabled={busy} type="submit">
          Save revised preview
        </Button>
        <Button
          disabled={busy}
          onClick={onCancel}
          type="button"
          variant="ghost"
        >
          Cancel
        </Button>
      </div>
    </form>
  );
}

function TaskEditor({
  agentProfiles,
  canMoveDown,
  canMoveUp,
  canRemove,
  index,
  task,
  onMove,
  onRemove,
  onUpdate,
}: Readonly<{
  agentProfiles: readonly AgentProfile[];
  canMoveDown: boolean;
  canMoveUp: boolean;
  canRemove: boolean;
  index: number;
  task: EditableTask;
  onMove: (id: string, direction: -1 | 1) => void;
  onRemove: (id: string) => void;
  onUpdate: (id: string, changes: Partial<EditableTask>) => void;
}>) {
  const fieldPrefix = `plan-task-${index + 1}`;
  return (
    <fieldset className="plan-task-editor">
      <legend>Task {index + 1}</legend>
      <div className="plan-task-editor-actions">
        <Button
          aria-label={`Move task ${index + 1} up`}
          disabled={!canMoveUp}
          onClick={() => onMove(task.id, -1)}
          size="icon"
          type="button"
          variant="ghost"
        >
          <ArrowUpIcon />
        </Button>
        <Button
          aria-label={`Move task ${index + 1} down`}
          disabled={!canMoveDown}
          onClick={() => onMove(task.id, 1)}
          size="icon"
          type="button"
          variant="ghost"
        >
          <ArrowDownIcon />
        </Button>
        <Button
          aria-label={`Remove task ${index + 1}`}
          disabled={!canRemove}
          onClick={() => onRemove(task.id)}
          size="icon"
          type="button"
          variant="ghost"
        >
          <Trash2Icon />
        </Button>
      </div>
      <Field>
        <FieldLabel htmlFor={`${fieldPrefix}-title`}>Task name</FieldLabel>
        <Input
          id={`${fieldPrefix}-title`}
          onChange={(event) => onUpdate(task.id, { title: event.target.value })}
          required
          value={task.title}
        />
      </Field>
      <Field>
        <FieldLabel htmlFor={`${fieldPrefix}-description`}>Scope</FieldLabel>
        <Textarea
          id={`${fieldPrefix}-description`}
          onChange={(event) =>
            onUpdate(task.id, { description: event.target.value })
          }
          required
          value={task.description}
        />
      </Field>
      <Field>
        <FieldLabel htmlFor={`${fieldPrefix}-criteria`}>
          Completion criteria
        </FieldLabel>
        <Textarea
          id={`${fieldPrefix}-criteria`}
          onChange={(event) =>
            onUpdate(task.id, {
              acceptanceCriteria: event.target.value
                .split("\n")
                .map((criterion) => criterion.trim())
                .filter(Boolean),
            })
          }
          required
          value={task.acceptanceCriteria.join("\n")}
        />
        <FieldDescription>Use one clear criterion per line.</FieldDescription>
      </Field>
      <Field>
        <FieldLabel htmlFor={`${fieldPrefix}-worker`}>Ticket worker</FieldLabel>
        <Select
          onValueChange={(value) =>
            onUpdate(task.id, {
              assignedAgentProfileName: value === noWorker ? undefined : value,
            })
          }
          value={task.assignedAgentProfileName ?? noWorker}
        >
          <SelectTrigger id={`${fieldPrefix}-worker`}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value={noWorker}>Use project default</SelectItem>
              {agentProfiles.map((profile) => (
                <SelectItem key={profile.name} value={profile.name}>
                  {profile.name}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
      </Field>
      <label className="checkbox-field">
        <input
          checked={task.requiresHumanReview}
          name={`${fieldPrefix}-requires-human-review`}
          onChange={(event) =>
            onUpdate(task.id, { requiresHumanReview: event.target.checked })
          }
          type="checkbox"
        />
        Require a person to approve this task before it is done
      </label>
    </fieldset>
  );
}

function tasksFromPlan(plan: BoardPlan): readonly EditableTask[] {
  return plan.preview.workItems.map((task) => ({
    id: task.id,
    title: task.title,
    description: task.description,
    acceptanceCriteria: task.acceptanceCriteria,
    budget: task.budget,
    requiresHumanReview: task.requiresHumanReview,
    assignedAgentProfileName: task.assignedAgentProfileName,
    assignedAgentModel: task.assignedAgentModel,
    assignedAgentEffort: task.assignedAgentEffort,
  }));
}
