use anyhow::Result;
use predicates::str::contains;
use std::path::Path;
use tempfile::TempDir;

fn bd3_command(bd3_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(bd3_utils_cargo_bin::cargo_bin("3bd")?);
    cmd.env("BD3_HOME", bd3_home);
    Ok(cmd)
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn update_does_not_start_interactive_prompt() -> Result<()> {
    let bd3_home = TempDir::new()?;

    bd3_command(bd3_home.path())?
        .arg("update")
        .assert()
        .failure()
        .stderr(contains("`3bd update` is not available in debug builds"));

    Ok(())
}
