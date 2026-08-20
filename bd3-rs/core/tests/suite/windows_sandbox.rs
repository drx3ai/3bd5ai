use anyhow::Context;
use bd3_core::exec::ExecCapturePolicy;
use bd3_core::exec::ExecParams;
use bd3_core::exec::process_exec_tool_call;
use bd3_core::sandboxing::SandboxPermissions;
use bd3_core::windows_sandbox::sandbox_setup_is_complete;
use bd3_features::Feature;
use bd3_protocol::config_types::WindowsSandboxLevel;
use bd3_protocol::exec_output::ExecToolCallOutput;
use bd3_protocol::models::PermissionProfile;
use bd3_protocol::permissions::FileSystemAccessMode;
use bd3_protocol::permissions::FileSystemPath;
use bd3_protocol::permissions::FileSystemSandboxEntry;
use bd3_protocol::permissions::FileSystemSandboxPolicy;
use bd3_protocol::permissions::FileSystemSpecialPath;
use bd3_protocol::permissions::NetworkSandboxPolicy;
use core_test_support::PathExt;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::test_bd3::TestBd3Harness;
use core_test_support::test_bd3::test_bd3;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &std::ffi::OsStr) -> Self {
        let original = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

enum TestBd3Home {
    Persistent(PathBuf),
    Temporary(TempDir),
}

impl TestBd3Home {
    fn path(&self) -> &Path {
        match self {
            Self::Persistent(path) => path.as_path(),
            Self::Temporary(temp_dir) => temp_dir.path(),
        }
    }
}

fn bd3_home_for_windows_sandbox_test(name: &str) -> anyhow::Result<TestBd3Home> {
    if let Some(test_tmpdir) = std::env::var_os("TEST_TMPDIR") {
        // The elevated backend provisions machine-local sandbox users. Bazel
        // retries run in the same Windows VM, so keep BD3_HOME stable within
        // the test temp root and let setup reconcile its persisted ACL state.
        let bd3_home = PathBuf::from(test_tmpdir).join(name);
        std::fs::create_dir_all(&bd3_home)
            .with_context(|| format!("create stable test BD3_HOME {}", bd3_home.display()))?;
        return Ok(TestBd3Home::Persistent(bd3_home));
    }

    Ok(TestBd3Home::Temporary(TempDir::new()?))
}

fn stage_windows_sandbox_helpers() -> anyhow::Result<()> {
    let test_exe = std::env::current_exe().context("resolve current Windows test executable")?;
    let test_exe_dir = test_exe
        .parent()
        .context("Windows test executable should have a parent directory")?;
    let resources_dir = test_exe_dir.join("bd3-resources");
    match std::fs::create_dir_all(&resources_dir) {
        Ok(()) => {}
        Err(err)
            if err.kind() == std::io::ErrorKind::PermissionDenied && resources_dir.is_dir() => {}
        Err(err) => {
            return Err(err)
                .with_context(|| format!("create resources dir {}", resources_dir.display()));
        }
    }
    for helper_name in ["bd3-windows-sandbox-setup", "bd3-command-runner"] {
        let helper = bd3_utils_cargo_bin::cargo_bin(helper_name)?;
        let file_name = Path::new(helper_name).with_extension("exe");
        let destination = resources_dir.join(file_name);
        if let Err(err) = std::fs::copy(&helper, &destination) {
            // A sandbox helper can briefly remain alive after the sandboxed
            // command exits. Bazel may retry the test while that process still
            // has the staged executable open, so keep the already-staged copy.
            if err.kind() == std::io::ErrorKind::PermissionDenied && destination.exists() {
                continue;
            }
            return Err(err).with_context(|| {
                format!(
                    "stage Windows sandbox helper {} at {}",
                    helper.display(),
                    destination.display()
                )
            });
        }
    }
    Ok(())
}

#[tokio::test]
#[serial(bd3_home)]
async fn windows_restricted_token_rejects_exact_and_glob_deny_read_policy() -> anyhow::Result<()> {
    let bd3_home =
        bd3_home_for_windows_sandbox_test("windows-restricted-token-deny-read-bd3-home")?;
    let _bd3_home_guard = EnvVarGuard::set("BD3_HOME", bd3_home.path().as_os_str());
    let workspace = TempDir::new()?;
    let cwd = dunce::canonicalize(workspace.path())?.abs();
    let secret = cwd.join("secret.env");
    let future_secret = cwd.join("future.env");
    let public = cwd.join("public.txt");
    std::fs::write(&secret, "glob secret\n")?;
    std::fs::write(&public, "public ok\n")?;

    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                pattern: "**/*.env".to_string(),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: future_secret.into(),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);
    let permission_profile = PermissionProfile::from_runtime_permissions(
        &file_system_sandbox_policy,
        NetworkSandboxPolicy::Restricted,
    );

    let err = process_exec_tool_call(
        ExecParams {
            command: vec![
                "cmd.exe".to_string(),
                "/D".to_string(),
                "/C".to_string(),
                "type secret.env >NUL 2>NUL & echo exact secret 1>future.env 2>NUL & type future.env 2>NUL & type public.txt & exit /B 0"
                    .to_string(),
            ],
            cwd: cwd.clone(),
            expiration: 10_000.into(),
            capture_policy: ExecCapturePolicy::ShellTool,
            env: HashMap::new(),
            network: None,
            network_environment_id: None,
            sandbox_permissions: SandboxPermissions::UseDefault,
            windows_sandbox_level: WindowsSandboxLevel::RestrictedToken,
            windows_sandbox_private_desktop: false,
            justification: None,
            arg0: None,
        },
        &permission_profile,
        &cwd,
        std::slice::from_ref(&cwd),
        &None,
        /*use_legacy_landlock*/ false,
        /*stdout_stream*/ None,
    )
    .await
    .expect_err("restricted-token sandbox should reject deny-read restrictions");

    assert_eq!(
        err.to_string(),
        "unsupported operation: windows unelevated restricted-token sandbox cannot enforce deny-read restrictions directly; refusing to run unsandboxed"
    );
    Ok(())
}

#[tokio::test]
#[serial(bd3_home)]
async fn windows_elevated_does_not_create_missing_workspace_metadata() -> anyhow::Result<()> {
    let bd3_home =
        bd3_home_for_windows_sandbox_test("windows-elevated-missing-metadata-bd3-home")?;
    let _bd3_home_guard = EnvVarGuard::set("BD3_HOME", bd3_home.path().as_os_str());
    stage_windows_sandbox_helpers()?;
    let workspace = TempDir::new()?;
    let cwd = dunce::canonicalize(workspace.path())?.abs();
    let permission_profile = PermissionProfile::workspace_write()
        .materialize_project_roots_with_workspace_roots(std::slice::from_ref(&cwd));

    let output = process_exec_tool_call(
        ExecParams {
            command: vec![
                "cmd.exe".to_string(),
                "/D".to_string(),
                "/C".to_string(),
                "echo sandbox-ok".to_string(),
            ],
            cwd: cwd.clone(),
            expiration: 10_000.into(),
            capture_policy: ExecCapturePolicy::ShellTool,
            env: HashMap::new(),
            network: None,
            network_environment_id: None,
            sandbox_permissions: SandboxPermissions::UseDefault,
            windows_sandbox_level: WindowsSandboxLevel::Elevated,
            windows_sandbox_private_desktop: false,
            justification: None,
            arg0: None,
        },
        &permission_profile,
        &cwd,
        std::slice::from_ref(&cwd),
        &None,
        /*use_legacy_landlock*/ false,
        /*stdout_stream*/ None,
    )
    .await?;

    assert_eq!(output.exit_code, 0, "sandboxed command should complete");
    for name in bd3_protocol::permissions::PROTECTED_METADATA_PATH_NAMES {
        let path = cwd.join(name);
        assert!(
            !path.exists(),
            "elevated setup should not create missing workspace metadata: {}",
            path.display()
        );
    }
    Ok(())
}

#[tokio::test]
#[serial(bd3_home)]
async fn windows_elevated_enforces_deny_read_and_protects_setup_marker() -> anyhow::Result<()> {
    let bd3_home = bd3_home_for_windows_sandbox_test("windows-elevated-deny-read-bd3-home")?;
    let _bd3_home_guard = EnvVarGuard::set("BD3_HOME", bd3_home.path().as_os_str());
    stage_windows_sandbox_helpers()?;
    let workspace = TempDir::new()?;
    let cwd = dunce::canonicalize(workspace.path())?.abs();
    let glob_secret = cwd.join("secret.env");
    let exact_secret = cwd.join("exact-secret.txt");
    let public = cwd.join("public.txt");
    let setup_marker = bd3_home.path().join(".sandbox").join("setup_marker.json");
    std::fs::write(&glob_secret, "glob secret\n")?;
    std::fs::write(&exact_secret, "exact secret\n")?;
    std::fs::write(&public, "public ok\n")?;

    let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                pattern: "**/*.env".to_string(),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
        FileSystemSandboxEntry {
            path: exact_secret.into(),
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        },
    ]);
    let permission_profile = PermissionProfile::from_runtime_permissions(
        &file_system_sandbox_policy,
        NetworkSandboxPolicy::Restricted,
    );

    let ExecToolCallOutput {
        exit_code,
        stdout,
        ..
    } = process_exec_tool_call(
        ExecParams {
            command: vec![
                "cmd.exe".to_string(),
                "/D".to_string(),
                "/C".to_string(),
                format!(
                    "(type secret.env 1>NUL 2>NUL && echo GLOB-READ || echo GLOB-DENIED) & (type exact-secret.txt 1>NUL 2>NUL && echo EXACT-READ || echo EXACT-DENIED) & (type \"{}\" 1>NUL 2>NUL && echo MARKER-READ-ALLOWED || echo MARKER-READ-DENIED) & (echo tampered > \"{}\" 2>NUL && echo MARKER-WRITE-ALLOWED || echo MARKER-WRITE-DENIED) & type public.txt",
                    setup_marker.display(),
                    setup_marker.display()
                ),
            ],
            cwd: cwd.clone(),
            expiration: 10_000.into(),
            capture_policy: ExecCapturePolicy::ShellTool,
            env: HashMap::new(),
            network: None,
            network_environment_id: None,
            sandbox_permissions: SandboxPermissions::UseDefault,
            windows_sandbox_level: WindowsSandboxLevel::Elevated,
            windows_sandbox_private_desktop: false,
            justification: None,
            arg0: None,
        },
        &permission_profile,
        &cwd,
        std::slice::from_ref(&cwd),
        &None,
        /*use_legacy_landlock*/ false,
        /*stdout_stream*/ None,
    )
    .await?;

    assert_eq!(exit_code, 0, "sandboxed command should complete");
    assert!(
        stdout.text.contains("GLOB-DENIED"),
        "glob deny-read should block the secret: {stdout:?}"
    );
    assert!(
        !stdout.text.contains("GLOB-READ"),
        "glob deny-read should not allow the secret: {stdout:?}"
    );
    assert!(
        stdout.text.contains("EXACT-DENIED"),
        "exact deny-read should block the secret: {stdout:?}"
    );
    assert!(
        !stdout.text.contains("EXACT-READ"),
        "exact deny-read should not allow the secret: {stdout:?}"
    );
    assert!(
        stdout.text.contains("public ok"),
        "allowed reads should still work: {stdout:?}"
    );
    assert!(
        stdout.text.contains("MARKER-READ-DENIED"),
        "sandboxed command should not read setup readiness: {stdout:?}"
    );
    assert!(
        stdout.text.contains("MARKER-WRITE-DENIED"),
        "sandboxed command should not modify setup readiness: {stdout:?}"
    );
    assert!(
        !stdout.text.contains("MARKER-READ-ALLOWED"),
        "sandboxed command must not read setup readiness: {stdout:?}"
    );
    assert!(
        !stdout.text.contains("MARKER-WRITE-ALLOWED"),
        "sandboxed command must not modify setup readiness: {stdout:?}"
    );
    assert!(
        sandbox_setup_is_complete(bd3_home.path()),
        "setup should remain ready after the tamper attempt"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial(bd3_home)]
async fn windows_elevated_shell_and_unified_exec_enforce_managed_deny_reads() -> anyhow::Result<()>
{
    let bd3_home =
        bd3_home_for_windows_sandbox_test("windows-elevated-tool-runtime-deny-read-bd3-home")?;
    let _bd3_home_guard = EnvVarGuard::set("BD3_HOME", bd3_home.path().as_os_str());
    stage_windows_sandbox_helpers()?;

    let configured_bd3_home = dunce::canonicalize(bd3_home.path())?.abs();
    let builder = test_bd3()
        .with_windows_cmd_shell()
        .with_config(move |config| {
            config.bd3_home = configured_bd3_home;
            config.set_windows_elevated_sandbox_enabled(true);
            config
                .features
                .enable(Feature::UnifiedExec)
                .expect("test config should allow unified exec");

            let file_system_sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry {
                    path: FileSystemPath::Special {
                        value: FileSystemSpecialPath::Root,
                    },
                    access: FileSystemAccessMode::Read,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::GlobPattern {
                        pattern: "**/*.env".to_string(),
                    },
                    access: FileSystemAccessMode::Deny,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::Path {
                        path: config.cwd.join("exact-secret.txt").into(),
                    },
                    access: FileSystemAccessMode::Deny,
                    missing_path_behavior: None,
                },
            ]);
            config
                .permissions
                .set_permission_profile(PermissionProfile::from_runtime_permissions(
                    &file_system_sandbox_policy,
                    NetworkSandboxPolicy::Restricted,
                ))
                .expect("set managed deny-read permission profile");
        })
        .with_workspace_setup(|cwd, _fs| async move {
            std::fs::write(
                cwd.join("secret.env"),
                "glob secret should remain private\n",
            )?;
            std::fs::write(
                cwd.join("exact-secret.txt"),
                "exact secret should remain private\n",
            )?;
            std::fs::write(cwd.join("public.txt"), "public ok\n")?;
            Ok(())
        });
    let harness = TestBd3Harness::with_builder(builder).await?;

    let command = concat!(
        "(type secret.env 1>NUL 2>NUL && echo GLOB-READ || echo GLOB-DENIED) & ",
        "(type exact-secret.txt 1>NUL 2>NUL && echo EXACT-READ || echo EXACT-DENIED) & ",
        "type public.txt"
    );
    let shell_call_id = "windows-managed-deny-read-shell-command";
    let unified_call_id = "windows-managed-deny-read-exec-command";
    let shell_args = json!({
        "command": command,
        "timeout_ms": 30_000,
        "login": false,
    });
    let unified_args = json!({
        "cmd": command,
        "yield_time_ms": 30_000,
        "tty": false,
        "login": false,
    });
    mount_sse_sequence(
        harness.server(),
        vec![
            sse(vec![
                ev_response_created("resp-windows-shell-deny-read"),
                ev_function_call(
                    shell_call_id,
                    "shell_command",
                    &serde_json::to_string(&shell_args)?,
                ),
                ev_completed("resp-windows-shell-deny-read"),
            ]),
            sse(vec![
                ev_response_created("resp-windows-unified-deny-read"),
                ev_function_call(
                    unified_call_id,
                    "exec_command",
                    &serde_json::to_string(&unified_args)?,
                ),
                ev_completed("resp-windows-unified-deny-read"),
            ]),
            sse(vec![
                ev_assistant_message("msg-windows-deny-read", "done"),
                ev_completed("resp-windows-deny-read-complete"),
            ]),
        ],
    )
    .await;

    let permission_profile = harness
        .test()
        .config
        .permissions
        .effective_permission_profile();
    harness
        .submit_with_permission_profile("read the sandbox fixtures", permission_profile)
        .await?;

    for (tool_name, call_id) in [
        ("shell_command", shell_call_id),
        ("exec_command", unified_call_id),
    ] {
        let output = harness.function_call_stdout(call_id).await;
        assert!(
            output.contains("GLOB-DENIED"),
            "{tool_name} should reject glob-denied reads: {output:?}"
        );
        assert!(
            output.contains("EXACT-DENIED"),
            "{tool_name} should reject exact-path-denied reads: {output:?}"
        );
        assert!(
            output.contains("public ok"),
            "{tool_name} should preserve allowed reads: {output:?}"
        );
        assert!(
            !output.contains("GLOB-READ") && !output.contains("glob secret"),
            "{tool_name} leaked glob-denied file contents: {output:?}"
        );
        assert!(
            !output.contains("EXACT-READ") && !output.contains("exact secret"),
            "{tool_name} leaked exact-path-denied file contents: {output:?}"
        );
    }

    Ok(())
}
