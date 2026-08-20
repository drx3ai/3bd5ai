use bd3_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::path::PathBuf;

/// Returns the path to the Bd3 configuration directory, which can be
/// specified by the `BD3_HOME` environment variable. If not set, defaults to
/// `~/.bd3`.
///
/// - If `BD3_HOME` is set, the value must exist and be a directory. The
///   value will be canonicalized and this function will Err otherwise.
/// - If `BD3_HOME` is not set, this function does not verify that the
///   directory exists.
pub fn find_bd3_home() -> std::io::Result<AbsolutePathBuf> {
    let bd3_home_env = std::env::var("BD3_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    find_bd3_home_from_env(bd3_home_env.as_deref())
}

fn find_bd3_home_from_env(bd3_home_env: Option<&str>) -> std::io::Result<AbsolutePathBuf> {
    // Honor the `BD3_HOME` environment variable when it is set to allow users
    // (and tests) to override the default location.
    match bd3_home_env {
        Some(val) => {
            let path = PathBuf::from(val);
            let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("BD3_HOME points to {val:?}, but that path does not exist"),
                ),
                _ => std::io::Error::new(
                    err.kind(),
                    format!("failed to read BD3_HOME {val:?}: {err}"),
                ),
            })?;

            if !metadata.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("BD3_HOME points to {val:?}, but that path is not a directory"),
                ))
            } else {
                let canonical = path.canonicalize().map_err(|err| {
                    std::io::Error::new(
                        err.kind(),
                        format!("failed to canonicalize BD3_HOME {val:?}: {err}"),
                    )
                })?;
                AbsolutePathBuf::from_absolute_path(canonical)
            }
        }
        None => {
            let mut p = home_dir().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?;
            p.push(".3bd");
            AbsolutePathBuf::from_absolute_path(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::find_bd3_home_from_env;
    use bd3_utils_absolute_path::AbsolutePathBuf;
    use dirs::home_dir;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_bd3_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-bd3-home");
        let missing_str = missing
            .to_str()
            .expect("missing 3bd home path should be valid utf-8");

        let err = find_bd3_home_from_env(Some(missing_str)).expect_err("missing BD3_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("BD3_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_bd3_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("bd3-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file 3bd home path should be valid utf-8");

        let err = find_bd3_home_from_env(Some(file_str)).expect_err("file BD3_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_bd3_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp 3bd home path should be valid utf-8");

        let resolved = find_bd3_home_from_env(Some(temp_str)).expect("valid BD3_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_bd3_home_without_env_uses_default_home_dir() {
        let resolved =
            find_bd3_home_from_env(/*bd3_home_env*/ None).expect("default BD3_HOME");
        let mut expected = home_dir().expect("home dir");
        expected.push(".3bd");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }
}
