use std::{
    error::Error,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

use crate::{
    domain::{
        Execution, ExecutionId, ExecutionStatus, ExecutionUsage, Project, ProjectId,
        SchemaMetadata, WorkItemId,
    },
    workspace::{
        DependencySharingStrategy, GitError, PathAccess, WorkspaceError, WorkspaceManager,
        WorkspaceProvisionRequest,
    },
};

use super::git_cli::GitCli;

fn repository() -> (TempDir, PathBuf) {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let repository_path = temporary_directory.path().join("project");
    fs::create_dir(&repository_path).expect("repository directory should be created");
    run_git(&repository_path, &["init", "--initial-branch=main"]);
    run_git(
        &repository_path,
        &["config", "user.email", "agent@example.test"],
    );
    run_git(&repository_path, &["config", "user.name", "Test Agent"]);
    fs::write(repository_path.join("README.md"), "# Project\n")
        .expect("initial project file should be written");
    run_git(&repository_path, &["add", "README.md"]);
    run_git(&repository_path, &["commit", "-m", "Initial commit"]);

    (temporary_directory, repository_path)
}

fn run_git(directory: &Path, arguments: &[&str]) {
    let arguments = arguments.iter().map(OsString::from).collect::<Vec<_>>();
    let output = GitCli
        .command(directory, &arguments)
        .output()
        .expect("Git command should start");
    assert!(
        output.status.success(),
        "Git command {:?} failed: {}",
        arguments,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn project(repository_path: &Path) -> Project {
    Project {
        schema: SchemaMetadata::current(),
        id: ProjectId::from("project-1"),
        name: "Workspace test project".to_owned(),
        repository_path: repository_path.display().to_string(),
        base_ref: "main".to_owned(),
        policy_set_id: "policy-1".to_owned(),
    }
}

fn manager(repository_path: &Path, workspace_root: &Path) -> WorkspaceManager {
    WorkspaceManager::new(&project(repository_path), workspace_root)
        .expect("workspace manager should initialize")
}

fn request(work_item_id: &str) -> WorkspaceProvisionRequest {
    WorkspaceProvisionRequest {
        work_item_id: WorkItemId::from(work_item_id),
        dependency_sharing: DependencySharingStrategy::IsolatedInstall,
    }
}

fn execution(work_item_id: &str, workspace_path: &Path) -> Execution {
    Execution {
        schema: SchemaMetadata::current(),
        id: ExecutionId::from("execution-1"),
        work_item_id: WorkItemId::from(work_item_id),
        adapter_name: "test-adapter".to_owned(),
        status: ExecutionStatus::Pending,
        session_id: None,
        workspace_path: workspace_path.display().to_string(),
        usage: ExecutionUsage {
            input_tokens: 0,
            output_tokens: 0,
            cost_micros: None,
        },
        last_event_sequence: 0,
    }
}

#[test]
fn provisions_an_external_worktree_idempotently_without_touching_the_base_worktree() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let manager = manager(&repository_path, &workspace_root);

    let assignment = manager
        .provision(request("task-1"))
        .expect("workspace should provision");
    let repeated_assignment = manager
        .provision(request("task-1"))
        .expect("same workspace should be reused");

    assert_eq!(assignment, repeated_assignment);
    assert_eq!(assignment.work_item_id(), &WorkItemId::from("task-1"));
    assert_eq!(assignment.branch_name(), "kanban/task-1");
    assert_eq!(
        assignment.dependency_sharing(),
        &DependencySharingStrategy::IsolatedInstall
    );
    assert!(
        assignment.path().starts_with(
            workspace_root
                .canonicalize()
                .expect("workspace root should resolve")
        )
    );
    assert!(
        !assignment.path().starts_with(
            repository_path
                .canonicalize()
                .expect("repository root should resolve")
        )
    );
    assert!(assignment.path.join("README.md").is_file());
    let status = Command::new("git")
        .arg("-C")
        .arg(&repository_path)
        .args(["status", "--porcelain"])
        .output()
        .expect("Git status should start");
    assert!(status.status.success());
    assert!(String::from_utf8_lossy(&status.stdout).trim().is_empty());
}

#[test]
fn recovers_an_interrupted_empty_target_by_attaching_the_precreated_task_branch() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let target_path = workspace_root.join("recover-task");
    fs::create_dir_all(&target_path).expect("empty interrupted workspace target should exist");
    run_git(&repository_path, &["branch", "kanban/recover-task", "main"]);
    let manager = manager(&repository_path, &workspace_root);

    let assignment = manager
        .provision(request("recover-task"))
        .expect("interrupted provisioning should recover");

    assert_eq!(
        assignment.path,
        target_path
            .canonicalize()
            .expect("recovered workspace path should resolve")
    );
    assert!(assignment.path.join("README.md").is_file());
    assert_eq!(assignment.branch_name, "kanban/recover-task");
}

#[test]
fn refuses_to_recover_a_task_branch_that_diverged_from_its_declared_base() {
    let (temporary_directory, repository_path) = repository();
    run_git(&repository_path, &["switch", "-c", "feature/diverged"]);
    fs::write(repository_path.join("feature.txt"), "unrelated branch work")
        .expect("feature file should be written");
    run_git(&repository_path, &["add", "feature.txt"]);
    run_git(&repository_path, &["commit", "-m", "Diverge from main"]);
    run_git(&repository_path, &["branch", "kanban/task-1"]);
    run_git(&repository_path, &["switch", "main"]);
    let workspace_root = temporary_directory.path().join("workspaces");
    let target_path = workspace_root.join("task-1");
    fs::create_dir_all(&target_path).expect("empty target should be created before recovery");
    let manager = manager(&repository_path, &workspace_root);

    assert!(matches!(
        manager.provision(request("task-1")),
        Err(WorkspaceError::RecoveryBranchDiverged { .. })
    ));
    assert!(target_path.is_dir());
    assert!(
        fs::read_dir(target_path)
            .expect("target should remain readable")
            .next()
            .is_none()
    );
}

#[test]
fn rejects_unsafe_workspace_ids_before_creating_any_path() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let manager = manager(&repository_path, &workspace_root);

    assert!(matches!(
        manager.provision(request("../escape")),
        Err(WorkspaceError::UnsafeWorkItemId { .. })
    ));
    assert!(
        fs::read_dir(&workspace_root)
            .expect("workspace root should remain readable")
            .next()
            .is_none()
    );
    assert!(matches!(
        manager.provision(request("")),
        Err(WorkspaceError::UnsafeWorkItemId { .. })
    ));
    assert!(matches!(
        manager.provision(request("CON")),
        Err(WorkspaceError::UnsafeWorkItemId { .. })
    ));
}

#[test]
fn rejects_workspace_roots_that_overlap_the_project_repository() {
    let (temporary_directory, repository_path) = repository();

    assert!(matches!(
        WorkspaceManager::new(
            &project(&repository_path),
            repository_path.join("workspaces")
        ),
        Err(WorkspaceError::WorkspaceRootOverlapsRepository { .. })
    ));
    assert!(!repository_path.join("workspaces").exists());
    assert!(matches!(
        WorkspaceManager::new(&project(&repository_path), temporary_directory.path()),
        Err(WorkspaceError::WorkspaceRootOverlapsRepository { .. })
    ));
}

#[test]
fn rejects_a_project_path_that_is_not_its_repository_root() {
    let (temporary_directory, repository_path) = repository();
    let nested_path = repository_path.join("nested");
    fs::create_dir(&nested_path).expect("nested directory should be created");
    let mut nested_project = project(&repository_path);
    nested_project.repository_path = nested_path.display().to_string();

    assert!(matches!(
        WorkspaceManager::new(
            &nested_project,
            temporary_directory.path().join("workspaces")
        ),
        Err(WorkspaceError::ProjectRepositoryMustBeRoot { .. })
    ));
}

#[test]
fn rejects_a_project_with_an_unresolvable_base_ref_before_creating_workspaces() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let mut project = project(&repository_path);
    project.base_ref = "missing-base-ref".to_owned();

    assert!(matches!(
        WorkspaceManager::new(&project, &workspace_root),
        Err(WorkspaceError::Git(GitError::CommandFailed { .. }))
    ));
    assert!(!workspace_root.exists());
}

#[test]
fn rejects_a_base_ref_that_looks_like_a_git_command_option() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let mut project = project(&repository_path);
    project.base_ref = "--quiet".to_owned();

    assert!(matches!(
        WorkspaceManager::new(&project, &workspace_root),
        Err(WorkspaceError::Git(GitError::CommandFailed { .. }))
    ));
    assert!(!workspace_root.exists());
}

#[test]
fn rejects_a_registered_workspace_path_with_a_different_branch() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let target_path = workspace_root.join("task-1");
    fs::create_dir_all(&workspace_root).expect("workspace root should be created");
    run_git(
        &repository_path,
        &[
            "worktree",
            "add",
            "-b",
            "feature/other-task",
            target_path.to_str().expect("path should be UTF-8"),
            "main",
        ],
    );
    let manager = manager(&repository_path, &workspace_root);

    assert!(matches!(
        manager.provision(request("task-1")),
        Err(WorkspaceError::WorkspacePathAssignedToDifferentBranch { .. })
    ));
}

#[test]
fn rejects_a_task_branch_that_is_registered_to_another_workspace() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let other_workspace_path = temporary_directory.path().join("other-workspace");
    run_git(
        &repository_path,
        &[
            "worktree",
            "add",
            "-b",
            "kanban/task-1",
            other_workspace_path.to_str().expect("path should be UTF-8"),
            "main",
        ],
    );
    let manager = manager(&repository_path, &workspace_root);

    assert!(matches!(
        manager.provision(request("task-1")),
        Err(WorkspaceError::BranchAlreadyAssigned { .. })
    ));
}

#[test]
fn never_removes_a_nonempty_unregistered_workspace_path() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let target_path = workspace_root.join("task-1");
    fs::create_dir_all(&target_path).expect("workspace target should be created");
    fs::write(target_path.join("preserve.txt"), "user content")
        .expect("user content should be written");
    let manager = manager(&repository_path, &workspace_root);

    assert!(matches!(
        manager.provision(request("task-1")),
        Err(WorkspaceError::WorkspacePathOccupied { .. })
    ));
    assert!(target_path.join("preserve.txt").is_file());
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_workspace_targets_without_following_them() {
    use std::os::unix::fs::symlink;

    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let linked_directory = temporary_directory.path().join("linked-directory");
    fs::create_dir(&workspace_root).expect("workspace root should be created");
    fs::create_dir(&linked_directory).expect("linked directory should be created");
    symlink(&linked_directory, workspace_root.join("task-1"))
        .expect("workspace target should be a symbolic link");
    let manager = manager(&repository_path, &workspace_root);

    assert!(matches!(
        manager.provision(request("task-1")),
        Err(WorkspaceError::WorkspacePathOccupied { .. })
    ));
    assert!(linked_directory.is_dir());
}

#[test]
fn verifies_an_execution_against_its_assigned_worktree() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let manager = manager(&repository_path, &workspace_root);
    let assignment = manager
        .provision(request("task-1"))
        .expect("workspace should provision");

    manager
        .verify_execution_workspace(&execution("task-1", &assignment.path), &assignment)
        .expect("matching execution workspace should verify");
    assert!(matches!(
        manager.verify_execution_workspace(&execution("task-2", &assignment.path), &assignment),
        Err(WorkspaceError::ExecutionWorkItemMismatch { .. })
    ));
    assert!(matches!(
        manager.verify_execution_workspace(&execution("task-1", &repository_path), &assignment),
        Err(WorkspaceError::ExecutionWorkspaceMismatch { .. })
    ));
    run_git(&assignment.path, &["switch", "-c", "alternate-branch"]);
    assert!(matches!(
        manager.verify_execution_workspace(&execution("task-1", &assignment.path), &assignment),
        Err(WorkspaceError::WorktreeIdentityMismatch { .. })
    ));
}

#[test]
fn denies_base_repository_writes_and_undeclared_paths_but_allows_assigned_workspace_paths() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let manager = manager(&repository_path, &workspace_root);
    let assignment = manager
        .provision(request("task-1"))
        .expect("workspace should provision");

    manager
        .authorize_path(
            &assignment,
            &assignment.path.join("new-directory/output.txt"),
            PathAccess::Write,
        )
        .expect("task worktree writes should be allowed");
    assert!(matches!(
        manager.authorize_path(
            &assignment,
            &repository_path.join("README.md"),
            PathAccess::Write,
        ),
        Err(WorkspaceError::BaseRepositoryMutationDenied { .. })
    ));
    assert!(matches!(
        manager.authorize_path(
            &assignment,
            &repository_path.join("README.md"),
            PathAccess::Read
        ),
        Err(WorkspaceError::UndeclaredPathAccessDenied { .. })
    ));
    assert!(matches!(
        manager.authorize_path(
            &assignment,
            &temporary_directory.path().join("outside.txt"),
            PathAccess::Read,
        ),
        Err(WorkspaceError::UndeclaredPathAccessDenied { .. })
    ));
    assert!(matches!(
        manager.authorize_path(
            &assignment,
            &temporary_directory.path().join("outside.txt"),
            PathAccess::Write,
        ),
        Err(WorkspaceError::UndeclaredPathAccessDenied { .. })
    ));
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
        assignment.dependency_sharing,
        DependencySharingStrategy::ExplicitProjectApprovedLink {
            approval_id: "policy-decision-1".to_owned(),
        }
    );
    assert!(!assignment.path.join("node_modules").exists());
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
