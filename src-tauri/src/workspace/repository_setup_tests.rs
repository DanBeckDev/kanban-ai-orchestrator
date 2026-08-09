use std::fs;

use crate::workspace::{
    GitError, WorkspaceError, inspect_project_repository, validate_project_repository,
};

use super::tests::repository;

#[test]
fn inspects_a_repository_root_with_a_name_and_detected_base_branch() {
    let (_temporary_directory, repository_path) = repository();

    let setup = inspect_project_repository(&repository_path)
        .expect("repository root should be available for setup");

    assert_eq!(setup.suggested_board_name, "project");
    assert_eq!(setup.base_ref, "main");
    assert_eq!(
        setup.repository_path,
        repository_path
            .canonicalize()
            .expect("repository path should resolve")
            .display()
            .to_string()
    );
}

#[test]
fn prefers_main_when_the_selected_repository_is_checked_out_on_a_feature_branch() {
    let (_temporary_directory, repository_path) = repository();
    super::tests::run_git(&repository_path, &["switch", "-c", "feature/setup-copy"]);

    let setup = inspect_project_repository(&repository_path)
        .expect("repository root should be available for setup");

    assert_eq!(setup.base_ref, "main");
}

#[test]
fn prefers_a_locally_known_remote_default_branch_over_conventional_branch_names() {
    let (_temporary_directory, repository_path) = repository();
    super::tests::run_git(&repository_path, &["branch", "release"]);
    super::tests::run_git(
        &repository_path,
        &[
            "remote",
            "add",
            "origin",
            "https://example.test/project.git",
        ],
    );
    super::tests::run_git(
        &repository_path,
        &["update-ref", "refs/remotes/origin/release", "HEAD"],
    );
    super::tests::run_git(
        &repository_path,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/release",
        ],
    );

    let setup = inspect_project_repository(&repository_path)
        .expect("repository root should be available for setup");

    assert_eq!(setup.base_ref, "origin/release");
}

#[test]
fn prefers_a_known_remote_primary_branch_when_its_default_is_not_available() {
    let (_temporary_directory, repository_path) = repository();
    super::tests::run_git(&repository_path, &["switch", "-c", "feature/setup-copy"]);
    super::tests::run_git(&repository_path, &["branch", "-D", "main"]);
    super::tests::run_git(
        &repository_path,
        &[
            "remote",
            "add",
            "origin",
            "https://example.test/project.git",
        ],
    );
    super::tests::run_git(
        &repository_path,
        &["update-ref", "refs/remotes/origin/trunk", "HEAD"],
    );

    let setup = inspect_project_repository(&repository_path)
        .expect("repository root should be available for setup");

    assert_eq!(setup.base_ref, "origin/trunk");
}

#[test]
fn falls_back_to_the_checked_out_branch_only_when_no_primary_branch_is_known() {
    let (_temporary_directory, repository_path) = repository();
    super::tests::run_git(&repository_path, &["switch", "-c", "feature/setup-copy"]);
    super::tests::run_git(&repository_path, &["branch", "-D", "main"]);

    let setup = inspect_project_repository(&repository_path)
        .expect("repository root should be available for setup");

    assert_eq!(setup.base_ref, "feature/setup-copy");
}

#[test]
fn rejects_a_selected_subdirectory_before_local_board_creation() {
    let (_temporary_directory, repository_path) = repository();
    let nested_path = repository_path.join("nested");
    fs::create_dir(&nested_path).expect("nested directory should be created");

    assert!(matches!(
        inspect_project_repository(&nested_path),
        Err(WorkspaceError::ProjectRepositoryMustBeRoot { .. })
    ));
}

#[test]
fn validates_an_advanced_base_ref_before_local_board_creation() {
    let (_temporary_directory, repository_path) = repository();

    assert!(matches!(
        validate_project_repository(&repository_path, Some("missing-base-ref")),
        Err(WorkspaceError::Git(GitError::CommandFailed { .. }))
    ));
}
