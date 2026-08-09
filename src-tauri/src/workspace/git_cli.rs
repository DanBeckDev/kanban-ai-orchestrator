use std::{
    borrow::Cow,
    env,
    ffi::{OsStr, OsString},
    fmt, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const REPOSITORY_CONTEXT_ENVIRONMENT_VARIABLES: &[&str] = &[
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CEILING_DIRECTORIES",
    "GIT_COMMON_DIR",
    "GIT_DIR",
    "GIT_DISCOVERY_ACROSS_FILESYSTEM",
    "GIT_INDEX_FILE",
    "GIT_NAMESPACE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_WORK_TREE",
];

#[derive(Default)]
pub(super) struct GitCli;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GitWorktree {
    pub path: PathBuf,
    pub branch: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct GitReviewArtifacts {
    pub head_commit: Option<String>,
    pub committed_diff_stat: Option<String>,
    pub working_diff_stat: Option<String>,
}

impl GitCli {
    pub fn repository_root(&self, directory: &Path) -> Result<PathBuf, GitError> {
        Ok(PathBuf::from(self.successful_text(
            directory,
            "resolve the project repository",
            &["rev-parse".into(), "--show-toplevel".into()],
        )?))
    }

    pub fn validate_branch_name(&self, directory: &Path, branch: &str) -> Result<(), GitError> {
        self.successful_text(
            directory,
            "validate the task branch name",
            &["check-ref-format".into(), "--branch".into(), branch.into()],
        )?;
        Ok(())
    }

    pub fn worktrees(&self, directory: &Path) -> Result<Vec<GitWorktree>, GitError> {
        parse_worktrees(&self.successful_text(
            directory,
            "list registered worktrees",
            &[
                "worktree".into(),
                "list".into(),
                "--porcelain".into(),
                "-z".into(),
            ],
        )?)
    }

    pub fn branch_exists(&self, directory: &Path, branch: &str) -> Result<bool, GitError> {
        let output = self.output(
            directory,
            &[
                "show-ref".into(),
                "--verify".into(),
                "--quiet".into(),
                format!("refs/heads/{branch}").into(),
            ],
        )?;

        match output.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(GitError::command_failed(
                "check whether the task branch exists",
                output,
            )),
        }
    }

    pub fn references_match(
        &self,
        directory: &Path,
        left: &str,
        right: &str,
    ) -> Result<bool, GitError> {
        Ok(self.reference_commit(directory, left)? == self.reference_commit(directory, right)?)
    }

    pub fn validate_revision(&self, directory: &Path, revision: &str) -> Result<(), GitError> {
        self.reference_commit(directory, revision).map(|_| ())
    }

    pub fn create_worktree(
        &self,
        directory: &Path,
        path: &Path,
        branch: &str,
        base_ref: &str,
    ) -> Result<(), GitError> {
        self.successful_text(
            directory,
            "create the task worktree",
            &[
                "worktree".into(),
                "add".into(),
                "-b".into(),
                branch.into(),
                git_path_argument(path).as_ref().as_os_str().to_owned(),
                base_ref.into(),
            ],
        )?;
        Ok(())
    }

    pub fn clone(
        &self,
        destination_parent: &Path,
        repository_url: &str,
        destination: &Path,
    ) -> Result<(), GitError> {
        self.successful_text(
            destination_parent,
            "clone the selected GitHub repository",
            &[
                "clone".into(),
                "--no-recurse-submodules".into(),
                "--".into(),
                repository_url.into(),
                git_path_argument(destination)
                    .as_ref()
                    .as_os_str()
                    .to_owned(),
            ],
        )?;
        Ok(())
    }

    pub fn attach_existing_branch(
        &self,
        directory: &Path,
        path: &Path,
        branch: &str,
    ) -> Result<(), GitError> {
        self.successful_text(
            directory,
            "attach the recovered task branch to its worktree",
            &[
                "worktree".into(),
                "add".into(),
                git_path_argument(path).as_ref().as_os_str().to_owned(),
                branch.into(),
            ],
        )?;
        Ok(())
    }

    pub fn worktree_identity(&self, directory: &Path) -> Result<(PathBuf, String), GitError> {
        let root = PathBuf::from(self.successful_text(
            directory,
            "resolve the task worktree root",
            &["rev-parse".into(), "--show-toplevel".into()],
        )?);
        let branch = self.successful_text(
            directory,
            "resolve the task worktree branch",
            &[
                "symbolic-ref".into(),
                "--quiet".into(),
                "--short".into(),
                "HEAD".into(),
            ],
        )?;

        Ok((root, branch))
    }

    pub fn review_artifacts(
        &self,
        directory: &Path,
        base_ref: &str,
    ) -> Result<GitReviewArtifacts, GitError> {
        let has_committed_changes = !self.references_match(directory, base_ref, "HEAD")?;
        let committed_diff_stat = has_committed_changes
            .then(|| {
                self.successful_text(
                    directory,
                    "summarize committed task changes",
                    &[
                        "diff".into(),
                        "--stat".into(),
                        "--no-ext-diff".into(),
                        "--stat-width=80".into(),
                        "--stat-count=20".into(),
                        format!("{base_ref}...HEAD").into(),
                    ],
                )
            })
            .transpose()?;
        let working_diff_stat = self.successful_text(
            directory,
            "summarize uncommitted task changes",
            &[
                "diff".into(),
                "--stat".into(),
                "--no-ext-diff".into(),
                "--stat-width=80".into(),
                "--stat-count=20".into(),
                "HEAD".into(),
            ],
        )?;

        Ok(GitReviewArtifacts {
            head_commit: has_committed_changes
                .then(|| self.reference_commit(directory, "HEAD"))
                .transpose()?,
            committed_diff_stat: committed_diff_stat.filter(|stat| !stat.is_empty()),
            working_diff_stat: (!working_diff_stat.is_empty()).then_some(working_diff_stat),
        })
    }

    pub(super) fn successful_text(
        &self,
        directory: &Path,
        operation: &'static str,
        arguments: &[OsString],
    ) -> Result<String, GitError> {
        let output = self.output(directory, arguments)?;
        if !output.status.success() {
            return Err(GitError::command_failed(operation, output));
        }

        String::from_utf8(output.stdout)
            .map(|text| text.trim_end().to_owned())
            .map_err(|_| GitError::NonUtf8Output { operation })
    }

    fn reference_commit(&self, directory: &Path, reference: &str) -> Result<String, GitError> {
        self.successful_text(
            directory,
            "resolve a Git reference",
            &[
                "rev-parse".into(),
                "--verify".into(),
                "--end-of-options".into(),
                format!("{reference}^{{commit}}").into(),
            ],
        )
    }

    pub(super) fn output(
        &self,
        directory: &Path,
        arguments: &[OsString],
    ) -> Result<Output, GitError> {
        self.command(directory, arguments)
            .output()
            .map_err(GitError::CommandIo)
    }

    pub(super) fn command(&self, directory: &Path, arguments: &[OsString]) -> Command {
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(git_path_argument(directory).as_ref())
            .args(arguments);
        for variable in REPOSITORY_CONTEXT_ENVIRONMENT_VARIABLES {
            command.env_remove(variable);
        }
        for (variable, _) in env::vars_os() {
            if is_repository_context_environment_variable(&variable) {
                command.env_remove(variable);
            }
        }
        command
    }
}

#[cfg(windows)]
fn git_path_argument(path: &Path) -> Cow<'_, Path> {
    let path_text = path.to_string_lossy();

    if let Some(network_path) = path_text.strip_prefix(r"\\?\UNC\") {
        Cow::Owned(PathBuf::from(format!(r"\\{network_path}")))
    } else if let Some(disk_path) = path_text.strip_prefix(r"\\?\") {
        Cow::Owned(PathBuf::from(disk_path))
    } else {
        Cow::Borrowed(path)
    }
}

#[cfg(not(windows))]
fn git_path_argument(path: &Path) -> Cow<'_, Path> {
    Cow::Borrowed(path)
}

fn is_repository_context_environment_variable(variable: &OsStr) -> bool {
    REPOSITORY_CONTEXT_ENVIRONMENT_VARIABLES
        .iter()
        .any(|known_variable| variable == OsStr::new(known_variable))
        || variable.to_string_lossy().starts_with("GIT_CONFIG_")
}

fn parse_worktrees(output: &str) -> Result<Vec<GitWorktree>, GitError> {
    output
        .split("\0\0")
        .filter(|record| !record.is_empty())
        .map(|record| {
            let mut lines = record.split('\0');
            let path = lines
                .next()
                .and_then(|line| line.strip_prefix("worktree "))
                .map(PathBuf::from)
                .ok_or(GitError::MalformedWorktreeList)?;
            let branch = lines.find_map(|line| {
                line.strip_prefix("branch ")
                    .map(|reference| reference.to_owned())
            });

            Ok(GitWorktree { path, branch })
        })
        .collect()
}

#[derive(Debug)]
pub enum GitError {
    CommandIo(io::Error),
    CommandFailed {
        operation: &'static str,
        exit_code: Option<i32>,
        stderr: String,
    },
    NonUtf8Output {
        operation: &'static str,
    },
    MalformedWorktreeList,
}

impl GitError {
    pub(super) fn command_failed(operation: &'static str, output: Output) -> Self {
        Self::CommandFailed {
            operation,
            exit_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        }
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandIo(error) => write!(formatter, "could not start Git: {error}"),
            Self::CommandFailed {
                operation,
                exit_code,
                stderr,
            } => write!(
                formatter,
                "Git could not {operation} (exit code {}): {stderr}",
                exit_code.map_or_else(|| "unknown".to_owned(), |code| code.to_string())
            ),
            Self::NonUtf8Output { operation } => {
                write!(
                    formatter,
                    "Git returned non-UTF-8 output while trying to {operation}"
                )
            }
            Self::MalformedWorktreeList => {
                write!(formatter, "Git returned an invalid worktree list")
            }
        }
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CommandIo(error) => Some(error),
            Self::CommandFailed { .. }
            | Self::NonUtf8Output { .. }
            | Self::MalformedWorktreeList => None,
        }
    }
}

#[cfg(test)]
mod tests;
