// This file contains the code responsible for formatting responses to requests.
// It can format both Response and ErrorResponse type messages.

use serde_json::Value;

use crate::dap_structs::{DapError, DapRequest, DapResponse};

/// Create a new Response message to be sent in response to some initial request with the supplied body.
pub fn create_response(initial_request: &DapRequest, body: Value) -> DapResponse {
    DapResponse {
        seq: initial_request.seq+1,
        request_seq: initial_request.seq,
        success: true,
        body: Some(body),
    }
}

/// Create a new ErrorResponse message to be sent in response to some initial request of the supplied error type.
pub fn create_error_response(initial_request: &DapRequest, error: DapError) -> DapResponse {
    DapResponse {
        seq: initial_request.seq+1,
        request_seq: initial_request.seq,
        success: false,
        body: Some(create_error_body(error)),
    }
}

/// Create the body of an ErrorResponse. Matches on the DapError enum to determine proper structure/etc.
// If you wish to add an error, look here.
fn create_error_body(error: DapError) -> Value {
    match error {
        DapError::AlreadyInitialized => {
            serde_json::json!({
                "error": {
                    "id": 1,
                    "format": "Already initialized",
                    "sendTelemetry": false,
                }
            })
        },
        DapError::NotImplemented(command) => {
            serde_json::json!({
                "error": {
                    "id": 2,
                    "format": format!("Command '{}' not implemented", command),
                    "sendTelemetry": false,
                }
            })
        },
    }
}