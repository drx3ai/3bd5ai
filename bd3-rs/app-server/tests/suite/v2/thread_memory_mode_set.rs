use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::TestAppServer;
use app_test_support::create_fake_rollout;
use app_test_support::create_mock_responses_server_repeating_assistant;
use bd3_app_server_protocol::ClientRequest;
use bd3_app_server_protocol::ThreadMemoryMode;
use bd3_app_server_protocol::ThreadMemoryModeSetParams;
use bd3_app_server_protocol::ThreadMemoryModeSetResponse;
use bd3_app_server_protocol::ThreadStartParams;
use bd3_app_server_protocol::ThreadStartResponse;
use bd3_features::Feature;
use bd3_protocol::ThreadId;
use bd3_state::StateRuntime;
use bd3_utils_absolute_path::test_support::PathExt;
use pretty_assertions::assert_eq;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn thread_memory_mode_set_updates_loaded_thread_state() -> Result<()> {
    let server = create_mock_responses_server_repeating_assistant("Done").await;
    let bd3_home = TempDir::new()?;
    MockResponsesConfig::new(&server.uri())
        .with_root_config("suppress_unstable_features_warning = true")
        .enable_feature(Feature::Sqlite)
        .write(bd3_home.path())?;
    let state_db = init_state_db(bd3_home.path()).await?;

    let mut mcp = TestAppServer::builder()
        .with_bd3_home(bd3_home.path())
        .build_initialized()
        .await?;

    let ThreadStartResponse { thread, .. } = mcp
        .start_thread(ThreadStartParams {
            model: Some("mock-model".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_uuid = ThreadId::from_string(&thread.id)?;

    let _: ThreadMemoryModeSetResponse = mcp
        .request(|request_id| ClientRequest::ThreadMemoryModeSet {
            request_id,
            params: ThreadMemoryModeSetParams {
                thread_id: thread.id,
                mode: ThreadMemoryMode::Disabled,
            },
        })
        .await?;

    let memory_mode = state_db.get_thread_memory_mode(thread_uuid).await?;
    assert_eq!(memory_mode.as_deref(), Some("disabled"));
    Ok(())
}

#[tokio::test]
async fn thread_memory_mode_set_updates_stored_thread_state() -> Result<()> {
    let server = create_mock_responses_server_repeating_assistant("Done").await;
    let bd3_home = TempDir::new()?;
    MockResponsesConfig::new(&server.uri())
        .with_root_config("suppress_unstable_features_warning = true")
        .enable_feature(Feature::Sqlite)
        .write(bd3_home.path())?;
    let state_db = init_state_db(bd3_home.path()).await?;

    let thread_id = create_fake_rollout(
        bd3_home.path(),
        "2025-01-06T08-30-00",
        "2025-01-06T08:30:00Z",
        "Stored thread preview",
        Some("mock_provider"),
        /*git_info*/ None,
    )?;
    let thread_uuid = ThreadId::from_string(&thread_id)?;

    let mut mcp = TestAppServer::builder()
        .with_bd3_home(bd3_home.path())
        .without_auto_env()
        .build_initialized()
        .await?;

    for mode in [ThreadMemoryMode::Disabled, ThreadMemoryMode::Enabled] {
        let _: ThreadMemoryModeSetResponse = mcp
            .request(|request_id| ClientRequest::ThreadMemoryModeSet {
                request_id,
                params: ThreadMemoryModeSetParams {
                    thread_id: thread_id.clone(),
                    mode,
                },
            })
            .await?;
    }

    let memory_mode = state_db.get_thread_memory_mode(thread_uuid).await?;
    assert_eq!(memory_mode.as_deref(), Some("enabled"));
    Ok(())
}

async fn init_state_db(bd3_home: &Path) -> Result<Arc<StateRuntime>> {
    let state_db = StateRuntime::init(
        bd3_state::SqliteConfig::new_for_testing(bd3_home.abs()),
        "mock_provider".into(),
    )
    .await?;
    state_db
        .mark_backfill_complete(/*last_watermark*/ None)
        .await?;
    Ok(state_db)
}
