use std::sync::Arc;

use bd3_config::McpServerTransportConfig;
use bd3_core::McpManager;
use bd3_core::config::Config;
use bd3_core::config::ConfigBuilder;
use bd3_core::plugins_manager_for_config;
use bd3_extension_api::ExtensionRegistryBuilder;
use bd3_extension_api::McpServerContribution;
use bd3_extension_api::McpServerContributionContext;
use bd3_extension_api::McpServerContributor;
use bd3_login::AuthManager;
use bd3_login::Bd3Auth;
use bd3_login::test_support::auth_manager_from_optional_auth;
use bd3_mcp::BD3_APPS_MCP_SERVER_NAME;
use pretty_assertions::assert_eq;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn contributes_hosted_plugin_runtime_without_an_executor() -> TestResult {
    let bd3_home = tempfile::tempdir()?;
    let config = ConfigBuilder::default()
        .bd3_home(bd3_home.path().to_path_buf())
        .fallback_cwd(Some(bd3_home.path().to_path_buf()))
        .cli_overrides(vec![
            ("features.apps".to_string(), true.into()),
            ("chatgpt_base_url".to_string(), "https://chatgpt.com".into()),
        ])
        .build()
        .await?;
    let auth = Bd3Auth::create_dummy_chatgpt_auth_for_testing();
    let manager = installed_manager(&config, Some(auth.clone()));

    let servers = manager.effective_servers(&config, Some(&auth)).await;
    let server = servers
        .get(BD3_APPS_MCP_SERVER_NAME)
        .ok_or("hosted plugin runtime should be contributed as a configured server")?
        .config();
    let McpServerTransportConfig::StreamableHttp { url, .. } = &server.transport else {
        panic!("hosted plugin runtime should use streamable HTTP");
    };
    assert_eq!(url, "https://chatgpt.com/backend-api/ps/mcp");

    Ok(())
}

#[tokio::test]
async fn runtime_overlay_preserves_disabled_server() -> TestResult {
    let bd3_home = tempfile::tempdir()?;
    let config = ConfigBuilder::default()
        .bd3_home(bd3_home.path().to_path_buf())
        .fallback_cwd(Some(bd3_home.path().to_path_buf()))
        .cli_overrides(vec![
            ("features.apps".to_string(), true.into()),
            (
                "mcp_servers.bd3_apps.url".to_string(),
                "https://example.com/mcp".into(),
            ),
            ("mcp_servers.bd3_apps.enabled".to_string(), false.into()),
        ])
        .build()
        .await?;
    let auth = Bd3Auth::create_dummy_chatgpt_auth_for_testing();
    let manager = installed_manager(&config, Some(auth.clone()));

    let servers = manager.effective_servers(&config, Some(&auth)).await;
    let server = servers
        .get(BD3_APPS_MCP_SERVER_NAME)
        .ok_or("hosted plugin runtime should remain configured")?;

    assert!(!server.enabled());
    Ok(())
}

#[tokio::test]
async fn default_fallback_overwrites_reserved_config_without_an_extension() -> TestResult {
    let bd3_home = tempfile::tempdir()?;
    let config = ConfigBuilder::default()
        .bd3_home(bd3_home.path().to_path_buf())
        .fallback_cwd(Some(bd3_home.path().to_path_buf()))
        .cli_overrides(vec![
            ("features.apps".to_string(), true.into()),
            (
                "mcp_servers.bd3_apps.url".to_string(),
                "https://example.com/mcp".into(),
            ),
        ])
        .build()
        .await?;
    let auth = Bd3Auth::create_dummy_chatgpt_auth_for_testing();
    let manager = McpManager::new(Arc::new(plugins_manager_for_config(
        &config,
        AuthManager::from_auth_for_testing(auth.clone()),
    )));

    let servers = manager.effective_servers(&config, Some(&auth)).await;
    let server = servers
        .get(BD3_APPS_MCP_SERVER_NAME)
        .ok_or("default Apps MCP should be present")?
        .config();
    let McpServerTransportConfig::StreamableHttp { url, .. } = &server.transport else {
        panic!("default Apps MCP should use streamable HTTP");
    };
    assert_eq!(url, "https://chatgpt.com/backend-api/ps/mcp");

    Ok(())
}

#[tokio::test]
async fn later_extension_can_remove_same_name_registration() -> TestResult {
    let bd3_home = tempfile::tempdir()?;
    let config = ConfigBuilder::default()
        .bd3_home(bd3_home.path().to_path_buf())
        .fallback_cwd(Some(bd3_home.path().to_path_buf()))
        .cli_overrides(vec![("features.apps".to_string(), true.into())])
        .build()
        .await?;
    let auth = Bd3Auth::create_dummy_chatgpt_auth_for_testing();
    let mut builder = ExtensionRegistryBuilder::new();
    bd3_mcp_extension::install(&mut builder);
    builder.mcp_server_contributor(Arc::new(RemoveBd3Apps));
    let manager = McpManager::new_with_extensions(
        Arc::new(plugins_manager_for_config(
            &config,
            AuthManager::from_auth_for_testing(auth.clone()),
        )),
        Arc::new(builder.build()),
        bd3_core::Bd3AppsToolsCache::default(),
    );

    let servers = manager.effective_servers(&config, Some(&auth)).await;

    assert!(!servers.contains_key(BD3_APPS_MCP_SERVER_NAME));
    Ok(())
}

#[tokio::test]
async fn hosted_apps_mcp_requires_chatgpt_auth() -> TestResult {
    let bd3_home = tempfile::tempdir()?;
    let config = ConfigBuilder::default()
        .bd3_home(bd3_home.path().to_path_buf())
        .fallback_cwd(Some(bd3_home.path().to_path_buf()))
        .cli_overrides(vec![("features.apps".to_string(), true.into())])
        .build()
        .await?;
    let auth = Bd3Auth::from_api_key("test");
    let manager = installed_manager(&config, Some(auth.clone()));

    let servers = manager.effective_servers(&config, Some(&auth)).await;
    assert!(!servers.contains_key(BD3_APPS_MCP_SERVER_NAME));

    Ok(())
}

#[tokio::test]
async fn disabled_apps_remove_reserved_server_config_for_all_hosts() -> TestResult {
    let bd3_home = tempfile::tempdir()?;
    let config = ConfigBuilder::default()
        .bd3_home(bd3_home.path().to_path_buf())
        .fallback_cwd(Some(bd3_home.path().to_path_buf()))
        .cli_overrides(vec![
            ("features.apps".to_string(), false.into()),
            (
                "mcp_servers.bd3_apps.url".to_string(),
                "https://example.com/mcp".into(),
            ),
        ])
        .build()
        .await?;
    let managers = [
        installed_manager(&config, /*auth*/ None),
        McpManager::new(Arc::new(plugins_manager_for_config(
            &config,
            auth_manager_from_optional_auth(/*auth*/ None),
        ))),
    ];
    for manager in managers {
        let servers = manager.runtime_servers(&config).await;
        assert!(!servers.contains_key(BD3_APPS_MCP_SERVER_NAME));
    }
    Ok(())
}

fn installed_manager(config: &Config, auth: Option<Bd3Auth>) -> McpManager {
    let mut builder = ExtensionRegistryBuilder::new();
    bd3_mcp_extension::install(&mut builder);
    McpManager::new_with_extensions(
        Arc::new(plugins_manager_for_config(
            config,
            auth_manager_from_optional_auth(auth),
        )),
        Arc::new(builder.build()),
        bd3_core::Bd3AppsToolsCache::default(),
    )
}

struct RemoveBd3Apps;

impl McpServerContributor<Config> for RemoveBd3Apps {
    fn id(&self) -> &'static str {
        "remove_bd3_apps"
    }

    fn contribute<'a>(
        &'a self,
        _context: McpServerContributionContext<'a, Config>,
    ) -> bd3_extension_api::ExtensionFuture<'a, Vec<McpServerContribution>> {
        Box::pin(async move {
            vec![McpServerContribution::Remove {
                name: BD3_APPS_MCP_SERVER_NAME.to_string(),
            }]
        })
    }
}
