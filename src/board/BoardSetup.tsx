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

const STANDARD_POLICY_SET_ID = "standard";

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
  const policySetId = STANDARD_POLICY_SET_ID;
  const [chooserMessage, setChooserMessage] = useState<string>();
  const [choosing, setChoosing] = useState(false);

  async function chooseRepository() {
    setChoosing(true);
    setChooserMessage(undefined);
    try {
      const selectedPath = await repositoryPicker();
      if (selectedPath === null) {
        setChooserMessage(
          "No project folder selected. No board has been created.",
        );
        return;
      }
      const inspectedRepository = await onInspectRepository(selectedPath);
      setRepository(inspectedRepository);
      setBoardName(inspectedRepository.suggestedBoardName);
      setBaseRef(inspectedRepository.baseRef);
    } catch (error) {
      setRepository(undefined);
      setChooserMessage(repositoryErrorMessage(error));
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
        <p>Choose a project folder and give your board a name.</p>
      </div>
      <Card className="board-setup-card" size="sm">
        <CardHeader>
          <CardTitle as="h3">Board details</CardTitle>
        </CardHeader>
        <CardContent>
          <form
            className="form-panel"
            id="create-board-form"
            onSubmit={createBoard}
          >
            <FieldGroup>
              <Field>
                <FieldTitle id="project-folder-label">
                  Project folder
                </FieldTitle>
                <Button
                  disabled={disabled}
                  onClick={() => void chooseRepository()}
                  type="button"
                  variant="outline"
                >
                  <FolderOpenIcon data-icon="inline-start" />
                  Choose project folder
                </Button>
                <FieldDescription>
                  Select the folder for the project you want to coordinate.
                </FieldDescription>
              </Field>
              {repository !== undefined && (
                <Alert className="repository-selection">
                  <FolderRootIcon aria-hidden="true" />
                  <AlertTitle>Selected project</AlertTitle>
                  <AlertDescription>
                    {repository.repositoryPath}
                  </AlertDescription>
                </Alert>
              )}
              {chooserMessage !== undefined && (
                <Alert className="setup-message">
                  <InfoIcon aria-hidden="true" />
                  <AlertTitle>Choose a project folder</AlertTitle>
                  <AlertDescription>{chooserMessage}</AlertDescription>
                </Alert>
              )}
              <Field>
                <FieldLabel htmlFor="board-name">Board name</FieldLabel>
                <Input
                  autoComplete="off"
                  disabled={disabled || repository === undefined}
                  id="board-name"
                  name="board-name"
                  onChange={(event) => setBoardName(event.target.value)}
                  required
                  value={boardName}
                />
              </Field>
            </FieldGroup>
            {repository !== undefined && (
              <>
                <Separator />
                <p className="setup-reassurance">
                  Kanban will prepare a separate workspace for each task.
                </p>
                <details className="advanced-disclosure">
                  <summary>Use a different starting point</summary>
                  <FieldGroup>
                    <Field>
                      <FieldLabel htmlFor="base-ref">
                        Start new work from
                      </FieldLabel>
                      <Input
                        autoComplete="off"
                        disabled={disabled}
                        id="base-ref"
                        name="base-ref"
                        onChange={(event) => setBaseRef(event.target.value)}
                        value={baseRef}
                      />
                      <FieldDescription>
                        Kanban normally uses your project&apos;s main line of
                        work. Change this only if your team asked you to.
                      </FieldDescription>
                    </Field>
                  </FieldGroup>
                </details>
              </>
            )}
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

function repositoryErrorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("repository root")) {
    return "Choose the top-level folder for your project, not a folder inside it.";
  }
  return "Kanban couldn't use that folder as a project. Choose another project folder and try again.";
}
