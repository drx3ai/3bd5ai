use bd3_api::ImageEditRequest;
use bd3_api::ImageGenerationRequest;
use bd3_api::ImageResponse;
use bd3_api::ImagesClient;
use bd3_api::ReqwestTransport;
use bd3_api::map_api_error;
use bd3_login::default_client::add_originator_header;
use bd3_login::default_client::create_client;
use bd3_model_provider::SharedModelProvider;
use bd3_protocol::error::Bd3Err;
use http::HeaderMap;
use http::HeaderValue;

const X_BD3_IMAGE_TURN_ID_HEADER: &str = "x-codex-image-turn-id";

pub(crate) struct ImageBackendError {
    message: String,
    bd3_error: Bd3Err,
}

impl ImageBackendError {
    fn from_api(error: bd3_api::ApiError) -> Self {
        let message = error.to_string();
        Self {
            message,
            bd3_error: map_api_error(error),
        }
    }

    fn from_message(message: String) -> Self {
        Self {
            bd3_error: Bd3Err::Stream(message.clone()),
            message,
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn bd3_error(&self) -> &Bd3Err {
        &self.bd3_error
    }
}

#[derive(Clone)]
pub(crate) struct Bd3ImagesBackend {
    provider: SharedModelProvider,
    originator: Option<String>,
}

impl Bd3ImagesBackend {
    /// Creates a backend that sends image requests through the active model provider.
    pub(crate) fn new(provider: SharedModelProvider, originator: Option<String>) -> Self {
        Self {
            provider,
            originator,
        }
    }

    /// Resolves the provider and auth required for the current image API request.
    async fn client(&self) -> Result<ImagesClient<ReqwestTransport>, ImageBackendError> {
        let provider = self
            .provider
            .api_provider()
            .await
            .map_err(|err| ImageBackendError::from_message(err.to_string()))?;
        let auth = self
            .provider
            .api_auth()
            .await
            .map_err(|err| ImageBackendError::from_message(err.to_string()))?;
        Ok(ImagesClient::new(
            ReqwestTransport::from_http_client(create_client()),
            provider,
            auth,
        ))
    }

    /// Sends a standalone image generation request through the configured Images client.
    pub(crate) async fn generate(
        &self,
        request: ImageGenerationRequest,
        turn_id: &str,
    ) -> Result<ImageResponse, ImageBackendError> {
        self.client()
            .await?
            .generate(
                &request,
                image_request_headers(self.originator.as_deref(), turn_id),
            )
            .await
            .map_err(ImageBackendError::from_api)
    }

    /// Sends a standalone image edit request through the configured Images client.
    pub(crate) async fn edit(
        &self,
        request: ImageEditRequest,
        turn_id: &str,
    ) -> Result<ImageResponse, ImageBackendError> {
        self.client()
            .await?
            .edit(
                &request,
                image_request_headers(self.originator.as_deref(), turn_id),
            )
            .await
            .map_err(ImageBackendError::from_api)
    }
}

fn image_request_headers(originator: Option<&str>, turn_id: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(turn_id) = HeaderValue::from_str(turn_id) {
        headers.insert(X_BD3_IMAGE_TURN_ID_HEADER, turn_id);
    }
    if let Some(originator) = originator {
        add_originator_header(&mut headers, originator);
    }
    headers
}
