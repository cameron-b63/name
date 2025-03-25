use serde_json::Value;

/// DAP Errors are sent as responses. This enum exists for convenience in structuring and describing different errors.
#[derive(PartialEq)]
pub enum DapError {
    AlreadyInitialized,
    AlreadyStartedDebugging,
    CapabilitiesNotLoaded,
    ImmortalChild,
    InsufficientArguments,
    LaunchFailed,
    NotImplemented(String),
    Unknown,
}

/// DAP Errors need to be formatted properly with JSON. This struct represents the required fields.
pub struct ErrorInfo {
    pub id: usize, 
    pub format: &'static str,
    pub send_telemetry: bool,
    pub error: DapError,
}

impl ErrorInfo {
    /// Serialization should not include the error field.
    pub fn to_value(&self) -> Value {
        serde_json::json!({
            "id": self.id,
            "format": self.format,
            "sendTelemetry": self.send_telemetry,
        })
    }
}

/// This table contains the error definitions for the DAP server.
// If you wish to add an error, look here.
pub const ERROR_DEFINITIONS: &[ErrorInfo] = &[
    ErrorInfo {
        id: 0,
        format: "Unknown error (error lookup failed to return a valid error)",
        send_telemetry: false,
        error: DapError::Unknown,
    },
    ErrorInfo {
        id: 1,
        format: "Already initialized",
        send_telemetry: false,
        error: DapError::AlreadyInitialized,
    },
    ErrorInfo {
        id: 2,
        format: "Command '{}' not implemented",
        send_telemetry: false,
        error: DapError::NotImplemented(String::new()),
    },
    ErrorInfo {
        id: 3,
        format: "Insufficient arguments",
        send_telemetry: false,
        error: DapError::InsufficientArguments,
    },
    ErrorInfo {
        id: 4,
        format: "Launch failed",
        send_telemetry: false,
        error: DapError::LaunchFailed,
    },
    ErrorInfo {
        id: 5,
        format: "Already started debugging",
        send_telemetry: false,
        error: DapError::AlreadyStartedDebugging,
    },
    ErrorInfo {
        id: 6,
        format: "Child process could not be killed",
        send_telemetry: false,
        error: DapError::ImmortalChild,
    },
    ErrorInfo {
        id: 7,
        format: "DAP Server capabilities could not be loaded. Internal error",
        send_telemetry: false,
        error: DapError::CapabilitiesNotLoaded,
    },
];
