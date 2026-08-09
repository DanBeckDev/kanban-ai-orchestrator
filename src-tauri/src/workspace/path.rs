use std::{
    fmt, fs, io,
    path::{Component, Path, PathBuf},
};

use crate::domain::WorkItemId;

use super::WorkspaceError;

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

pub(super) fn workspace_name(work_item_id: &WorkItemId) -> Result<&str, WorkspaceError> {
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

pub(super) fn prepare_workspace_path(path: &Path) -> Result<(), WorkspaceError> {
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

pub(super) fn reject_overlapping_roots(
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

pub(super) fn paths_match(left: &Path, right: &Path) -> Result<bool, WorkspaceError> {
    Ok(resolved_path_for_creation(left)? == resolved_path_for_creation(right)?)
}

pub(super) fn resolved_existing_path(path: &Path) -> Result<PathBuf, WorkspaceError> {
    path.canonicalize().map_err(WorkspaceError::FileSystem)
}

pub(super) fn resolved_path_for_creation(path: &Path) -> Result<PathBuf, WorkspaceError> {
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
                let component = candidate.file_name().ok_or_else(missing_parent_error)?;
                missing_components.push(component.to_owned());
                candidate = candidate
                    .parent()
                    .ok_or_else(missing_parent_error)?
                    .to_path_buf();
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn missing_parent_error() -> WorkspaceError {
    WorkspaceError::FileSystem(io::Error::new(
        io::ErrorKind::NotFound,
        "could not resolve an existing parent directory",
    ))
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

#[cfg(test)]
mod tests {
    use super::{
        is_reserved_windows_directory_name, normalized_absolute_path, resolved_path_for_creation,
    };
    use crate::workspace::WorkspaceError;
    use std::{env, path::Path};

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
        assert!(is_reserved_windows_directory_name("con"));
        assert!(is_reserved_windows_directory_name("LPT9"));
        assert!(!is_reserved_windows_directory_name("task-1"));
    }
}
