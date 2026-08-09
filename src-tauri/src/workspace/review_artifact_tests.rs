use std::fs;

use super::tests::{execution, manager, repository, request, run_git};

#[test]
fn review_artifacts_include_committed_and_staged_task_changes() {
    let (temporary_directory, repository_path) = repository();
    let workspace_root = temporary_directory.path().join("workspaces");
    let manager = manager(&repository_path, &workspace_root);
    let assignment = manager
        .provision(request("task-1"))
        .expect("workspace should provision");
    fs::write(assignment.path().join("README.md"), "# Updated project\n")
        .expect("task change should be written");
    run_git(assignment.path(), &["add", "README.md"]);
    run_git(assignment.path(), &["commit", "-m", "Task update"]);
    fs::write(
        assignment.path().join("README.md"),
        "# Updated project again\n",
    )
    .expect("staged task change should be written");
    run_git(assignment.path(), &["add", "README.md"]);

    let artifacts = manager
        .collect_review_artifacts(&execution("task-1", assignment.path()))
        .expect("review artifacts should collect");

    assert!(artifacts.head_commit.is_some());
    assert!(
        artifacts
            .committed_diff_stat
            .expect("committed changes should be visible")
            .contains("README.md")
    );
    assert!(
        artifacts
            .working_diff_stat
            .expect("staged changes should be visible")
            .contains("README.md")
    );
}
