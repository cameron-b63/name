use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExceptionBreakpointFilter {
    pub filter: String,
    pub label: String,
    pub default: bool,
}

/// Structure is defined to ensure proper serialization.
#[derive(Debug, Serialize, Deserialize)]
pub struct Capabilities {
    pub supports_configuration_done_request: bool,
    pub supports_function_breakpoints: bool,
    pub supports_conditional_breakpoints: bool,
    pub supports_hit_conditional_breakpoints: bool,
    pub supports_evaluate_for_hovers: bool,
    pub exception_breakpoint_filters: Vec<ExceptionBreakpointFilter>,
    pub supports_step_back: bool,
    pub supports_set_variable: bool,
    pub supports_restart_frame: bool,
    pub supports_goto_targets_request: bool,
    pub supports_step_in_targets_request: bool,
    pub supports_completions_request: bool,
    pub supports_modules_request: bool,
    pub additional_module_columns: Vec<Value>,
    pub supported_checksum_algorithms: Vec<Value>,
}

pub static CAPABILITIES: OnceLock<Capabilities> = OnceLock::new();

/// This JSON data represents the capabilities of the DAP server. It is sent to the client on an initialize request.
pub fn get_capabilities() -> &'static Capabilities {
    CAPABILITIES.get_or_init(|| Capabilities {
        supports_configuration_done_request: false,
        supports_function_breakpoints: false,
        supports_conditional_breakpoints: false,
        supports_hit_conditional_breakpoints: false,
        supports_evaluate_for_hovers: false,
        exception_breakpoint_filters: vec![ExceptionBreakpointFilter {
            filter: "filterID".to_string(),
            label: "label".to_string(),
            default: false,
        }],
        supports_step_back: false,
        supports_set_variable: false,
        supports_restart_frame: false,
        supports_goto_targets_request: false,
        supports_step_in_targets_request: false,
        supports_completions_request: false,
        supports_modules_request: false,
        additional_module_columns: vec![],
        supported_checksum_algorithms: vec![],
    })
}
