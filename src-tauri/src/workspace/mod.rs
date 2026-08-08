mod git_cli;

use std::{
    error::Error,
    fmt, fs, io,
    path::{Component, Path, PathBuf},
};

use crate::domain::{Execution, Project, WorkItemId};

pub use git_cli::GitError;

use git_cli::GitCli;

const TASK_BRANCH_PREFIX: &str = "kanban";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DependencySharingStrategy {
    IsolatedInstall,
    ManagedSharedCache,
    ExplicitProjectApprovedLink { approval_id: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathAccess {
    Read,
    Write,
}

impl fmt::Display for PathAccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => write!(formatter, "read"),
            Self::Write => write!(formatter, "write"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceProvisionRequest {
    pub work_item_id: WorkItemId,
    pub dependency_sharing: DependencySharingStrategy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceAssignment {
    work_item_id: WorkItemId,
    branch_name: String,
    path: PathBuf,
    dependency_sharing: DependencySharingStrategy,
}

impl WorkspaceAssignment {
    pub fn work_item_id(&self) -> &WorkItemId {
        &self.work_item_id
    }

    pub fn branch_name(&self) -> &str {
        &self.branch_name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn dependency_sharing(&self) -> &DependencySharingStrategy {
        &self.dependency_sharing
    }
}

pub struct WorkspaceManager {
    repository_root: PathBuf,
    workspace_root: PathBuf,
    base_ref: String,
    git: GitCli,
}

impl WorkspaceManager {
    pub fn new(
        project: &Project,
        workspace_root: impl AsRef<Path>,
    ) -> Result<Self, WorkspaceError> {
        let declared_repository_path =
            resolved_path_for_creation(Path::new(&project.repository_path))?;
        let git = GitCli;
        let detected_repository_path =
            resolved_existing_path(&git.repository_root(&declared_repository_path)?)?;

        if declared_repository_path != detected_repository_path {
            return Err(WorkspaceError::ProjectRepositoryMustBeRoot {
                declared_path: declared_repository_path,
                detected_path: detected_repository_path,
            });
        }
        git.validate_revision(&detected_repository_path, &project.base_ref)?;

        let workspace_root = resolved_path_for_creation(workspace_root.as_ref())?;
        reject_overlapping_roots(&detected_repository_path, &workspace_root)?;
        fs::create_dir_all(&workspace_root)?;
        let workspace_root = resolved_existing_path(&workspace_root)?;
        reject_overlapping_roots(&detected_repository_path, &workspace_root)?;

        Ok(Self {
            repository_root: detected_repository_path,
            workspace_root,
            base_ref: project.base_ref.clone(),
            git,
        })
    }

    pub fn provision(
        &self,
        request: WorkspaceProvisionRequest,
    ) -> Result<WorkspaceAssignment, WorkspaceError> {
        let workspace_name = workspace_name(&request.work_item_id)?.to_owned();
        let branch_name = format!("{TASK_BRANCH_PREFIX}/{workspace_name}");
        self.git
            .validate_branch_name(&self.repository_root, &branch_name)?;
        let assignment = WorkspaceAssignment {
            work_item_id: request.work_item_id,
            branch_name,
            path: self.workspace_root.join(&workspace_name),
            dependency_sharing: request.dependency_sharing,
        };
        let worktrees = self.git.worktrees(&self.repository_root)?;

        for existing_worktree in &worktrees {
            if paths_match(&existing_worktree.path, &assignment.path)? {
                let existing_branch = existing_worktree
                    .branch
                    .clone()
                    .unwrap_or_else(|| "detached HEAD".to_owned());
                if existing_branch != format!("refs/heads/{}", assignment.branch_name) {
                    return Err(WorkspaceError::WorkspacePathAssignedToDifferentBranch {
                        path: assignment.path,
                        existing_branch,
                        requested_branch: assignment.branch_name,
                    });
                }

                self.verify_assignment_identity(&assignment)?;
                return Ok(assignment);
            }
        }

        if let Some(existing_worktree) = worktrees.iter().find(|worktree| {
            worktree.branch.as_deref() == Some(&format!("refs/heads/{}", assignment.branch_name))
        }) {
            return Err(WorkspaceError::BranchAlreadyAssigned {
                branch_name: assignment.branch_name,
                path: resolved_path_for_creation(&existing_worktree.path)?,
            });
        }

        let branch_exists = self
            .git
            .branch_exists(&self.repository_root, &assignment.branch_name)?;
        if branch_exists
            && !self.git.references_match(
                &self.repository_root,
                &assignment.branch_name,
                &self.base_ref,
            )?
        {
            return Err(WorkspaceError::RecoveryBranchDiverged {
                branch_name: assignment.branch_name,
                base_ref: self.base_ref.clone(),
            });
        }

        prepare_workspace_path(&assignment.path)?;
        if branch_exists {
            self.git.attach_existing_branch(
                &self.repository_root,
                &assignment.path,
                &assignment.branch_name,
            )?;
        } else {
            self.git.create_worktree(
                &self.repository_root,
                &assignment.path,
                &assignment.branch_name,
                &self.base_ref,
            )?;
        }

        self.verify_assignment_identity(&assignment)?;
        Ok(assignment)
    }

    pub fn verify_execution_workspace(
        &self,
        execution: &Execution,
        assignment: &WorkspaceAssignment,
    ) -> Result<(), WorkspaceError> {
        self.verify_assignment_belongs_to_manager(assignment)?;
        if execution.work_item_id != assignment.work_item_id {
            return Err(WorkspaceError::ExecutionWorkItemMismatch {
                execution_work_item_id: execution.work_item_id.clone(),
                assignment_work_item_id: assignment.work_item_id.clone(),
            });
        }

        let execution_workspace = resolved_existing_path(Path::new(&execution.workspace_path))?;
        if execution_workspace != assignment.path {
            return Err(WorkspaceError::ExecutionWorkspaceMismatch {
                execution_path: execution_workspace,
                assignment_path: assignment.path.clone(),
            });
        }

        self.verify_assignment_identity(assignment)
    }

    pub fn authorize_path(
        &self,
        assignment: &WorkspaceAssignment,
        requested_path: &Path,
        access: PathAccess,
    ) -> Result<(), WorkspaceError> {
        self.verify_assignment_belongs_to_manager(assignment)?;
        let requested_path = resolved_path_for_creation(requested_path)?;
        if requested_path.starts_with(&assignment.path) {
            return Ok(());
        }

        if access == PathAccess::Write && requested_path.starts_with(&self.repository_root) {
            return Err(WorkspaceError::BaseRepositoryMutationDenied {
                path: requested_path,
            });
        }

        Err(WorkspaceError::UndeclaredPathAccessDenied {
            path: requested_path,
            access,
        })
    }

    fn verify_assignment_identity(
        &self,
        assignment: &WorkspaceAssignment,
    ) -> Result<(), WorkspaceError> {
        self.verify_assignment_belongs_to_manager(assignment)?;
        let (actual_root, actual_branch) = self.git.worktree_identity(&assignment.path)?;
        let actual_root = resolved_existing_path(&actual_root)?;
        if actual_root != assignment.path || actual_branch != assignment.branch_name {
            return Err(WorkspaceError::WorktreeIdentityMismatch {
                expected_path: assignment.path.clone(),
                actual_path: actual_root,
                expected_branch: assignment.branch_name.clone(),
                actual_branch,
            });
        }

        Ok(())
    }

    fn verify_assignment_belongs_to_manager(
        &self,
        assignment: &WorkspaceAssignment,
    ) -> Result<(), WorkspaceError> {
        if !assignment.path.starts_with(&self.workspace_root) {
            return Err(WorkspaceError::WorkspaceAssignmentOutsideManagedRoot {
                assignment_path: assignment.path.clone(),
                workspace_root: self.workspace_root.clone(),
            });
        }

        Ok(())
    }
}

fn workspace_name(work_item_id: &WorkItemId) -> Result<&str, WorkspaceError> {
    let name = work_item_id.0.as_str();
    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        || is_reserved_windows_directory_name(name)
    {
        return Err(WorkspaceError::UnsafeWorkItemId {
            work_item_id: work_item_id.clone(),
        });
    }

    Ok(name)
}

fn is_reserved_windows_directory_name(name: &str) -> bool {
    let normalized_name = name.to_ascii_uppercase();
    matches!(normalized_name.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            normalized_name.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
        })
}

fn prepare_workspace_path(path: &Path) -> Result<(), WorkspaceError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(WorkspaceError::WorkspacePathOccupied {
                path: path.to_path_buf(),
            })
        }
        Ok(_) if fs::read_dir(path)?.next().is_some() => {
            Err(WorkspaceError::WorkspacePathOccupied {
                path: path.to_path_buf(),
            })
        }
        Ok(_) => {
            fs::remove_dir(path)?;
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn reject_overlapping_roots(
    repository_root: &Path,
    workspace_root: &Path,
) -> Result<(), WorkspaceError> {
    if workspace_root.starts_with(repository_root) || repository_root.starts_with(workspace_root) {
        return Err(WorkspaceError::WorkspaceRootOverlapsRepository {
            repository_path: repository_root.to_path_buf(),
            workspace_path: workspace_root.to_path_buf(),
        });
    }

    Ok(())
}

fn paths_match(left: &Path, right: &Path) -> Result<bool, WorkspaceError> {
    Ok(resolved_path_for_creation(left)? == resolved_path_for_creation(right)?)
}

fn resolved_existing_path(path: &Path) -> Result<PathBuf, WorkspaceError> {
    path.canonicalize().map_err(WorkspaceError::FileSystem)
}

fn resolved_path_for_creation(path: &Path) -> Result<PathBuf, WorkspaceError> {
    let mut candidate = normalized_absolute_path(path)?;
    let mut missing_components = Vec::new();

    loop {
        match candidate.canonicalize() {
            Ok(existing_path) => {
                let mut resolved_path = existing_path;
                for component in missing_components.iter().rev() {
                    resolved_path.push(component);
                }
                return Ok(resolved_path);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let component = candidate.file_name().ok_or_else(|| {
                    WorkspaceError::FileSystem(io::Error::new(
                        io::ErrorKind::NotFound,
                        "could not resolve an existing parent directory",
                    ))
                })?;
                missing_components.push(component.to_owned());
                candidate = candidate
                    .parent()
                    .ok_or_else(|| {
                        WorkspaceError::FileSystem(io::Error::new(
                            io::ErrorKind::NotFound,
                            "could not resolve an existing parent directory",
                        ))
                    })?
                    .to_path_buf();
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn normalized_absolute_path(path: &Path) -> Result<PathBuf, WorkspaceError> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(component) => normalized.push(component),
        }
    }

    Ok(normalized)
}

#[derive(Debug)]
pub enum WorkspaceError {
    FileSystem(io::Error),
    Git(GitError),
    ProjectRepositoryMustBeRoot {
        declared_path: PathBuf,
        detected_path: PathBuf,
    },
    WorkspaceRootOverlapsRepository {
        repository_path: PathBuf,
        workspace_path: PathBuf,
    },
    UnsafeWorkItemId {
        work_item_id: WorkItemId,
    },
    WorkspacePathAssignedToDifferentBranch {
        path: PathBuf,
        existing_branch: String,
        requested_branch: String,
    },
    BranchAlreadyAssigned {
        branch_name: String,
        path: PathBuf,
    },
    WorkspacePathOccupied {
        path: PathBuf,
    },
    RecoveryBranchDiverged {
        branch_name: String,
        base_ref: String,
    },
    ExecutionWorkItemMismatch {
        execution_work_item_id: WorkItemId,
        assignment_work_item_id: WorkItemId,
    },
    ExecutionWorkspaceMismatch {
        execution_path: PathBuf,
        assignment_path: PathBuf,
    },
    WorktreeIdentityMismatch {
        expected_path: PathBuf,
        actual_path: PathBuf,
        expected_branch: String,
        actual_branch: String,
    },
    WorkspaceAssignmentOutsideManagedRoot {
        assignment_path: PathBuf,
        workspace_root: PathBuf,
    },
    BaseRepositoryMutationDenied {
        path: PathBuf,
    },
    UndeclaredPathAccessDenied {
        path: PathBuf,
        access: PathAccess,
    },
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileSystem(error) => write!(formatter, "workspace filesystem error: {error}"),
            Self::Git(error) => write!(formatter, "workspace Git error: {error}"),
            Self::ProjectRepositoryMustBeRoot {
                declared_path,
                detected_path,
            } => write!(
                formatter,
                "declared project path {} must be the repository root {}, not a subdirectory",
                declared_path.display(),
                detected_path.display()
            ),
            Self::WorkspaceRootOverlapsRepository {
                repository_path,
                workspace_path,
            } => write!(
                formatter,
                "workspace root {} must not overlap project repository {}",
                workspace_path.display(),
                repository_path.display()
            ),
            Self::UnsafeWorkItemId { work_item_id } => write!(
                formatter,
                "work item id {} is not safe for a workspace directory",
                work_item_id.0
            ),
            Self::WorkspacePathAssignedToDifferentBranch {
                path,
                existing_branch,
                requested_branch,
            } => write!(
                formatter,
                "workspace path {} already belongs to branch {}; expected {}",
                path.display(),
                existing_branch,
                requested_branch
            ),
            Self::BranchAlreadyAssigned { branch_name, path } => write!(
                formatter,
                "task branch {branch_name} is already assigned to workspace {}",
                path.display()
            ),
            Self::WorkspacePathOccupied { path } => write!(
                formatter,
                "workspace path {} is occupied and cannot be recovered safely",
                path.display()
            ),
            Self::RecoveryBranchDiverged {
                branch_name,
                base_ref,
            } => write!(
                formatter,
                "recovery branch {branch_name} no longer matches declared base ref {base_ref}"
            ),
            Self::ExecutionWorkItemMismatch {
                execution_work_item_id,
                assignment_work_item_id,
            } => write!(
                formatter,
                "execution work item {} does not match workspace work item {}",
                execution_work_item_id.0, assignment_work_item_id.0
            ),
            Self::ExecutionWorkspaceMismatch {
                execution_path,
                assignment_path,
            } => write!(
                formatter,
                "execution workspace {} does not match assigned workspace {}",
                execution_path.display(),
                assignment_path.display()
            ),
            Self::WorktreeIdentityMismatch {
                expected_path,
                actual_path,
                expected_branch,
                actual_branch,
            } => write!(
                formatter,
                "worktree identity mismatch: expected {} on {}, found {} on {}",
                expected_branch,
                expected_path.display(),
                actual_branch,
                actual_path.display()
            ),
            Self::WorkspaceAssignmentOutsideManagedRoot {
                assignment_path,
                workspace_root,
            } => write!(
                formatter,
                "workspace assignment {} is outside this manager's declared root {}",
                assignment_path.display(),
                workspace_root.display()
            ),
            Self::BaseRepositoryMutationDenied { path } => write!(
                formatter,
                "writing the base repository path {} is denied",
                path.display()
            ),
            Self::UndeclaredPathAccessDenied { path, access } => write!(
                formatter,
                "{} access to undeclared path {} is denied",
                access,
                path.display()
            ),
        }
    }
}

impl Error for WorkspaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FileSystem(error) => Some(error),
            Self::Git(error) => Some(error),
            Self::ProjectRepositoryMustBeRoot { .. }
            | Self::WorkspaceRootOverlapsRepository { .. }
            | Self::UnsafeWorkItemId { .. }
            | Self::WorkspacePathAssignedToDifferentBranch { .. }
            | Self::BranchAlreadyAssigned { .. }
            | Self::WorkspacePathOccupied { .. }
            | Self::RecoveryBranchDiverged { .. }
            | Self::ExecutionWorkItemMismatch { .. }
            | Self::ExecutionWorkspaceMismatch { .. }
            | Self::WorktreeIdentityMismatch { .. }
            | Self::WorkspaceAssignmentOutsideManagedRoot { .. }
            | Self::BaseRepositoryMutationDenied { .. }
            | Self::UndeclaredPathAccessDenied { .. } => None,
        }
    }
}

impl From<io::Error> for WorkspaceError {
    fn from(error: io::Error) -> Self {
        Self::FileSystem(error)
    }
}

impl From<GitError> for WorkspaceError {
    fn from(error: GitError) -> Self {
        Self::Git(error)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod path_tests {
    use std::{env, path::Path};

    use super::{WorkspaceError, normalized_absolute_path, resolved_path_for_creation};

    #[test]
    fn normalizes_relative_paths_before_applying_workspace_boundaries() {
        assert_eq!(
            normalized_absolute_path(Path::new("workspace/../workspace"))
                .expect("relative paths should normalize"),
            env::current_dir()
                .expect("current directory should resolve")
                .join("workspace")
        );
    }

    #[cfg(unix)]
    #[test]
    fn preserves_non_absence_filesystem_errors_during_path_resolution() {
        assert!(matches!(
            resolved_path_for_creation(Path::new("/dev/null/child")),
            Err(WorkspaceError::FileSystem(_))
        ));
    }

    #[test]
    fn reserves_windows_device_names_even_when_running_on_another_platform() {
        assert!(super::is_reserved_windows_directory_name("con"));
        assert!(super::is_reserved_windows_directory_name("LPT9"));
        assert!(!super::is_reserved_windows_directory_name("task-1"));
    }
}
