use serde::{self, Deserialize, Serialize};
use serde_json::Value;

// This code describes JSON schema for DAP messages.
// It uses the serde crate to make working with this JSON easier.

/// DAP messages can be requests, responses, or events.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum DapMessage {
    Request(DapRequest),
    Response(DapResponse),
    Event(DapEvent),
}

/// DAP requests are sent from the client to the server.
#[derive(Serialize, Deserialize, Debug)]
pub struct DapRequest {
    pub seq: usize,
    pub command: String,
    pub arguments: Option<Value>,
}

/// DAP responses are sent from the server to the client. They are responses to requests.
#[derive(Serialize, Deserialize, Debug)]
pub struct DapResponse {
    pub seq: usize, 
    pub request_seq: usize,
    pub success: bool,
    pub body: Option<Value>,
}

/// DAP events are sent from the server to the client. They describe actions which may not have been initiated by a request.
#[derive(Serialize, Deserialize, Debug)]
pub struct DapEvent {
    pub seq: usize,
    pub event: String,
    pub body: Option<Value>,
}

/// Launch arguments to launch the debugging process are formatted like so:
#[derive(Serialize, Deserialize, Debug)]
pub struct LaunchArguments {
    pub name_emu_path: String,
    pub exe_name: String,
}

