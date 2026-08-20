use anyhow::Context;
use anyhow::Result;
use bd3_cloud_config::cloud_config_bundle_loader_for_storage;
use bd3_config::CloudConfigBundleLoader;
use bd3_config::ConfigLoadOptions;
use bd3_core::config::Config;
use bd3_core::config::ConfigBuilder;
use bd3_core::config::LoaderOverrides;
use bd3_core::config::bootstrap_auth_config;
use bd3_core::config::find_bd3_home;
use bd3_core::config::load_config_toml_with_layer_stack;
use bd3_utils_absolute_path::AbsolutePathBuf;
use bd3_utils_cli::CliConfigOverrides;

pub(crate) async fn load_config(
    config_overrides: &CliConfigOverrides,
    loader_overrides: LoaderOverrides,
) -> Result<Config> {
    let cli_overrides = config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;
    let bd3_home = find_bd3_home().context("failed to resolve BD3_HOME")?;
    let cwd = AbsolutePathBuf::current_dir().context("failed to resolve current directory")?;
    let bootstrap_config = load_config_toml_with_layer_stack(
        bd3_home.as_path(),
        Some(&cwd),
        cli_overrides.clone(),
        ConfigLoadOptions {
            loader_overrides: loader_overrides.clone(),
            strict_config: false,
            cloud_config_bundle: CloudConfigBundleLoader::default(),
        },
    )
    .await
    .context("failed to load bootstrap configuration")?;
    let cloud_config_bundle = cloud_config_bundle_loader_for_storage(
        bootstrap_auth_config(bd3_home.as_path(), &bootstrap_config)
            .context("failed to resolve cloud configuration authentication")?,
        /*enable_bd3_api_key_env*/ false,
    )
    .await
    .context("failed to initialize cloud configuration authentication")?;

    ConfigBuilder::default()
        .bd3_home(bd3_home.to_path_buf())
        .cli_overrides(cli_overrides)
        .loader_overrides(loader_overrides)
        .cloud_config_bundle(cloud_config_bundle)
        .build()
        .await
        .context("failed to load configuration")
}
