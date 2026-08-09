import { useState, type FormEvent } from "react";

type BoardSetupProps = Readonly<{
  busy: boolean;
  onCreate: (input: CreateBoardInput) => Promise<void>;
  onBack: () => void;
}>;

export type CreateBoardInput = Readonly<{
  projectId: string;
  projectName: string;
  repositoryPath: string;
  baseRef: string;
  policySetId: string;
  boardId: string;
  boardName: string;
}>;

const initialCreateBoardInput: CreateBoardInput = {
  projectId: "",
  projectName: "",
  repositoryPath: "",
  baseRef: "main",
  policySetId: "standard",
  boardId: "",
  boardName: "",
};

export function BoardSetup({ busy, onCreate, onBack }: BoardSetupProps) {
  const [createInput, setCreateInput] = useState(initialCreateBoardInput);

  function updateCreateInput(field: keyof CreateBoardInput, value: string) {
    setCreateInput((current) => ({ ...current, [field]: value }));
  }

  async function createBoard(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onCreate(createInput);
  }

  return (
    <section aria-labelledby="board-setup-title" className="board-setup">
      <div>
        <p className="eyebrow">Local-first board</p>
        <h2 id="board-setup-title">Create a local board</h2>
        <p>
          Board creation currently uses the existing advanced project details.
          The board is stored on this device, and its state and dependency rules
          are enforced by the local daemon, not the browser.
        </p>
      </div>
      <div className="setup-grid">
        <form className="panel form-panel" onSubmit={createBoard}>
          <h3>Create a board</h3>
          <label>
            Project ID
            <input
              required
              value={createInput.projectId}
              onChange={(event) =>
                updateCreateInput("projectId", event.target.value)
              }
            />
          </label>
          <label>
            Project name
            <input
              required
              value={createInput.projectName}
              onChange={(event) =>
                updateCreateInput("projectName", event.target.value)
              }
            />
          </label>
          <label>
            Repository path
            <input
              required
              value={createInput.repositoryPath}
              onChange={(event) =>
                updateCreateInput("repositoryPath", event.target.value)
              }
            />
          </label>
          <div className="form-row">
            <label>
              Base ref
              <input
                required
                value={createInput.baseRef}
                onChange={(event) =>
                  updateCreateInput("baseRef", event.target.value)
                }
              />
            </label>
            <label>
              Policy set
              <input
                required
                value={createInput.policySetId}
                onChange={(event) =>
                  updateCreateInput("policySetId", event.target.value)
                }
              />
            </label>
          </div>
          <label>
            New board ID
            <input
              required
              value={createInput.boardId}
              onChange={(event) =>
                updateCreateInput("boardId", event.target.value)
              }
            />
          </label>
          <label>
            New board name
            <input
              required
              value={createInput.boardName}
              onChange={(event) =>
                updateCreateInput("boardName", event.target.value)
              }
            />
          </label>
          <button disabled={busy} type="submit">
            Create local board
          </button>
          <button disabled={busy} onClick={onBack} type="button">
            Back to your boards
          </button>
        </form>
      </div>
    </section>
  );
}
