use std::path::Path;

use super::git_cli::{GitCli, GitError};

const CONVENTIONAL_PRIMARY_BRANCHES: &[&str] = &["main", "master", "trunk"];

impl GitCli {
    pub fn preferred_base_branch(&self, directory: &Path) -> Result<String, GitError> {
        let remotes = self.remote_names(directory)?;
        if let Some(branch) = self.remote_default_branch(directory, &remotes)? {
            return Ok(branch);
        }
        for branch in CONVENTIONAL_PRIMARY_BRANCHES {
            if self.reference_exists(directory, branch)? {
                return Ok((*branch).to_owned());
            }
        }
        for remote in remotes {
            for branch in CONVENTIONAL_PRIMARY_BRANCHES {
                let reference = format!("{remote}/{branch}");
                if self.reference_exists(directory, &reference)? {
                    return Ok(reference);
                }
            }
        }
        self.checked_out_branch(directory)
    }

    fn checked_out_branch(&self, directory: &Path) -> Result<String, GitError> {
        self.successful_text(
            directory,
            "resolve the repository checked-out branch",
            &["rev-parse".into(), "--abbrev-ref".into(), "HEAD".into()],
        )
    }

    fn remote_names(&self, directory: &Path) -> Result<Vec<String>, GitError> {
        Ok(self
            .successful_text(directory, "list repository remotes", &["remote".into()])?
            .lines()
            .map(str::to_owned)
            .collect())
    }

    fn remote_default_branch(
        &self,
        directory: &Path,
        remotes: &[String],
    ) -> Result<Option<String>, GitError> {
        for remote in remotes {
            let output = self.output(
                directory,
                &[
                    "symbolic-ref".into(),
                    "--quiet".into(),
                    "--short".into(),
                    format!("refs/remotes/{remote}/HEAD").into(),
                ],
            )?;
            match output.status.code() {
                Some(0) => {
                    let branch = String::from_utf8(output.stdout)
                        .map(|text| text.trim_end().to_owned())
                        .map_err(|_| GitError::NonUtf8Output {
                            operation: "resolve the remote default branch",
                        })?;
                    if self.reference_exists(directory, &branch)? {
                        return Ok(Some(branch));
                    }
                }
                Some(1) => continue,
                _ => {
                    return Err(GitError::command_failed(
                        "resolve the remote default branch",
                        output,
                    ));
                }
            }
        }
        Ok(None)
    }

    fn reference_exists(&self, directory: &Path, reference: &str) -> Result<bool, GitError> {
        let output = self.output(
            directory,
            &[
                "rev-parse".into(),
                "--verify".into(),
                "--quiet".into(),
                "--end-of-options".into(),
                format!("{reference}^{{commit}}").into(),
            ],
        )?;
        match output.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(GitError::command_failed(
                "check whether a repository reference exists",
                output,
            )),
        }
    }
}
