//! Public 3BD model aliases.
//!
//! The 3BD UI uses branded model names while provider requests keep the
//! provider-owned identifiers. Unknown names deliberately pass through so
//! custom providers and future models remain usable.

/// Convert a 3BD-facing model alias into the provider-owned model identifier.
pub fn provider_model_name(model: &str) -> String {
    match model {
        "3BD4" => "gpt-5.4".to_string(),
        "3BD5" => "gpt-5.5".to_string(),
        "3BD6SOL" => "gpt-5.6-sol".to_string(),
        "3BD6TERRA" => "gpt-5.6-terra".to_string(),
        "3BD6LUNA" => "gpt-5.6-luna".to_string(),
        "3BD51" => "gpt-5.1-codex".to_string(),
        "3BD51MAX" => "gpt-5.1-codex-max".to_string(),
        _ => model.to_string(),
    }
}

/// Convert a provider-owned model identifier into its 3BD-facing display name.
pub fn display_model_name(model: &str) -> String {
    match model {
        "gpt-5.4" => "3BD4".to_string(),
        "gpt-5.5" => "3BD5".to_string(),
        "gpt-5.6-sol" => "3BD6SOL".to_string(),
        "gpt-5.6-terra" => "3BD6TERRA".to_string(),
        "gpt-5.6-luna" => "3BD6LUNA".to_string(),
        "gpt-5.1-codex" => "3BD51".to_string(),
        "gpt-5.1-codex-max" => "3BD51MAX".to_string(),
        _ => model.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{display_model_name, provider_model_name};

    #[test]
    fn maps_public_aliases_to_provider_ids() {
        assert_eq!(provider_model_name("3BD5"), "gpt-5.5");
        assert_eq!(provider_model_name("3BD51MAX"), "gpt-5.1-codex-max");
    }

    #[test]
    fn maps_provider_ids_to_public_aliases() {
        assert_eq!(display_model_name("gpt-5.5"), "3BD5");
        assert_eq!(display_model_name("gpt-5.6-terra"), "3BD6TERRA");
    }

    #[test]
    fn preserves_unknown_models() {
        assert_eq!(provider_model_name("custom-model"), "custom-model");
        assert_eq!(display_model_name("custom-model"), "custom-model");
    }
}
