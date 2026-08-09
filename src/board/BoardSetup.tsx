import { useState, type FormEvent } from "react";
import {
  FolderOpenIcon,
  FolderRootIcon,
  GitBranchIcon,
  InfoIcon,
} from "lucide-react";

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
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

import type {
  CloneGitHubRepositoryRequest,
  CreateLocalBoardRequest,
  RepositorySetup,
} from "./types";

export type RepositoryPicker = () => Promise<string | null>;

type BoardSetupProps = Readonly<{
  busy: boolean;
  repositoryPicker: RepositoryPicker;
  cloneDestinationPicker: RepositoryPicker;
  onInspectRepository: (repositoryPath: string) => Promise<RepositorySetup>;
  onCloneGitHubRepository: (
    input: CloneGitHubRepositoryRequest,
  ) => Promise<RepositorySetup>;
  onCreate: (input: CreateLocalBoardRequest) => Promise<void>;
  onBack: () => void;
}>;

const STANDARD_POLICY_SET_ID = "standard";

export function BoardSetup({
  busy,
  repositoryPicker,
  cloneDestinationPicker,
  onInspectRepository,
  onCloneGitHubRepository,
  onCreate,
  onBack,
}: BoardSetupProps) {
  const [repository, setRepository] = useState<RepositorySetup>();
  const [boardName, setBoardName] = useState("");
  const [baseRef, setBaseRef] = useState("");
  const policySetId = STANDARD_POLICY_SET_ID;
  const [repositorySource, setRepositorySource] = useState<"local" | "github">(
    "local",
  );
  const [githubRepositoryUrl, setGithubRepositoryUrl] = useState("");
  const [cloneDestination, setCloneDestination] = useState<string>();
  const [chooserMessage, setChooserMessage] = useState<string>();
  const [choosing, setChoosing] = useState(false);

  function applyRepositorySetup(inspectedRepository: RepositorySetup) {
    setRepository(inspectedRepository);
    setBoardName(inspectedRepository.suggestedBoardName);
    setBaseRef(inspectedRepository.baseRef);
  }

  async function chooseLocalRepository() {
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
      applyRepositorySetup(inspectedRepository);
    } catch (error) {
      setRepository(undefined);
      setChooserMessage(repositoryErrorMessage(error));
    } finally {
      setChoosing(false);
    }
  }

  async function chooseCloneDestination() {
    setChoosing(true);
    setChooserMessage(undefined);
    try {
      const selectedPath = await cloneDestinationPicker();
      if (selectedPath === null) {
        setChooserMessage(
          "No clone destination selected. No repository has been cloned.",
        );
        return;
      }
      setCloneDestination(selectedPath);
    } finally {
      setChoosing(false);
    }
  }

  async function cloneGitHubRepository() {
    if (!githubRepositoryUrl.trim() || cloneDestination === undefined) return;
    setChoosing(true);
    setChooserMessage(undefined);
    try {
      applyRepositorySetup(
        await onCloneGitHubRepository({
          repositoryUrl: githubRepositoryUrl.trim(),
          destinationParentPath: cloneDestination,
        }),
      );
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
        <h2 id="board-setup-title">Set up workspace</h2>
        <p>Link a GitHub repository or use one that is already on this Mac.</p>
      </div>
      <Card className="board-setup-card" size="sm">
        <CardHeader>
          <CardTitle as="h3">Workspace details</CardTitle>
        </CardHeader>
        <CardContent>
          <form
            className="form-panel"
            id="create-board-form"
            onSubmit={createBoard}
          >
            <FieldGroup>
              <Field>
                <FieldTitle id="repository-source-label">
                  Repository source
                </FieldTitle>
                <ToggleGroup
                  aria-labelledby="repository-source-label"
                  onValueChange={(value) => {
                    if (value === "local" || value === "github") {
                      setRepositorySource(value);
                      setChooserMessage(undefined);
                    }
                  }}
                  type="single"
                  value={repositorySource}
                >
                  <ToggleGroupItem value="local">
                    <FolderOpenIcon data-icon="inline-start" />
                    Use a local repository
                  </ToggleGroupItem>
                  <ToggleGroupItem value="github">
                    <GitBranchIcon data-icon="inline-start" />
                    Link a GitHub repository
                  </ToggleGroupItem>
                </ToggleGroup>
                <FieldDescription>
                  Both options keep the project on this device so Kanban can
                  work safely in separate task workspaces.
                </FieldDescription>
              </Field>
              {repositorySource === "local" ? (
                <Field>
                  <FieldTitle id="project-folder-label">
                    Project folder
                  </FieldTitle>
                  <Button
                    disabled={disabled}
                    onClick={() => void chooseLocalRepository()}
                    type="button"
                    variant="outline"
                  >
                    <FolderOpenIcon data-icon="inline-start" />
                    Choose project folder
                  </Button>
                  <FieldDescription>
                    Select the existing repository for the project you want to
                    coordinate.
                  </FieldDescription>
                </Field>
              ) : (
                <FieldGroup>
                  <Field>
                    <FieldLabel htmlFor="github-repository-url">
                      GitHub repository URL
                    </FieldLabel>
                    <Input
                      autoComplete="url"
                      disabled={disabled}
                      id="github-repository-url"
                      name="github-repository-url"
                      onChange={(event) =>
                        setGithubRepositoryUrl(event.target.value)
                      }
                      placeholder="https://github.com/owner/repository"
                      type="url"
                      value={githubRepositoryUrl}
                    />
                    <FieldDescription>
                      Use a GitHub HTTPS or SSH URL. Kanban uses your existing
                      Git sign-in or SSH setup and does not store credentials.
                    </FieldDescription>
                  </Field>
                  <Field>
                    <FieldTitle id="clone-destination-label">
                      Clone destination
                    </FieldTitle>
                    <Button
                      aria-describedby="clone-destination-description"
                      disabled={disabled}
                      onClick={() => void chooseCloneDestination()}
                      type="button"
                      variant="outline"
                    >
                      <FolderOpenIcon data-icon="inline-start" />
                      Choose clone destination
                    </Button>
                    <FieldDescription id="clone-destination-description">
                      {cloneDestination === undefined
                        ? "Choose the existing folder where Kanban should create the repository folder."
                        : `Kanban will create the repository folder in ${cloneDestination}.`}
                    </FieldDescription>
                  </Field>
                  <Button
                    disabled={
                      disabled ||
                      !githubRepositoryUrl.trim() ||
                      cloneDestination === undefined
                    }
                    onClick={() => void cloneGitHubRepository()}
                    type="button"
                    variant="outline"
                  >
                    <GitBranchIcon data-icon="inline-start" />
                    Clone repository
                  </Button>
                </FieldGroup>
              )}
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
                <p className="setup-reassurance">
                  You&apos;ll choose project agents and their defaults in
                  Settings after setup.
                </p>
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
            Set up workspace
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
  if (message.includes("could not clone")) {
    return "Kanban couldn't clone that repository. Check the URL and your Git access, then try again.";
  }
  if (
    message.includes("GitHub repository URL") ||
    message.includes("clone destination") ||
    message.includes("already exists")
  ) {
    return message;
  }
  return "Kanban couldn't use that folder as a project. Choose another project folder and try again.";
}
