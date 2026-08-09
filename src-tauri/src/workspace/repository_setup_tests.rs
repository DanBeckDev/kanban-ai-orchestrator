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
