mod error;
mod git_cli;
mod path;

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::domain::{Execution, Project, WorkItemId};

use git_cli::GitCli;
use path::{
    paths_match, prepare_workspace_path, reject_overlapping_roots, resolved_existing_path,
    resolved_path_for_creation, workspace_name,
};

pub use error::WorkspaceError;
pub use git_cli::GitError;
pub use path::PathAccess;

const TASK_BRANCH_PREFIX: &str = "kanban";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DependencySharingStrategy {
    IsolatedInstall,
    ManagedSharedCache,
    ExplicitProjectApprovedLink { approval_id: String },
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod error_tests;
