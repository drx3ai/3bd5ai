use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use http::header::HeaderMap;

use bd3_core::config::Config;
use bd3_http_client::HttpClientFactory;
use bd3_http_client::OutboundProxyPolicy;
use bd3_login::AuthManager;
use std::sync::Arc;

pub fn set_user_agent_suffix(suffix: &str) {
    if let Ok(mut guard) = bd3_login::default_client::USER_AGENT_SUFFIX.lock() {
        guard.replace(suffix.to_string());
    }
}

pub fn append_error_log(message: impl AsRef<str>) {
    let ts = Utc::now().to_rfc3339();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("error.log")
    {
        use std::io::Write as _;
        let _ = writeln!(f, "[{ts}] {}", message.as_ref());
    }
}

/// Normalize the configured base URL to a canonical form used by the backend client.
/// - trims trailing '/'
/// - appends '/backend-api' for ChatGPT hosts when missing
pub fn normalize_base_url(input: &str) -> String {
    let mut base_url = input.to_string();
    while base_url.ends_with('/') {
        base_url.pop();
    }
    if (base_url.starts_with("https://chatgpt.com")
        || base_url.starts_with("https://chat.openai.com"))
        && !base_url.contains("/backend-api")
    {
        base_url = format!("{base_url}/backend-api");
    }
    base_url
}

pub async fn load_auth_manager(
    chatgpt_base_url: Option<String>,
) -> (Option<Arc<AuthManager>>, HttpClientFactory) {
    // TODO: pass in cli overrides once cloud tasks properly support them.
    let config = match Config::load_with_cli_overrides(Vec::new()).await {
        Ok(config) => config,
        Err(error) => {
            append_error_log(format!(
                "failed to load auth config; using transport-default proxy handling: {error}"
            ));
            let http_client_factory = HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault);
            return (None, http_client_factory);
        }
    };
    let http_client_factory = config.http_client_factory();
    let mut auth_config = config.auth_config();
    auth_config.chatgpt_base_url = chatgpt_base_url.or(Some(config.chatgpt_base_url.clone()));
    let auth_manager = match AuthManager::shared_from_auth_config(
        auth_config,
        /*enable_bd3_api_key_env*/ false,
    )
    .await
    {
        Ok(auth_manager) => auth_manager,
        Err(error) => {
            append_error_log(format!("failed to load auth: {error}"));
            return (None, http_client_factory);
        }
    };
    (Some(auth_manager), http_client_factory)
}

/// Build headers for ChatGPT-backed requests: `User-Agent`, optional `Authorization`,
/// and optional `ChatGPT-Account-Id`.
pub async fn build_chatgpt_headers() -> HeaderMap {
    use http::header::HeaderValue;
    use http::header::USER_AGENT;

    set_user_agent_suffix("bd3_cloud_tasks_tui");
    let ua = bd3_login::default_client::get_bd3_user_agent();
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&ua).unwrap_or(HeaderValue::from_static("bd3-cli")),
    );
    if let Some(am) = load_auth_manager(/*chatgpt_base_url*/ None).await.0
        && let Some(auth) = am.auth().await
        && auth.uses_bd3_backend()
    {
        headers.extend(bd3_model_provider::auth_provider_from_auth(&auth).to_auth_headers());
    }
    headers
}

/// Construct a browser-friendly task URL for the given backend base URL.
pub fn task_url(base_url: &str, task_id: &str) -> String {
    let normalized = normalize_base_url(base_url);
    if let Some(root) = normalized.strip_suffix("/backend-api") {
        return format!("{root}/3bd/tasks/{task_id}");
    }
    if let Some(root) = normalized.strip_suffix("/api/3bd") {
        return format!("{root}/3bd/tasks/{task_id}");
    }
    if normalized.ends_with("/3bd") {
        return format!("{normalized}/tasks/{task_id}");
    }
    format!("{normalized}/3bd/tasks/{task_id}")
}

pub fn format_relative_time(reference: DateTime<Utc>, ts: DateTime<Utc>) -> String {
    let mut secs = (reference - ts).num_seconds();
    if secs < 0 {
        secs = 0;
    }
    if secs < 60 {
        return format!("{secs}s ago");
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("{hours}h ago");
    }
    let local = ts.with_timezone(&Local);
    local.format("%b %e %H:%M").to_string()
}

pub fn format_relative_time_now(ts: DateTime<Utc>) -> String {
    format_relative_time(Utc::now(), ts)
}
