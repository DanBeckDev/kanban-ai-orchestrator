use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::{
    GitCli, RepositorySetup, WorkspaceError, inspect_project_repository,
    path::{is_safe_directory_component, resolved_existing_path},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepositoryCloneRequest {
    pub repository_url: String,
    pub destination_parent_path: String,
}

pub fn clone_github_repository(
    request: GitHubRepositoryCloneRequest,
) -> Result<RepositorySetup, WorkspaceError> {
    let repository_url = request.repository_url.trim();
    let destination_name = github_repository_directory_name(repository_url)?;
    let destination_parent = clone_destination_parent(&request.destination_parent_path)?;
    let destination = destination_parent.join(destination_name);

    clone_repository(&destination_parent, repository_url, &destination)
}

fn clone_repository(
    destination_parent: &Path,
    repository_url: &str,
    destination: &Path,
) -> Result<RepositorySetup, WorkspaceError> {
    match fs::symlink_metadata(destination) {
        Ok(_) => {
            return Err(WorkspaceError::CloneDestinationOccupied {
                path: destination.to_path_buf(),
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    let git = GitCli;
    if git
        .clone(destination_parent, repository_url, destination)
        .is_err()
    {
        remove_partial_clone(destination);
        return Err(WorkspaceError::GitHubCloneFailed);
    }

    match inspect_project_repository(destination) {
        Ok(setup) => Ok(setup),
        Err(_) => {
            remove_partial_clone(destination);
            Err(WorkspaceError::GitHubCloneFailed)
        }
    }
}

fn clone_destination_parent(path: &str) -> Result<PathBuf, WorkspaceError> {
    let path = resolved_existing_path(Path::new(path))?;
    if path.is_dir() {
        Ok(path)
    } else {
        Err(WorkspaceError::CloneDestinationMustBeDirectory { path })
    }
}

fn github_repository_directory_name(repository_url: &str) -> Result<String, WorkspaceError> {
    let repository_path =
        github_repository_path(repository_url).ok_or(WorkspaceError::InvalidGitHubRepositoryUrl)?;
    let mut segments = repository_path.split('/');
    let owner = segments.next();
    let repository = segments
        .next()
        .map(|name| name.strip_suffix(".git").unwrap_or(name));

    if owner.is_none_or(|name| !is_safe_directory_component(name))
        || repository.is_none_or(|name| !is_safe_directory_component(name))
        || segments.next().is_some()
    {
        return Err(WorkspaceError::InvalidGitHubRepositoryUrl);
    }

    Ok(repository.expect("repository validated above").to_owned())
}

fn github_repository_path(repository_url: &str) -> Option<&str> {
    let repository_url = repository_url.trim();
    repository_url
        .strip_prefix("https://github.com/")
        .or_else(|| repository_url.strip_prefix("ssh://git@github.com/"))
        .or_else(|| repository_url.strip_prefix("git@github.com:"))
        .map(|path| path.trim_end_matches('/'))
        .filter(|path| !path.is_empty() && !path.contains(['?', '#', '\\']))
}

fn remove_partial_clone(destination: &Path) {
    if matches!(fs::symlink_metadata(destination), Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink())
    {
        let _ = fs::remove_dir_all(destination);
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::TempDir;

    use super::{
        GitHubRepositoryCloneRequest, clone_github_repository, clone_repository,
        github_repository_directory_name, remove_partial_clone,
    };
    use crate::workspace::{
        WorkspaceError,
        tests::{repository, run_git},
    };

    #[test]
    fn accepts_supported_github_https_and_ssh_urls() {
        for url in [
            "https://github.com/acme/reliable-app.git",
            "ssh://git@github.com/acme/reliable-app",
            "git@github.com:acme/reliable-app.git",
        ] {
            assert_eq!(
                github_repository_directory_name(url).expect("URL should be accepted"),
                "reliable-app"
            );
        }
    }

    #[test]
    fn rejects_non_github_or_unsafe_clone_urls() {
        for url in [
            "https://example.test/acme/reliable-app",
            "https://token@github.com/acme/reliable-app",
            "https://github.com/acme/../reliable-app",
            "https://github.com/CON/reliable-app",
            "https://github.com/acme/reliable-app/extra",
            "https://github.com/acme/reliable-app?token=secret",
            "https://github.com/acme/CON",
        ] {
            assert!(matches!(
                github_repository_directory_name(url),
                Err(WorkspaceError::InvalidGitHubRepositoryUrl)
            ));
        }
    }

    #[test]
    fn rejects_an_existing_clone_target_without_touching_it() {
        let temporary_directory = TempDir::new().expect("temporary directory should exist");
        let destination = temporary_directory.path().join("reliable-app");
        fs::create_dir(&destination).expect("target should exist");
        fs::write(destination.join("keep.txt"), "keep").expect("target marker should exist");

        assert!(matches!(
            clone_github_repository(GitHubRepositoryCloneRequest {
                repository_url: "https://github.com/acme/reliable-app".to_owned(),
                destination_parent_path: temporary_directory.path().display().to_string(),
            }),
            Err(WorkspaceError::CloneDestinationOccupied { .. })
        ));
        assert_eq!(
            fs::read_to_string(destination.join("keep.txt")).expect("marker should remain"),
            "keep"
        );
    }

    #[test]
    fn removes_only_an_ordinary_partial_clone_directory() {
        let temporary_directory = TempDir::new().expect("temporary directory should exist");
        let destination = temporary_directory.path().join("partial-clone");
        fs::create_dir(&destination).expect("partial directory should exist");
        remove_partial_clone(&destination);
        assert!(!destination.exists());
    }

    #[test]
    fn clone_failure_does_not_create_a_target() {
        let temporary_directory = TempDir::new().expect("temporary directory should exist");
        let destination = temporary_directory.path().join("missing-repository");

        assert!(matches!(
            clone_repository(
                temporary_directory.path(),
                Path::new("missing-repository")
                    .display()
                    .to_string()
                    .as_str(),
                &destination,
            ),
            Err(WorkspaceError::GitHubCloneFailed)
        ));
        assert!(!destination.exists());
    }

    #[test]
    fn removes_an_invalid_clone_after_git_succeeds() {
        let temporary_directory = TempDir::new().expect("temporary directory should exist");
        let source = temporary_directory.path().join("empty-source");
        fs::create_dir(&source).expect("source should exist");
        run_git(&source, &["init", "--initial-branch=main"]);
        let destination = temporary_directory.path().join("empty-clone");

        assert!(matches!(
            clone_repository(
                temporary_directory.path(),
                source.display().to_string().as_str(),
                &destination,
            ),
            Err(WorkspaceError::GitHubCloneFailed)
        ));
        assert!(!destination.exists());
    }

    #[test]
    fn rejects_a_file_as_a_clone_destination_before_running_git() {
        let temporary_directory = TempDir::new().expect("temporary directory should exist");
        let destination = temporary_directory.path().join("not-a-directory");
        fs::write(&destination, "file").expect("file should exist");

        assert!(matches!(
            clone_github_repository(GitHubRepositoryCloneRequest {
                repository_url: "https://github.com/acme/reliable-app".to_owned(),
                destination_parent_path: destination.display().to_string(),
            }),
            Err(WorkspaceError::CloneDestinationMustBeDirectory { .. })
        ));
    }

    #[test]
    fn repository_fixture_remains_a_valid_clone_source_for_the_git_boundary() {
        let (_source_directory, source) = repository();
        let destination_parent = TempDir::new().expect("destination should exist");
        let destination = destination_parent.path().join("copy");
        super::GitCli
            .clone(
                destination_parent.path(),
                &source.display().to_string(),
                &destination,
            )
            .expect("typed Git boundary should clone a local test repository");
        assert!(destination.join(".git").exists());
    }
}
