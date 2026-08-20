use std::path::PathBuf;

use bd3_utils_absolute_path::AbsolutePathBuf;

/// Runtime paths needed by exec-server child processes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecServerRuntimePaths {
    /// Stable path to the Bd3 executable used to launch hidden helper modes.
    pub bd3_self_exe: AbsolutePathBuf,
    /// Path to the Linux sandbox helper alias used when the platform sandbox
    /// needs to re-enter Bd3 by argv0.
    pub bd3_linux_sandbox_exe: Option<AbsolutePathBuf>,
}

impl ExecServerRuntimePaths {
    pub fn from_optional_paths(
        bd3_self_exe: Option<PathBuf>,
        bd3_linux_sandbox_exe: Option<PathBuf>,
    ) -> std::io::Result<Self> {
        let bd3_self_exe = bd3_self_exe.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "3BD executable path is not configured",
            )
        })?;
        Self::new(bd3_self_exe, bd3_linux_sandbox_exe)
    }

    pub fn new(
        bd3_self_exe: PathBuf,
        bd3_linux_sandbox_exe: Option<PathBuf>,
    ) -> std::io::Result<Self> {
        Ok(Self {
            bd3_self_exe: absolute_path(bd3_self_exe)?,
            bd3_linux_sandbox_exe: bd3_linux_sandbox_exe.map(absolute_path).transpose()?,
        })
    }
}

fn absolute_path(path: PathBuf) -> std::io::Result<AbsolutePathBuf> {
    AbsolutePathBuf::from_absolute_path(path.as_path())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
}
