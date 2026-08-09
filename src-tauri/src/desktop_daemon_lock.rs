use std::{
    error::Error,
    fmt,
    fs::OpenOptions,
    io,
    path::{Path, PathBuf},
};

use fs2::FileExt;

pub(crate) struct DaemonLock {
    _file: std::fs::File,
}

impl DaemonLock {
    pub(crate) fn acquire(data_directory: &Path) -> Result<Self, DaemonLockError> {
        let path = data_directory.join("daemon.lock");
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .map_err(|source| DaemonLockError::Open {
                path: path.clone(),
                source,
            })?;
        file.try_lock_exclusive()
            .map_err(|source| DaemonLockError::Acquire { path, source })?;
        Ok(Self { _file: file })
    }
}

#[derive(Debug)]
pub(crate) enum DaemonLockError {
    Open { path: PathBuf, source: io::Error },
    Acquire { path: PathBuf, source: io::Error },
}

impl fmt::Display for DaemonLockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open { path, source } => {
                write!(
                    formatter,
                    "cannot open local daemon lock {}: {source}",
                    path.display()
                )
            }
            Self::Acquire { path, source } => {
                write!(
                    formatter,
                    "local board daemon is already running for {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for DaemonLockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open { source, .. } | Self::Acquire { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs};

    use tempfile::TempDir;

    use super::{DaemonLock, DaemonLockError};

    #[test]
    fn prevents_another_daemon_from_using_the_same_data_directory() {
        let temporary_directory = TempDir::new().expect("temporary directory should be created");
        let _first = DaemonLock::acquire(temporary_directory.path())
            .expect("first daemon should acquire the lock");

        assert!(matches!(
            DaemonLock::acquire(temporary_directory.path()),
            Err(DaemonLockError::Acquire { .. })
        ));
    }

    #[test]
    fn reports_actionable_lock_acquisition_and_open_errors() {
        let temporary_directory = TempDir::new().expect("temporary directory should be created");
        let _first = DaemonLock::acquire(temporary_directory.path())
            .expect("first daemon should acquire the lock");
        let acquisition_error = DaemonLock::acquire(temporary_directory.path())
            .err()
            .expect("second daemon should be rejected");
        let occupied_path = temporary_directory.path().join("occupied-path");
        fs::write(&occupied_path, "not a directory").expect("fixture file should be written");
        let open_error = DaemonLock::acquire(&occupied_path)
            .err()
            .expect("a file cannot contain the daemon lock");

        assert!(matches!(
            &acquisition_error,
            DaemonLockError::Acquire { .. }
        ));
        assert!(acquisition_error.to_string().contains("already running"));
        assert!(acquisition_error.source().is_some());
        assert!(matches!(&open_error, DaemonLockError::Open { .. }));
        assert!(open_error.to_string().contains("cannot open"));
        assert!(open_error.source().is_some());
    }
}
