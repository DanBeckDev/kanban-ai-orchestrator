use std::{collections::BTreeSet, error::Error, ffi::OsStr, io, path::Path};

use tempfile::TempDir;

use super::{
    GitCli, GitError, REPOSITORY_CONTEXT_ENVIRONMENT_VARIABLES,
    is_repository_context_environment_variable, parse_worktrees,
};

#[cfg(windows)]
use super::git_path_argument;

#[test]
fn parses_attached_and_detached_worktrees() {
    let worktrees = parse_worktrees(
        "worktree /workspace/project\0HEAD abc123\0branch refs/heads/main\0\0worktree /workspace/task\0HEAD def456\0detached\0\0",
    )
    .expect("Git worktree output should parse");

    assert_eq!(worktrees.len(), 2);
    assert_eq!(worktrees[0].branch.as_deref(), Some("refs/heads/main"));
    assert!(worktrees[1].branch.is_none());
}

#[test]
fn rejects_worktree_records_without_a_path() {
    assert!(parse_worktrees("HEAD abc123\0\0").is_err());
}

#[test]
fn clears_inherited_git_repository_context_before_starting_git() {
    let arguments = ["status".into()];
    let cleared_variables: BTreeSet<_> = GitCli
        .command(Path::new("/workspace/project"), &arguments)
        .get_envs()
        .filter(|(_, value)| value.is_none())
        .map(|(key, _)| key.to_string_lossy().into_owned())
        .collect();
    let expected_variables = REPOSITORY_CONTEXT_ENVIRONMENT_VARIABLES
        .iter()
        .map(|variable| (*variable).to_owned())
        .collect();

    assert_eq!(cleared_variables, expected_variables);
    assert!(is_repository_context_environment_variable(OsStr::new(
        "GIT_CONFIG_KEY_0"
    )));
    assert!(is_repository_context_environment_variable(OsStr::new(
        "GIT_CONFIG_VALUE_0"
    )));
    assert!(is_repository_context_environment_variable(OsStr::new(
        "GIT_CONFIG_GLOBAL"
    )));
    assert!(!is_repository_context_environment_variable(OsStr::new(
        "GIT_PAGER"
    )));
}

#[cfg(windows)]
#[test]
fn converts_verbatim_disk_and_network_paths_for_git() {
    assert_eq!(
        git_path_argument(Path::new(r"\\?\C:\workspace\task-1")).as_ref(),
        Path::new(r"C:\workspace\task-1")
    );
    assert_eq!(
        git_path_argument(Path::new(r"\\?\UNC\host\share\task-1")).as_ref(),
        Path::new(r"\\host\share\task-1")
    );
}

#[test]
fn reports_git_boundary_errors_without_hiding_their_source_contract() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let command_error = GitCli
        .repository_root(temporary_directory.path())
        .expect_err("a non-repository should fail Git validation");
    let io_error = GitError::CommandIo(io::Error::other("Git executable unavailable"));
    let failed_error = GitError::CommandFailed {
        operation: "create the task worktree",
        exit_code: Some(2),
        stderr: "invalid reference".to_owned(),
    };
    let non_utf8_error = GitError::NonUtf8Output {
        operation: "list registered worktrees",
    };

    assert!(matches!(command_error, GitError::CommandFailed { .. }));
    assert!(command_error.to_string().contains("project repository"));
    assert_eq!(
        failed_error.to_string(),
        "Git could not create the task worktree (exit code 2): invalid reference"
    );
    assert!(non_utf8_error.to_string().contains("non-UTF-8"));
    assert!(io_error.source().is_some());
    assert!(failed_error.source().is_none());
    assert!(non_utf8_error.source().is_none());
    assert!(GitError::MalformedWorktreeList.source().is_none());
}
