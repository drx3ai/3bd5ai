use std::path::Path;

use anyhow::Result;
use app_test_support::app_server_json_shutdown_event;
use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;

fn bd3_command(bd3_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(bd3_utils_cargo_bin::cargo_bin("3bd")?);
    cmd.env("BD3_HOME", bd3_home);
    Ok(cmd)
}

#[test]
fn strict_config_rejects_unknown_config_fields_for_app_server() -> Result<()> {
    let bd3_home = TempDir::new()?;
    std::fs::write(
        bd3_home.path().join("config.toml"),
        r#"
foo = "bar"
"#,
    )?;

    let mut cmd = bd3_command(bd3_home.path())?;
    cmd.args(["app-server", "--strict-config", "--listen", "off"])
        .assert()
        .failure()
        .stderr(contains("unknown configuration field"));

    Ok(())
}

#[test]
fn agents_reject_session_overrides_before_starting_the_daemon() -> Result<()> {
    let bd3_home = TempDir::new()?;

    for args in [
        ["--yolo", "agents"].as_slice(),
        ["--search", "agents"].as_slice(),
        ["--model", "gpt-5", "agents"].as_slice(),
    ] {
        let mut cmd = bd3_command(bd3_home.path())?;
        cmd.args(args)
            .assert()
            .failure()
            .stderr(contains("invocation-specific configuration overrides"));
    }

    Ok(())
}

#[test]
fn app_server_emits_json_info_events() -> Result<()> {
    let bd3_home = TempDir::new()?;
    let event = app_server_json_shutdown_event("3bd", &["app-server"], bd3_home.path())?;

    assert_eq!(
        event,
        json!({
            "level": "INFO",
            "fields": {
                "message": "processor task exited",
                "exit_reason": "stdio_connection_closed",
                "remaining_connection_count": 0,
                "shutdown_forced": false,
            },
            "target": "bd3_app_server",
        })
    );

    Ok(())
}
