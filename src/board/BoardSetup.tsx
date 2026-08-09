import { useState, type FormEvent } from "react";
import { FolderOpenIcon, FolderRootIcon, InfoIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldTitle,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";

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
        <h2 id="board-setup-title">Create a board</h2>
        <p>
          Choose the local Git repository whose work you want to coordinate.
          Nothing is created until you confirm.
        </p>
      </div>
      <Card className="board-setup-card">
        <CardHeader>
          <CardTitle>Choose a repository</CardTitle>
        </CardHeader>
        <CardContent>
          <form
            className="form-panel"
            id="create-board-form"
            onSubmit={createBoard}
          >
            <FieldGroup>
              <Field>
                <FieldTitle id="repository-label">Repository</FieldTitle>
                <Button
                  aria-describedby="repository-label"
                  disabled={disabled}
                  onClick={() => void chooseRepository()}
                  type="button"
                  variant="outline"
                >
                  <FolderOpenIcon data-icon="inline-start" />
                  Choose repository
                </Button>
                <FieldDescription>
                  Select the Git repository that this board coordinates.
                </FieldDescription>
              </Field>
              {repository !== undefined && (
                <Alert className="repository-selection">
                  <FolderRootIcon aria-hidden="true" />
                  <AlertTitle>Git root</AlertTitle>
                  <AlertDescription>
                    {repository.repositoryPath}
                  </AlertDescription>
                </Alert>
              )}
              {chooserMessage !== undefined && (
                <Alert className="setup-message">
                  <InfoIcon aria-hidden="true" />
                  <AlertTitle>Repository selection</AlertTitle>
                  <AlertDescription>{chooserMessage}</AlertDescription>
                </Alert>
              )}
              <Field>
                <FieldLabel htmlFor="board-name">Board name</FieldLabel>
                <Input
                  disabled={disabled || repository === undefined}
                  id="board-name"
                  onChange={(event) => setBoardName(event.target.value)}
                  required
                  value={boardName}
                />
                <FieldDescription>
                  Suggested from the repository folder.
                </FieldDescription>
              </Field>
            </FieldGroup>
            <Separator />
            <section aria-label="Safe defaults" className="safe-defaults">
              <p className="eyebrow">Safe defaults</p>
              <p>
                Base branch: {repository?.baseRef ?? "—"} · Policy: Standard
              </p>
            </section>
            <details className="advanced-disclosure">
              <summary>Advanced setup</summary>
              <FieldGroup>
                <Field>
                  <FieldLabel htmlFor="base-ref">Base branch</FieldLabel>
                  <Input
                    disabled={disabled || repository === undefined}
                    id="base-ref"
                    onChange={(event) => setBaseRef(event.target.value)}
                    value={baseRef}
                  />
                  <FieldDescription>
                    Changing the base branch changes where task worktrees start.
                  </FieldDescription>
                </Field>
                <Field>
                  <FieldLabel htmlFor="policy-set">Policy</FieldLabel>
                  <Input
                    disabled={disabled || repository === undefined}
                    id="policy-set"
                    onChange={(event) => setPolicySetId(event.target.value)}
                    value={policySetId}
                  />
                  <FieldDescription>
                    Changing the policy changes which local actions need
                    approval.
                  </FieldDescription>
                </Field>
              </FieldGroup>
            </details>
          </form>
        </CardContent>
        <CardFooter className="form-actions">
          <Button
            disabled={disabled}
            onClick={onBack}
            type="button"
            variant="ghost"
          >
            Back to your boards
          </Button>
          <Button
            disabled={disabled || repository === undefined || !boardName.trim()}
            form="create-board-form"
            type="submit"
          >
            Create board
          </Button>
        </CardFooter>
      </Card>
    </section>
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
