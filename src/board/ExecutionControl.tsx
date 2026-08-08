import type { Execution } from "./types";

type ExecutionControlProps = Readonly<{
  busy: boolean;
  executions: readonly Execution[];
  onStop: (executionId: string) => Promise<void>;
}>;

export function ExecutionControl({
  busy,
  executions,
  onStop,
}: ExecutionControlProps) {
  const execution = executions.find(
    ({ status }) => status === "running" || status === "awaiting_input",
  );
  if (execution === undefined) return null;

  return (
    <section className="execution-control">
      <h5>Direct process control</h5>
      <p>
        Stops the direct worker process and records an interrupted attempt. A
        process started by that worker may need manual cleanup.
      </p>
      <button
        disabled={busy}
        type="button"
        onClick={() => void onStop(execution.id)}
      >
        Stop agent
      </button>
    </section>
  );
}
