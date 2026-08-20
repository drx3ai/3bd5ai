use super::*;
use bd3_protocol::error::Bd3ErrorDetails;

pub(super) fn environment_selection_error(err: Bd3Err) -> JSONRPCErrorError {
    match err.details() {
        Bd3ErrorDetails::InvalidRequest(message) => invalid_request(message.clone()),
        _ => internal_error(format!("failed to validate environment selections: {err}")),
    }
}
