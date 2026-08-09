import { useEffect, useState } from "react";

import type { Execution, ExecutionActivityPage } from "./types";

const POLL_INTERVAL_MILLISECONDS = 750;
const ROW_HEIGHT_PIXELS = 48;
const VIEWPORT_HEIGHT_PIXELS = 240;
const OVERSCAN_ROWS = 3;
const MAX_RETAINED_RENDERED_CHUNKS = 128;

type ActivityStreamProps = Readonly<{
  execution: Execution;
  onLoad: (
    executionId: string,
    afterSequence?: number,
  ) => Promise<ExecutionActivityPage>;
}>;

export function ActivityStream({ execution, onLoad }: ActivityStreamProps) {
  const [chunks, setChunks] = useState<ExecutionActivityPage["chunks"]>([]);
  const [error, setError] = useState<string>();

  useEffect(() => {
    let active = true;
    let loading = false;
    let afterSequence: number | undefined;

    async function load() {
      if (loading) return;
      loading = true;
      let hasMore = false;
      const requestedAfterSequence = afterSequence;
      try {
        const page = await onLoad(execution.id, afterSequence);
        if (!active) return;
        if (page.chunks.length > 0) {
          afterSequence = page.chunks.at(-1)?.sequence;
          setChunks((current) => appendChunks(current, page.chunks));
        }
        setError(undefined);
        hasMore = page.hasMore && afterSequence !== requestedAfterSequence;
      } catch (loadError) {
        if (active) setError(errorMessage(loadError));
      } finally {
        loading = false;
      }
      if (active && hasMore) void load();
    }

    setChunks([]);
    setError(undefined);
    void load();
    const intervalId = window.setInterval(
      () => void load(),
      POLL_INTERVAL_MILLISECONDS,
    );
    return () => {
      active = false;
      window.clearInterval(intervalId);
    };
  }, [execution.id, onLoad]);

  return (
    <section
      aria-label={`Live activity for ${execution.id}`}
      className="activity-stream"
    >
      <h5>Live agent activity</h5>
      {error !== undefined ? (
        <p className="activity-error" role="status">
          Activity is temporarily unavailable: {error}
        </p>
      ) : chunks.length === 0 ? (
        <p className="activity-empty">
          Waiting for the agent to report activity.
        </p>
      ) : (
        <VirtualizedActivityList chunks={chunks} />
      )}
    </section>
  );
}

function VirtualizedActivityList({
  chunks,
}: Readonly<{ chunks: ExecutionActivityPage["chunks"] }>) {
  const [scrollTop, setScrollTop] = useState(0);
  const firstVisibleIndex = Math.floor(scrollTop / ROW_HEIGHT_PIXELS);
  const firstRow = Math.max(firstVisibleIndex - OVERSCAN_ROWS, 0);
  const visibleRows = Math.ceil(VIEWPORT_HEIGHT_PIXELS / ROW_HEIGHT_PIXELS);
  const lastRow = Math.min(
    firstVisibleIndex + visibleRows + OVERSCAN_ROWS,
    chunks.length,
  );
  const visibleChunks = chunks.slice(firstRow, lastRow);

  return (
    <div
      aria-live="polite"
      className="activity-viewport"
      onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
      role="log"
    >
      <ol
        style={{
          height: chunks.length * ROW_HEIGHT_PIXELS,
          position: "relative",
        }}
      >
        {visibleChunks.map((chunk, index) => (
          <li
            key={chunk.sequence}
            style={{
              height: ROW_HEIGHT_PIXELS,
              position: "absolute",
              top: (firstRow + index) * ROW_HEIGHT_PIXELS,
            }}
          >
            <span>{chunk.kind.replaceAll("_", " ")}</span>
            <p>{chunk.summary}</p>
            <time dateTime={chunk.recordedAt}>{chunk.recordedAt}</time>
          </li>
        ))}
      </ol>
    </div>
  );
}

function appendChunks(
  current: ExecutionActivityPage["chunks"],
  additions: ExecutionActivityPage["chunks"],
): ExecutionActivityPage["chunks"] {
  const knownSequences = new Set(current.map((chunk) => chunk.sequence));
  return [
    ...current,
    ...additions.filter((chunk) => !knownSequences.has(chunk.sequence)),
  ].slice(-MAX_RETAINED_RENDERED_CHUNKS);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
