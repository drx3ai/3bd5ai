use crate::bespoke_event_handling::apply_bespoke_event_handling;
use crate::command_exec::CommandExecManager;
use crate::command_exec::StartCommandExecParams;
use crate::config_manager::ConfigManager;
use crate::error_code::INPUT_TOO_LARGE_ERROR_CODE;
use crate::error_code::invalid_params;
use crate::models::supported_models;
use crate::outgoing_message::ConnectionId;
use crate::outgoing_message::ConnectionRequestId;
use crate::outgoing_message::OutgoingMessageSender;
use crate::outgoing_message::RequestContext;
use crate::outgoing_message::ThreadScopedOutgoingMessageSender;
use crate::skills_watcher::SkillsWatcher;
use crate::thread_status::ThreadWatchManager;
use crate::thread_status::resolve_thread_status;
use chrono::Duration as ChronoDuration;
use chrono::SecondsFormat;
use bd3_analytics::AnalyticsEventsClient;
use bd3_analytics::AnalyticsJsonRpcError;
use bd3_analytics::InputError;
use bd3_analytics::TurnSteerRequestError;
use bd3_app_server_protocol::Account;
use bd3_app_server_protocol::AccountLoginCompletedNotification;
use bd3_app_server_protocol::AccountTokenUsageDailyBucket;
use bd3_app_server_protocol::AccountTokenUsageSummary;
use bd3_app_server_protocol::AccountUpdatedNotification;
use bd3_app_server_protocol::AddCreditsNudgeCreditType;
use bd3_app_server_protocol::AddCreditsNudgeEmailStatus;
use bd3_app_server_protocol::AdditionalContextEntry;
use bd3_app_server_protocol::AdditionalContextKind;
use bd3_app_server_protocol::AppListUpdatedNotification;
use bd3_app_server_protocol::AppSummary;
use bd3_app_server_protocol::AppTemplateSummary;
use bd3_app_server_protocol::AppTemplateUnavailableReason;
use bd3_app_server_protocol::AppsInstalledParams;
use bd3_app_server_protocol::AppsInstalledResponse;
use bd3_app_server_protocol::AppsListParams;
use bd3_app_server_protocol::AppsListResponse;
use bd3_app_server_protocol::AppsReadParams;
use bd3_app_server_protocol::AppsReadResponse;
use bd3_app_server_protocol::AskForApproval;
use bd3_app_server_protocol::AuthMode;
use bd3_app_server_protocol::CancelLoginAccountParams;
use bd3_app_server_protocol::CancelLoginAccountResponse;
use bd3_app_server_protocol::CancelLoginAccountStatus;
use bd3_app_server_protocol::ClientInfo;
use bd3_app_server_protocol::ClientRequest;
use bd3_app_server_protocol::ClientResponsePayload;
use bd3_app_server_protocol::Bd3ErrorInfo;
use bd3_app_server_protocol::CollaborationModeListParams;
use bd3_app_server_protocol::CollaborationModeListResponse;
use bd3_app_server_protocol::CommandExecParams;
use bd3_app_server_protocol::CommandExecResizeParams;
use bd3_app_server_protocol::CommandExecTerminateParams;
use bd3_app_server_protocol::CommandExecWriteParams;
use bd3_app_server_protocol::ConfigWarningNotification;
use bd3_app_server_protocol::ConsumeAccountRateLimitResetCreditOutcome;
use bd3_app_server_protocol::ConsumeAccountRateLimitResetCreditParams;
use bd3_app_server_protocol::ConsumeAccountRateLimitResetCreditResponse;
use bd3_app_server_protocol::ConversationGitInfo;
use bd3_app_server_protocol::ConversationSummary;
use bd3_app_server_protocol::DeprecationNoticeNotification;
use bd3_app_server_protocol::DynamicToolFunctionSpec;
use bd3_app_server_protocol::DynamicToolNamespaceTool;
use bd3_app_server_protocol::DynamicToolSpec;
use bd3_app_server_protocol::EnvironmentAddParams;
use bd3_app_server_protocol::EnvironmentAddResponse;
use bd3_app_server_protocol::EnvironmentInfoParams;
use bd3_app_server_protocol::EnvironmentInfoResponse;
use bd3_app_server_protocol::EnvironmentShellInfo;
use bd3_app_server_protocol::EnvironmentStatusKind;
use bd3_app_server_protocol::EnvironmentStatusParams;
use bd3_app_server_protocol::EnvironmentStatusResponse;
use bd3_app_server_protocol::ExperimentalFeature as ApiExperimentalFeature;
use bd3_app_server_protocol::ExperimentalFeatureListParams;
use bd3_app_server_protocol::ExperimentalFeatureListResponse;
use bd3_app_server_protocol::ExperimentalFeatureStage as ApiExperimentalFeatureStage;
use bd3_app_server_protocol::FeedbackUploadParams;
use bd3_app_server_protocol::FeedbackUploadResponse;
use bd3_app_server_protocol::GetAccountParams;
use bd3_app_server_protocol::GetAccountRateLimitsResponse;
use bd3_app_server_protocol::GetAccountResponse;
use bd3_app_server_protocol::GetAccountTokenUsageParams;
use bd3_app_server_protocol::GetAccountTokenUsageResponse;
use bd3_app_server_protocol::GetAuthStatusParams;
use bd3_app_server_protocol::GetAuthStatusResponse;
use bd3_app_server_protocol::GetConversationSummaryParams;
use bd3_app_server_protocol::GetConversationSummaryResponse;
use bd3_app_server_protocol::GetWorkspaceMessagesResponse;
use bd3_app_server_protocol::GitDiffToRemoteParams;
use bd3_app_server_protocol::GitDiffToRemoteResponse;
use bd3_app_server_protocol::GitInfo as ApiGitInfo;
use bd3_app_server_protocol::HookHandlerMetadata;
use bd3_app_server_protocol::HookMetadata;
use bd3_app_server_protocol::HooksListParams;
use bd3_app_server_protocol::HooksListResponse;
use bd3_app_server_protocol::InitializeParams;
use bd3_app_server_protocol::InitializeResponse;
use bd3_app_server_protocol::InstalledApp;
use bd3_app_server_protocol::JSONRPCErrorError;
use bd3_app_server_protocol::ListMcpServerStatusParams;
use bd3_app_server_protocol::ListMcpServerStatusResponse;
use bd3_app_server_protocol::LoginAccountParams;
use bd3_app_server_protocol::LoginAccountResponse;
use bd3_app_server_protocol::LoginApiKeyParams;
use bd3_app_server_protocol::LoginAppBrand;
use bd3_app_server_protocol::LogoutAccountResponse;
use bd3_app_server_protocol::MarketplaceAddParams;
use bd3_app_server_protocol::MarketplaceAddResponse;
use bd3_app_server_protocol::MarketplaceInterface;
use bd3_app_server_protocol::MarketplaceRemoveParams;
use bd3_app_server_protocol::MarketplaceRemoveResponse;
use bd3_app_server_protocol::MarketplaceUpgradeErrorInfo;
use bd3_app_server_protocol::MarketplaceUpgradeParams;
use bd3_app_server_protocol::MarketplaceUpgradeResponse;
use bd3_app_server_protocol::McpResourceReadParams;
use bd3_app_server_protocol::McpResourceReadResponse;
use bd3_app_server_protocol::McpServerOauthClientRegistration;
use bd3_app_server_protocol::McpServerOauthLoginCompletedNotification;
use bd3_app_server_protocol::McpServerOauthLoginParams;
use bd3_app_server_protocol::McpServerOauthLoginResponse;
use bd3_app_server_protocol::McpServerRefreshResponse;
use bd3_app_server_protocol::McpServerStatus;
use bd3_app_server_protocol::McpServerStatusDetail;
use bd3_app_server_protocol::McpServerToolCallParams;
use bd3_app_server_protocol::McpServerToolCallResponse;
use bd3_app_server_protocol::MemoryResetResponse;
use bd3_app_server_protocol::MockExperimentalMethodParams;
use bd3_app_server_protocol::MockExperimentalMethodResponse;
use bd3_app_server_protocol::ModelListParams;
use bd3_app_server_protocol::ModelListResponse;
use bd3_app_server_protocol::PermissionProfileListParams;
use bd3_app_server_protocol::PermissionProfileListResponse;
use bd3_app_server_protocol::PermissionProfileSummary;
use bd3_app_server_protocol::PluginDetail;
use bd3_app_server_protocol::PluginInstallParams;
use bd3_app_server_protocol::PluginInstallResponse;
use bd3_app_server_protocol::PluginInstalledParams;
use bd3_app_server_protocol::PluginInstalledResponse;
use bd3_app_server_protocol::PluginInterface;
use bd3_app_server_protocol::PluginListMarketplaceKind;
use bd3_app_server_protocol::PluginListParams;
use bd3_app_server_protocol::PluginListResponse;
use bd3_app_server_protocol::PluginMarketplaceEntry;
use bd3_app_server_protocol::PluginReadParams;
use bd3_app_server_protocol::PluginReadResponse;
use bd3_app_server_protocol::PluginShareCheckoutParams;
use bd3_app_server_protocol::PluginShareCheckoutResponse;
use bd3_app_server_protocol::PluginShareContext;
use bd3_app_server_protocol::PluginShareDeleteParams;
use bd3_app_server_protocol::PluginShareDeleteResponse;
use bd3_app_server_protocol::PluginShareDiscoverability;
use bd3_app_server_protocol::PluginShareListItem;
use bd3_app_server_protocol::PluginShareListParams;
use bd3_app_server_protocol::PluginShareListResponse;
use bd3_app_server_protocol::PluginSharePrincipal;
use bd3_app_server_protocol::PluginSharePrincipalType;
use bd3_app_server_protocol::PluginShareSaveParams;
use bd3_app_server_protocol::PluginShareSaveResponse;
use bd3_app_server_protocol::PluginShareTarget;
use bd3_app_server_protocol::PluginShareUpdateDiscoverability;
use bd3_app_server_protocol::PluginShareUpdateTargetsParams;
use bd3_app_server_protocol::PluginShareUpdateTargetsResponse;
use bd3_app_server_protocol::PluginSkillReadParams;
use bd3_app_server_protocol::PluginSkillReadResponse;
use bd3_app_server_protocol::PluginSource;
use bd3_app_server_protocol::PluginSummary;
use bd3_app_server_protocol::PluginUninstallParams;
use bd3_app_server_protocol::PluginUninstallResponse;
use bd3_app_server_protocol::RateLimitResetCredit;
use bd3_app_server_protocol::RateLimitResetCreditStatus;
use bd3_app_server_protocol::RateLimitResetCreditsSummary;
use bd3_app_server_protocol::RateLimitResetType;
use bd3_app_server_protocol::RequestId;
use bd3_app_server_protocol::ReviewDelivery as ApiReviewDelivery;
use bd3_app_server_protocol::ReviewStartParams;
use bd3_app_server_protocol::ReviewStartResponse;
use bd3_app_server_protocol::ReviewTarget as ApiReviewTarget;
use bd3_app_server_protocol::SandboxMode;
use bd3_app_server_protocol::SendAddCreditsNudgeEmailParams;
use bd3_app_server_protocol::SendAddCreditsNudgeEmailResponse;
use bd3_app_server_protocol::ServerNotification;
use bd3_app_server_protocol::ServerRequestResolvedNotification;
use bd3_app_server_protocol::SkillSummary;
use bd3_app_server_protocol::SkillsConfigWriteParams;
use bd3_app_server_protocol::SkillsConfigWriteResponse;
use bd3_app_server_protocol::SkillsExtraRootsSetParams;
use bd3_app_server_protocol::SkillsExtraRootsSetResponse;
use bd3_app_server_protocol::SkillsListParams;
use bd3_app_server_protocol::SkillsListResponse;
use bd3_app_server_protocol::SortDirection;
use bd3_app_server_protocol::Thread;
use bd3_app_server_protocol::ThreadApproveGuardianDeniedActionParams;
use bd3_app_server_protocol::ThreadApproveGuardianDeniedActionResponse;
use bd3_app_server_protocol::ThreadArchiveParams;
use bd3_app_server_protocol::ThreadArchiveResponse;
use bd3_app_server_protocol::ThreadArchivedNotification;
use bd3_app_server_protocol::ThreadBackgroundTerminal;
use bd3_app_server_protocol::ThreadBackgroundTerminalsCleanParams;
use bd3_app_server_protocol::ThreadBackgroundTerminalsCleanResponse;
use bd3_app_server_protocol::ThreadBackgroundTerminalsListParams;
use bd3_app_server_protocol::ThreadBackgroundTerminalsListResponse;
use bd3_app_server_protocol::ThreadBackgroundTerminalsTerminateParams;
use bd3_app_server_protocol::ThreadBackgroundTerminalsTerminateResponse;
use bd3_app_server_protocol::ThreadClosedNotification;
use bd3_app_server_protocol::ThreadCompactStartParams;
use bd3_app_server_protocol::ThreadCompactStartResponse;
use bd3_app_server_protocol::ThreadDecrementElicitationParams;
use bd3_app_server_protocol::ThreadDecrementElicitationResponse;
use bd3_app_server_protocol::ThreadDeleteParams;
use bd3_app_server_protocol::ThreadDeleteResponse;
use bd3_app_server_protocol::ThreadDeletedNotification;
use bd3_app_server_protocol::ThreadForkParams;
use bd3_app_server_protocol::ThreadForkResponse;
use bd3_app_server_protocol::ThreadGoal;
use bd3_app_server_protocol::ThreadGoalClearParams;
use bd3_app_server_protocol::ThreadGoalClearResponse;
use bd3_app_server_protocol::ThreadGoalClearedNotification;
use bd3_app_server_protocol::ThreadGoalGetParams;
use bd3_app_server_protocol::ThreadGoalGetResponse;
use bd3_app_server_protocol::ThreadGoalSetParams;
use bd3_app_server_protocol::ThreadGoalSetResponse;
use bd3_app_server_protocol::ThreadGoalStatus;
use bd3_app_server_protocol::ThreadGoalUpdatedNotification;
use bd3_app_server_protocol::ThreadHistoryBuilder;
#[cfg(test)]
use bd3_app_server_protocol::ThreadHistoryMode;
use bd3_app_server_protocol::ThreadIncrementElicitationParams;
use bd3_app_server_protocol::ThreadIncrementElicitationResponse;
use bd3_app_server_protocol::ThreadInjectItemsParams;
use bd3_app_server_protocol::ThreadInjectItemsResponse;
use bd3_app_server_protocol::ThreadItem;
use bd3_app_server_protocol::ThreadItemEntry;
use bd3_app_server_protocol::ThreadItemsListParams;
use bd3_app_server_protocol::ThreadItemsListResponse;
use bd3_app_server_protocol::ThreadListCwdFilter;
use bd3_app_server_protocol::ThreadListParams;
use bd3_app_server_protocol::ThreadListResponse;
use bd3_app_server_protocol::ThreadLoadedListParams;
use bd3_app_server_protocol::ThreadLoadedListResponse;
use bd3_app_server_protocol::ThreadMemoryModeSetParams;
use bd3_app_server_protocol::ThreadMemoryModeSetResponse;
use bd3_app_server_protocol::ThreadMetadataGitInfoUpdateParams;
use bd3_app_server_protocol::ThreadMetadataUpdateParams;
use bd3_app_server_protocol::ThreadMetadataUpdateResponse;
use bd3_app_server_protocol::ThreadNameUpdatedNotification;
use bd3_app_server_protocol::ThreadProjectUpdatedNotification;
use bd3_app_server_protocol::ThreadReadParams;
use bd3_app_server_protocol::ThreadReadResponse;
use bd3_app_server_protocol::ThreadRealtimeAppendAudioParams;
use bd3_app_server_protocol::ThreadRealtimeAppendAudioResponse;
use bd3_app_server_protocol::ThreadRealtimeAppendSpeechParams;
use bd3_app_server_protocol::ThreadRealtimeAppendSpeechResponse;
use bd3_app_server_protocol::ThreadRealtimeAppendTextParams;
use bd3_app_server_protocol::ThreadRealtimeAppendTextResponse;
use bd3_app_server_protocol::ThreadRealtimeListVoicesResponse;
use bd3_app_server_protocol::ThreadRealtimeStartParams;
use bd3_app_server_protocol::ThreadRealtimeStartResponse;
use bd3_app_server_protocol::ThreadRealtimeStartTransport;
use bd3_app_server_protocol::ThreadRealtimeStopParams;
use bd3_app_server_protocol::ThreadRealtimeStopResponse;
use bd3_app_server_protocol::ThreadResumeInitialTurnsPageParams;
use bd3_app_server_protocol::ThreadResumeParams;
use bd3_app_server_protocol::ThreadResumeResponse;
use bd3_app_server_protocol::ThreadRollbackParams;
use bd3_app_server_protocol::ThreadSearchOccurrence;
use bd3_app_server_protocol::ThreadSearchOccurrencesParams;
use bd3_app_server_protocol::ThreadSearchOccurrencesResponse;
use bd3_app_server_protocol::ThreadSearchParams;
use bd3_app_server_protocol::ThreadSearchResponse;
use bd3_app_server_protocol::ThreadSearchResult;
use bd3_app_server_protocol::ThreadSearchSortKey;
use bd3_app_server_protocol::ThreadSearchTextRange;
use bd3_app_server_protocol::ThreadSetNameParams;
use bd3_app_server_protocol::ThreadSetNameResponse;
use bd3_app_server_protocol::ThreadSettings;
use bd3_app_server_protocol::ThreadSettingsUpdateParams;
use bd3_app_server_protocol::ThreadSettingsUpdateResponse;
use bd3_app_server_protocol::ThreadShellCommandParams;
use bd3_app_server_protocol::ThreadShellCommandResponse;
use bd3_app_server_protocol::ThreadSortKey;
use bd3_app_server_protocol::ThreadSourceKind;
use bd3_app_server_protocol::ThreadStartParams;
use bd3_app_server_protocol::ThreadStartResponse;
use bd3_app_server_protocol::ThreadStartedNotification;
use bd3_app_server_protocol::ThreadStatus;
use bd3_app_server_protocol::ThreadTurnsListParams;
use bd3_app_server_protocol::ThreadTurnsListResponse;
use bd3_app_server_protocol::ThreadUnarchiveParams;
use bd3_app_server_protocol::ThreadUnarchiveResponse;
use bd3_app_server_protocol::ThreadUnarchivedNotification;
use bd3_app_server_protocol::ThreadUnsubscribeParams;
use bd3_app_server_protocol::ThreadUnsubscribeResponse;
use bd3_app_server_protocol::ThreadUnsubscribeStatus;
use bd3_app_server_protocol::Turn;
use bd3_app_server_protocol::TurnEnvironmentParams;
use bd3_app_server_protocol::TurnError;
use bd3_app_server_protocol::TurnInterruptParams;
use bd3_app_server_protocol::TurnInterruptResponse;
use bd3_app_server_protocol::TurnItemsView;
use bd3_app_server_protocol::TurnStartParams;
use bd3_app_server_protocol::TurnStartResponse;
use bd3_app_server_protocol::TurnStatus;
use bd3_app_server_protocol::TurnSteerParams;
use bd3_app_server_protocol::TurnSteerResponse;
use bd3_app_server_protocol::UserInput as V2UserInput;
use bd3_app_server_protocol::WindowsSandboxReadiness;
use bd3_app_server_protocol::WindowsSandboxReadinessResponse;
use bd3_app_server_protocol::WindowsSandboxSetupCompletedNotification;
use bd3_app_server_protocol::WindowsSandboxSetupMode;
use bd3_app_server_protocol::WindowsSandboxSetupStartParams;
use bd3_app_server_protocol::WindowsSandboxSetupStartResponse;
use bd3_app_server_protocol::WorkspaceMessage;
use bd3_app_server_protocol::WorkspaceMessageType;
use bd3_arg0::Arg0DispatchPaths;
use bd3_backend_client::AddCreditsNudgeCreditType as BackendAddCreditsNudgeCreditType;
use bd3_backend_client::Client as BackendClient;
use bd3_backend_client::Bd3WorkspaceMessage as BackendWorkspaceMessage;
use bd3_backend_client::Bd3WorkspaceMessageType as BackendWorkspaceMessageType;
use bd3_backend_client::Bd3WorkspaceMessagesResponse as BackendWorkspaceMessagesResponse;
use bd3_backend_client::ConsumeRateLimitResetCreditCode as BackendConsumeRateLimitResetCreditCode;
use bd3_backend_client::RateLimitResetCreditDetails as BackendRateLimitResetCreditDetails;
use bd3_backend_client::RateLimitResetCreditsDetails as BackendRateLimitResetCreditsDetails;
use bd3_backend_client::RequestError as BackendRequestError;
use bd3_backend_client::TokenUsageProfile;
use bd3_chatgpt::connectors;
use bd3_config::CloudConfigBundleLoadError;
use bd3_config::CloudConfigBundleLoadErrorCode;
use bd3_config::ConfigLayerStack;
use bd3_config::loader::project_trust_key;
use bd3_config::types::McpServerTransportConfig;
use bd3_connectors::AppInfo;
use bd3_core::Bd3Thread;
use bd3_core::Bd3ThreadSettingsOverrides;
use bd3_core::ForkSnapshot;
use bd3_core::McpManager;
use bd3_core::NewThread;
use bd3_core::NotSubmittedReason;
#[cfg(test)]
use bd3_core::SessionMeta;
use bd3_core::StartThreadOptions;
use bd3_core::SteerSubmission;
use bd3_core::ThreadConfigSnapshot;
use bd3_core::ThreadManager;
use bd3_core::TurnInput;
use bd3_core::TurnInputRequest;
use bd3_core::TurnInputSubmission;
use bd3_core::TurnStartOptions;
use bd3_core::config::Config;
use bd3_core::config::ConfigOverrides;
use bd3_core::config::NetworkProxyAuditMetadata;
use bd3_core::config::edit::ConfigEdit;
use bd3_core::config::edit::ConfigEditsBuilder;
use bd3_core::connectors::AccessibleConnectorsStatus;
use bd3_core::exec::ExecCapturePolicy;
use bd3_core::exec::ExecExpiration;
use bd3_core::exec::ExecParams;
use bd3_core::exec_env::create_env;
use bd3_core::path_utils;
#[cfg(test)]
use bd3_core::read_head_for_summary;
use bd3_core::sandboxing::SandboxPermissions;
use bd3_core::truncate_rollout_after_turn_id;
use bd3_core::truncate_rollout_before_turn_id;
use bd3_core::windows_sandbox::WindowsSandboxLevelExt;
use bd3_core::windows_sandbox::WindowsSandboxSetupMode as CoreWindowsSandboxSetupMode;
use bd3_core::windows_sandbox::WindowsSandboxSetupRequest;
use bd3_core::windows_sandbox::sandbox_setup_is_complete;
use bd3_core_plugins::PluginInstallError as CorePluginInstallError;
use bd3_core_plugins::PluginInstallRequest;
use bd3_core_plugins::PluginReadRequest;
use bd3_core_plugins::PluginUninstallError as CorePluginUninstallError;
use bd3_core_plugins::PluginsManager;
use bd3_core_plugins::loader::load_plugin_apps;
use bd3_core_plugins::manifest::PluginManifestInterface;
use bd3_core_plugins::marketplace::MarketplaceError;
use bd3_core_plugins::marketplace::MarketplacePluginSource;
use bd3_core_plugins::marketplace_add::MarketplaceAddError;
use bd3_core_plugins::marketplace_add::MarketplaceAddRequest;
use bd3_core_plugins::marketplace_add::add_marketplace as add_marketplace_to_bd3_home;
use bd3_core_plugins::marketplace_remove::MarketplaceRemoveError;
use bd3_core_plugins::marketplace_remove::MarketplaceRemoveRequest as CoreMarketplaceRemoveRequest;
use bd3_core_plugins::marketplace_remove::remove_marketplace;
use bd3_core_plugins::remote::RemoteMarketplace;
use bd3_core_plugins::remote::RemoteMarketplaceSource;
use bd3_core_plugins::remote::RemotePluginCatalogError;
use bd3_core_plugins::remote::RemotePluginDetail as RemoteCatalogPluginDetail;
use bd3_core_plugins::remote::RemotePluginServiceConfig;
use bd3_core_plugins::remote::RemotePluginShareContext as RemoteCatalogPluginShareContext;
use bd3_core_plugins::remote::RemotePluginShareSummary as RemoteCatalogPluginShareSummary;
use bd3_core_plugins::remote::RemotePluginSummary as RemoteCatalogPluginSummary;
use bd3_exec_server::EnvironmentManager;
use bd3_exec_server::EnvironmentObservedStatus;
use bd3_exec_server::LOCAL_ENVIRONMENT_ID;
use bd3_exec_server::LOCAL_FS;
use bd3_features::FEATURES;
use bd3_features::Feature;
use bd3_features::Stage;
use bd3_feedback::Bd3Feedback;
use bd3_feedback::FeedbackAttachmentPath;
use bd3_feedback::FeedbackUploadOptions;
use bd3_git_utils::git_diff_to_remote;
use bd3_git_utils::resolve_root_git_project_for_trust;
use bd3_login::AuthManager;
use bd3_login::BD3_OPEN_APP_URL;
use bd3_login::Bd3Auth;
use bd3_login::LoginSuccessPage;
use bd3_login::LoginSuccessPageBrand;
use bd3_login::ServerOptions as LoginServerOptions;
use bd3_login::ShutdownHandle;
use bd3_login::complete_device_code_login;
use bd3_login::login_with_api_key;
use bd3_login::login_with_bedrock_api_key;
use bd3_login::oauth_client_id;
use bd3_login::request_device_code;
use bd3_login::run_login_server;
use bd3_mcp::McpRuntimeContext;
use bd3_mcp::McpServerStatusSnapshot;
use bd3_mcp::McpSnapshotDetail;
use bd3_mcp::collect_mcp_server_status_snapshot_with_detail;
use bd3_mcp::discover_supported_scopes;
use bd3_mcp::read_mcp_resource as read_mcp_resource_without_thread;
use bd3_mcp::resolve_oauth_scopes;
use bd3_memories_write::clear_memory_roots_contents;
use bd3_model_provider::create_model_provider;
use bd3_models_manager::collaboration_mode_presets::builtin_collaboration_mode_presets;
use bd3_protocol::ThreadId;
use bd3_protocol::config_types::CollaborationMode;
use bd3_protocol::config_types::ForcedLoginMethod;
use bd3_protocol::config_types::Personality;
use bd3_protocol::config_types::ReasoningSummary;
use bd3_protocol::config_types::TrustLevel;
use bd3_protocol::config_types::WindowsSandboxLevel;
use bd3_protocol::error::Bd3Err;
use bd3_protocol::error::Result as Bd3Result;
#[cfg(test)]
use bd3_protocol::items::TurnItem;
use bd3_protocol::models::ResponseItem;
use bd3_protocol::openai_models::ReasoningEffort;
use bd3_protocol::protocol::AgentStatus;
use bd3_protocol::protocol::ConversationAudioParams;
use bd3_protocol::protocol::ConversationSpeechParams;
use bd3_protocol::protocol::ConversationStartParams;
use bd3_protocol::protocol::ConversationStartTransport;
use bd3_protocol::protocol::ConversationTextParams;
use bd3_protocol::protocol::EnvironmentConfigState;
use bd3_protocol::protocol::EventMsg;
#[cfg(test)]
use bd3_protocol::protocol::GitInfo as CoreGitInfo;
use bd3_protocol::protocol::McpAuthStatus as CoreMcpAuthStatus;
use bd3_protocol::protocol::Op;
use bd3_protocol::protocol::RealtimeVoicesList;
use bd3_protocol::protocol::ReviewDelivery as CoreReviewDelivery;
use bd3_protocol::protocol::ReviewRequest;
use bd3_protocol::protocol::ReviewTarget as CoreReviewTarget;
use bd3_protocol::protocol::SessionConfiguredEvent;
#[cfg(test)]
use bd3_protocol::protocol::SessionMetaLine;
use bd3_protocol::protocol::TurnEnvironmentSelection;
use bd3_protocol::protocol::TurnEnvironmentSelections;
use bd3_protocol::protocol::W3cTraceContext;
use bd3_protocol::protocol::strip_user_message_prefix;
use bd3_protocol::user_input::MAX_USER_INPUT_TEXT_CHARS;
use bd3_protocol::user_input::UserInput as CoreInputItem;
use bd3_rmcp_client::McpOAuthClientRegistration;
use bd3_rmcp_client::StreamableHttpRedirectMode;
use bd3_rmcp_client::perform_oauth_login_return_url;
use bd3_rollout::InitialHistory;
use bd3_rollout::ResumedHistory;
use bd3_rollout::RolloutItem;
use bd3_rollout::is_persisted_rollout_item;
use bd3_rollout::state_db::StateDbHandle;
use bd3_rollout::state_db::reconcile_rollout;
use bd3_state::ThreadMetadata;
use bd3_state::log_db::LogDbLayer;
use bd3_thread_store::ArchiveThreadParams as StoreArchiveThreadParams;
use bd3_thread_store::ArchiveThreadsParams as StoreArchiveThreadsParams;
use bd3_thread_store::ClearableField as StoreClearableField;
use bd3_thread_store::DeleteThreadsParams as StoreDeleteThreadsParams;
use bd3_thread_store::GitInfoPatch as StoreGitInfoPatch;
use bd3_thread_store::ItemSortKey as StoreItemSortKey;
use bd3_thread_store::ListItemsParams as StoreListItemsParams;
use bd3_thread_store::ListThreadsParams as StoreListThreadsParams;
use bd3_thread_store::ListTurnsParams as StoreListTurnsParams;
use bd3_thread_store::LoadThreadHistoryParams as StoreLoadThreadHistoryParams;
use bd3_thread_store::LocalThreadStore;
use bd3_thread_store::ReadThreadByRolloutPathParams as StoreReadThreadByRolloutPathParams;
use bd3_thread_store::ReadThreadParams as StoreReadThreadParams;
use bd3_thread_store::SearchThreadOccurrencesParams as StoreSearchThreadOccurrencesParams;
use bd3_thread_store::SearchThreadsParams as StoreSearchThreadsParams;
use bd3_thread_store::SortDirection as StoreSortDirection;
use bd3_thread_store::StoredThread;
use bd3_thread_store::StoredTurn;
use bd3_thread_store::StoredTurnItemsView;
use bd3_thread_store::StoredTurnStatus;
use bd3_thread_store::ThreadMetadataPatch as StoreThreadMetadataPatch;
use bd3_thread_store::ThreadRelationFilter as StoreThreadRelationFilter;
use bd3_thread_store::ThreadSortKey as StoreThreadSortKey;
use bd3_thread_store::ThreadStore;
use bd3_thread_store::ThreadStoreError;
use bd3_utils_absolute_path::AbsolutePathBuf;
use bd3_utils_pty::DEFAULT_OUTPUT_BYTES_CAP;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Error as IoError;
use std::path::Path;
use std::path::PathBuf;
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use tokio::sync::SemaphorePermit;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tokio_util::sync::DropGuard;
use tokio_util::task::TaskTracker;
use toml::Value as TomlValue;
use tracing::Instrument;
use tracing::error;
use tracing::info;
use tracing::warn;
use uuid::Uuid;

#[cfg(test)]
use bd3_app_server_protocol::ServerRequest;

mod account_processor;
mod apps_processor;
mod bedrock_auth;
mod catalog_processor;
mod command_exec_processor;
mod config_processor;
mod diagnostics;
mod environment_processor;
mod feedback_doctor_report;
mod feedback_processor;
mod fs_processor;
mod git_processor;
mod initialize_processor;
mod marketplace_processor;
mod mcp_processor;
mod persisted_resume_settings;
mod plugins;
mod process_exec_processor;
mod projects;
mod remote_control_processor;
mod search;
mod thread_enrichment;
mod thread_fork_goal;
mod thread_processor;
mod thread_queue_processor;
mod thread_sections;
mod token_usage_replay;
mod turn_processor;
mod windows_sandbox_processor;

pub(crate) use account_processor::AccountRequestProcessor;
pub(crate) use apps_processor::AppsRequestProcessor;
pub(crate) use catalog_processor::CatalogRequestProcessor;
pub(crate) use command_exec_processor::CommandExecRequestProcessor;
pub(crate) use config_processor::ConfigRequestProcessor;
pub(crate) use diagnostics::read_server_diagnostics;
pub(crate) use environment_processor::EnvironmentRequestProcessor;
pub(crate) use feedback_processor::FeedbackRequestProcessor;
pub(crate) use fs_processor::FsRequestProcessor;
pub(crate) use git_processor::GitRequestProcessor;
pub(crate) use initialize_processor::InitializeRequestProcessor;
pub(crate) use marketplace_processor::MarketplaceRequestProcessor;
pub(crate) use mcp_processor::McpRequestProcessor;
pub(crate) use plugins::PluginRequestProcessor;
pub(crate) use process_exec_processor::ProcessExecRequestProcessor;
pub(crate) use projects::ProjectRequestProcessor;
pub(crate) use remote_control_processor::RemoteControlRequestProcessor;
pub(crate) use search::SearchRequestProcessor;
pub(crate) use thread_goal_processor::ThreadGoalRequestProcessor;
pub(crate) use thread_processor::ThreadRequestProcessor;
pub(crate) use thread_queue_processor::ThreadQueueRequestProcessor;
pub(crate) use turn_processor::TurnRequestProcessor;
pub(crate) use windows_sandbox_processor::WindowsSandboxRequestProcessor;

use crate::error_code::internal_error;
use crate::error_code::invalid_request;
use crate::filters::compute_source_filters;
use crate::filters::source_kind_matches;
use crate::thread_state::ConnectionCapabilities;
use crate::thread_state::ThreadListenerCommand;
use crate::thread_state::ThreadState;
use crate::thread_state::ThreadStateManager;
use token_usage_replay::restored_token_usage_turn_id;
use token_usage_replay::send_thread_token_usage_update_to_connection;

fn resolve_request_cwd(cwd: Option<PathBuf>) -> Result<Option<AbsolutePathBuf>, JSONRPCErrorError> {
    cwd.map(|cwd| {
        AbsolutePathBuf::relative_to_current_dir(path_utils::normalize_for_native_workdir(cwd))
            .map_err(|err| invalid_request(format!("invalid cwd: {err}")))
    })
    .transpose()
}

fn resolve_turn_environment_selections(
    thread_manager: &ThreadManager,
    environments: Option<Vec<TurnEnvironmentParams>>,
) -> Result<Option<Vec<TurnEnvironmentSelection>>, JSONRPCErrorError> {
    let Some(environments) = environments else {
        return Ok(None);
    };
    let mut selections = Vec::with_capacity(environments.len());
    for environment in environments {
        let environment_id = environment.environment_id;
        let cwd = environment
            .cwd
            .to_inferred_path_uri()
            .ok_or_else(|| {
                invalid_request(format!(
                    "invalid cwd for environment `{environment_id}`: path `{}` does not use absolute POSIX or Windows path syntax",
                    environment.cwd
                ))
            })?;
        let workspace_roots = environment
            .runtime_workspace_roots
            .map(|roots| {
                let mut resolved_roots = Vec::new();
                for root in roots {
                    let root = root.to_inferred_path_uri().ok_or_else(|| {
                        invalid_request(format!(
                            "invalid runtime workspace root for environment `{environment_id}`: path `{root}` does not use absolute POSIX or Windows path syntax"
                        ))
                    })?;
                    if !resolved_roots.contains(&root) {
                        resolved_roots.push(root);
                    }
                }
                Ok::<_, JSONRPCErrorError>(resolved_roots)
            })
            .transpose()?
            .unwrap_or_else(|| vec![cwd.clone()]);
        selections.push(TurnEnvironmentSelection {
            environment_id,
            cwd,
            workspace_roots,
            config: EnvironmentConfigState::FromThread,
        });
    }
    thread_manager
        .validate_environment_selections(&selections)
        .map_err(environment_selection_error)?;
    Ok(Some(selections))
}

fn resolve_runtime_workspace_roots(workspace_roots: Vec<AbsolutePathBuf>) -> Vec<AbsolutePathBuf> {
    let mut resolved_roots = Vec::new();
    for root in workspace_roots {
        if !resolved_roots.iter().any(|existing| existing == &root) {
            resolved_roots.push(root);
        }
    }
    resolved_roots
}

mod config_errors;
mod request_errors;
mod thread_delete;
mod thread_goal_processor;
mod thread_lifecycle;
mod thread_resume_redaction;
mod thread_summary;

use self::config_errors::*;
use self::request_errors::*;
use self::thread_goal_processor::api_thread_goal_from_state;
use self::thread_lifecycle::*;
use self::thread_resume_redaction::*;
use self::thread_summary::*;

pub(crate) use self::thread_lifecycle::populate_thread_turns_from_history;
pub(crate) use self::thread_processor::thread_from_stored_thread;
#[cfg(test)]
pub(crate) use self::thread_summary::read_summary_from_rollout;
#[cfg(test)]
pub(crate) use self::thread_summary::summary_to_thread;
pub(crate) use self::thread_summary::thread_settings_from_config_snapshot;

pub(crate) fn build_legacy_api_turns_from_rollout_items(items: &[RolloutItem]) -> Vec<Turn> {
    let mut builder = ThreadHistoryBuilder::new();
    for item in items {
        if is_persisted_rollout_item(item, bd3_protocol::protocol::ThreadHistoryMode::Legacy) {
            builder.handle_rollout_item(item);
        }
    }
    builder.finish()
}
