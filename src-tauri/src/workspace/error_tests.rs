use std::{error::Error, path::PathBuf};

use super::tests::{manager, repository, request};
use super::{DependencySharingStrategy, GitError, PathAccess, WorkspaceError};
use crate::domain::WorkItemId;

#[test]
fn records_an_explicit_dependency_sharing_strategy_without_creating_a_link() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let manager = manager(&repository_path, &workspace_root);
    let mut request = request("task-1");
    request.dependency_sharing = DependencySharingStrategy::ExplicitProjectApprovedLink {
        approval_id: "policy-decision-1".to_owned(),
    };

    let assignment = manager
        .provision(request)
        .expect("workspace should provision without linking dependencies");
    assert_eq!(
        assignment.dependency_sharing(),
        &DependencySharingStrategy::ExplicitProjectApprovedLink {
            approval_id: "policy-decision-1".to_owned()
        }
    );
    assert!(!assignment.path().join("node_modules").exists());
}

#[test]
fn rejects_an_assignment_owned_by_a_different_workspace_manager() {
    let (temporary_directory, repository_path) = repository();
    let primary_manager = manager(
        &repository_path,
        &temporary_directory.path().join("workspaces-a"),
    );
    let other_manager = manager(
        &repository_path,
        &temporary_directory.path().join("workspaces-b"),
    );
    let assignment = primary_manager
        .provision(request("task-1"))
        .expect("workspace should provision");

    assert!(matches!(
        other_manager.authorize_path(&assignment, assignment.path(), PathAccess::Write),
        Err(WorkspaceError::WorkspaceAssignmentOutsideManagedRoot { .. })
    ));
}

#[test]
fn reports_workspace_errors_with_sources_only_for_wrapped_failures() {
    let filesystem_error = WorkspaceError::FileSystem(std::io::Error::other("disk unavailable"));
    let policy_error = WorkspaceError::BaseRepositoryMutationDenied {
        path: PathBuf::from("/project/README.md"),
    };

    assert_eq!(
        policy_error.to_string(),
        "writing the base repository path /project/README.md is denied"
    );
    assert!(filesystem_error.source().is_some());
    assert!(policy_error.source().is_none());
}

#[test]
fn formats_every_workspace_policy_error_for_an_actionable_user_response() {
    let errors = vec![
        WorkspaceError::Git(GitError::MalformedWorktreeList),
        WorkspaceError::ProjectRepositoryMustBeRoot {
            declared_path: PathBuf::from("/project/nested"),
            detected_path: PathBuf::from("/project"),
        },
        WorkspaceError::WorkspaceRootOverlapsRepository {
            repository_path: PathBuf::from("/project"),
            workspace_path: PathBuf::from("/project/workspaces"),
        },
        WorkspaceError::UnsafeWorkItemId {
            work_item_id: WorkItemId::from("../escape"),
        },
        WorkspaceError::WorkspacePathAssignedToDifferentBranch {
            path: PathBuf::from("/workspaces/task-1"),
            existing_branch: "feature/other".to_owned(),
            requested_branch: "kanban/task-1".to_owned(),
        },
        WorkspaceError::BranchAlreadyAssigned {
            branch_name: "kanban/task-1".to_owned(),
            path: PathBuf::from("/workspaces/other"),
        },
        WorkspaceError::WorkspacePathOccupied {
            path: PathBuf::from("/workspaces/task-1"),
        },
        WorkspaceError::RecoveryBranchDiverged {
            branch_name: "kanban/task-1".to_owned(),
            base_ref: "main".to_owned(),
        },
        WorkspaceError::ExecutionWorkItemMismatch {
            execution_work_item_id: WorkItemId::from("task-2"),
            assignment_work_item_id: WorkItemId::from("task-1"),
        },
        WorkspaceError::ExecutionWorkspaceMismatch {
            execution_path: PathBuf::from("/project"),
            assignment_path: PathBuf::from("/workspaces/task-1"),
        },
        WorkspaceError::WorktreeIdentityMismatch {
            expected_path: PathBuf::from("/workspaces/task-1"),
            actual_path: PathBuf::from("/workspaces/task-1"),
            expected_branch: "kanban/task-1".to_owned(),
            actual_branch: "alternate".to_owned(),
        },
        WorkspaceError::WorkspaceAssignmentOutsideManagedRoot {
            assignment_path: PathBuf::from("/outside/task-1"),
            workspace_root: PathBuf::from("/workspaces"),
        },
        WorkspaceError::BaseRepositoryMutationDenied {
            path: PathBuf::from("/project/README.md"),
        },
        WorkspaceError::UndeclaredPathAccessDenied {
            path: PathBuf::from("/outside"),
            access: PathAccess::Read,
        },
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
    }
    assert!(
        WorkspaceError::Git(GitError::MalformedWorktreeList)
            .source()
            .is_some()
    );
    assert_eq!(PathAccess::Read.to_string(), "read");
    assert_eq!(PathAccess::Write.to_string(), "write");
}
