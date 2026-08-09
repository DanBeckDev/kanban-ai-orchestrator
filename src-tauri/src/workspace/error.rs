use std::{error::Error, fmt, io, path::PathBuf};

use crate::domain::WorkItemId;

use super::{GitError, PathAccess};

#[derive(Debug)]
pub enum WorkspaceError {
    FileSystem(io::Error),
    Git(GitError),
    ProjectRepositoryMustBeRoot {
        declared_path: PathBuf,
        detected_path: PathBuf,
    },
    InvalidGitHubRepositoryUrl,
    CloneDestinationMustBeDirectory {
        path: PathBuf,
    },
    CloneDestinationOccupied {
        path: PathBuf,
    },
    GitHubCloneFailed,
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
            Self::InvalidGitHubRepositoryUrl => write!(
                formatter,
                "enter a GitHub repository URL such as https://github.com/owner/repository"
            ),
            Self::CloneDestinationMustBeDirectory { path } => write!(
                formatter,
                "choose an existing folder for the clone destination, not {}",
                path.display()
            ),
            Self::CloneDestinationOccupied { path } => write!(
                formatter,
                "{} already exists; choose another destination folder or rename the repository first",
                path.display()
            ),
            Self::GitHubCloneFailed => write!(
                formatter,
                "Kanban could not clone that repository. Check the URL and your existing Git access, then try again."
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
            _ => None,
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
