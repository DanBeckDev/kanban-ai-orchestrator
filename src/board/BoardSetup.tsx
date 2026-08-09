import { useState, type FormEvent } from "react";

import type { CreateLocalBoardRequest, RepositorySetup } from "./types";

export type RepositoryPicker = () => Promise<string | null>;

type BoardSetupProps = Readonly<{
  busy: boolean;
  repositoryPicker: RepositoryPicker;
  onInspectRepository: (repositoryPath: string) => Promise<RepositorySetup>;
  onCreate: (input: CreateLocalBoardRequest) => Promise<void>;
  onBack: () => void;
}>;

const standardPolicySetId = "standard";

export function BoardSetup({
  busy,
  repositoryPicker,
  onInspectRepository,
  onCreate,
  onBack,
}: BoardSetupProps) {
  const [repository, setRepository] = useState<RepositorySetup>();
  const [boardName, setBoardName] = useState("");
  const [baseRef, setBaseRef] = useState("");
  const [policySetId, setPolicySetId] = useState(standardPolicySetId);
  const [chooserMessage, setChooserMessage] = useState<string>();
  const [choosing, setChoosing] = useState(false);

  async function chooseRepository() {
    setChoosing(true);
    setChooserMessage(undefined);
    try {
      const selectedPath = await repositoryPicker();
      if (selectedPath === null) {
        setChooserMessage("No repository selected. No board has been created.");
        return;
      }
      const inspectedRepository = await onInspectRepository(selectedPath);
      setRepository(inspectedRepository);
      setBoardName(inspectedRepository.suggestedBoardName);
      setBaseRef(inspectedRepository.baseRef);
    } catch (error) {
      setRepository(undefined);
      setChooserMessage(errorMessage(error));
    } finally {
      setChoosing(false);
    }
  }

  async function createBoard(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (repository === undefined) return;
    await onCreate({
      name: boardName,
      repositoryPath: repository.repositoryPath,
      baseRef,
      policySetId,
    });
  }

  const disabled = busy || choosing;

  return (
    <section aria-labelledby="board-setup-title" className="board-setup">
      <div>
        <p className="eyebrow">Local-first board</p>
        <h2 id="board-setup-title">Create a board</h2>
        <p>
          Choose the local Git repository whose work you want to coordinate.
          Nothing is created until you confirm.
        </p>
      </div>
      <form className="panel form-panel" onSubmit={createBoard}>
        <div>
          <p id="repository-label">Repository</p>
          <button
            aria-describedby="repository-label"
            disabled={disabled}
            onClick={() => void chooseRepository()}
            type="button"
          >
            Choose repository
          </button>
        </div>
        {repository !== undefined && (
          <p className="repository-selection">
            <span>{repository.repositoryPath}</span> <strong>Git root</strong>
          </p>
        )}
        {chooserMessage !== undefined && (
          <p aria-live="polite" className="setup-message" role="status">
            {chooserMessage}
          </p>
        )}
        <label>
          Board name
          <input
            disabled={disabled || repository === undefined}
            onChange={(event) => setBoardName(event.target.value)}
            required
            value={boardName}
          />
        </label>
        <p className="form-hint">Suggested from the repository folder.</p>
        <section aria-label="Safe defaults" className="safe-defaults">
          <h3>Safe defaults</h3>
          <p>Base branch: {repository?.baseRef ?? "—"} · Policy: Standard</p>
        </section>
        <details>
          <summary>Advanced setup</summary>
          <label>
            Base branch
            <input
              disabled={disabled || repository === undefined}
              onChange={(event) => setBaseRef(event.target.value)}
              value={baseRef}
            />
          </label>
          <p className="form-hint">
            Changing the base branch changes where task worktrees start.
          </p>
          <label>
            Policy
            <input
              disabled={disabled || repository === undefined}
              onChange={(event) => setPolicySetId(event.target.value)}
              value={policySetId}
            />
          </label>
          <p className="form-hint">
            Changing the policy changes which local actions need approval.
          </p>
        </details>
        <div className="form-actions">
          <button disabled={disabled} onClick={onBack} type="button">
            Back to your boards
          </button>
          <button
            disabled={disabled || repository === undefined || !boardName.trim()}
            type="submit"
          >
            Create board
          </button>
        </div>
      </form>
    </section>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
