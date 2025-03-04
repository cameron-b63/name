// This code is responsible for handling 'request' type DAP messages, issued through VSCode.
// It should call helpers which perform the specified translation.
// The DAP specification is here: https://microsoft.github.io/debug-adapter-protocol/specification

use serde_json::{from_value, Value};

use crate::{dap_server::DapServer, dap_structs::{DapError, DapRequest, DapResponse}, response::{create_error_response, create_response}};


pub fn handle_request(dap_server: &mut DapServer, request: DapRequest) -> Result<DapResponse, DapResponse> {
    // Match on request.command to take appropriate action
    let command: &str = &request.command;
    match command {
        // DAP requests:

        // Initialize
        "initialize" => {
            // Initialize the debug adapter (send configuration information).
            // This must only be done once.
            if dap_server.is_initialized() {
                return Err(create_error_response(&request, DapError::AlreadyInitialized));
            }

            // Set initialized to true and get configuration information
            let configuration: Value = dap_server.initialize();

            // Return configuration information
            return Ok(create_response(&request, configuration));
        }
        // ConfigurationDone (does not have to support)
        // Launch 
        "launch" => {
            // Cannot launch more than once 
            if dap_server.has_child() {
                return Err(create_error_response(&request, DapError::AlreadyStartedDebugging));
            }
            // Retrieve arguments
            let arguments = match request.arguments.clone() {
                Some(args) => args,
                None => return Err(create_error_response(&request, DapError::InsufficientArguments)),
            };

            let structured_arguments = match from_value(arguments) {
                Ok(args) => args,
                Err(_) => return Err(create_error_response(&request, DapError::InsufficientArguments)),
            };

            // Launch subprocess
            let launch_response: DapResponse = match dap_server.launch(structured_arguments) {
                Ok(res) => create_response(&request, res),
                Err(e) => create_error_response(&request, e),
            };

            return Ok(launch_response);
        }
        // Attach (NOT supported right now. Only launch is supported)
        // Restart (does not have to support)
        // Disconnect
        "disconnect" => {
            // Disconnect the debugger
            dap_server.disconnect();
            return Ok(create_response(&request, Value::Null));
        }
        // Terminate (does not have to support)
        // BreakpointLocations (does not have to support)
        // SetBreakpoints 
        // SetFunctionBreakpoints (does not have to support)
        // SetExceptionBreakpoints (does not have to support)
        // DataBreakpointInfo (does not have to support)
        // SetDataBreakpoints (does not have to support)
        // SetInstructionBreakpoints (does not have to support)
        // Next
        // StepIn
        // StepOut
        // StepBack (does not have to support)
        // ReverseContinue (does not have to support)
        // RestartFrame (does not have to support)
        // Goto (does not have to support)
        // Pause
        // StackTrace
        // Scopes
        // Variables
        // SetVariable (does not have to support)
        // Source
        // Threads
        // TerminateThreads (does not have to support)
        // Modules (does not have to support)
        // LoadedSources (does not have to support)
        // Evaluate
        // SetExpression (does not have to support)
        // StepInTargets (does not have to support)
        // GotoTargets (does not have to support)
        // Completions (does not have to support)
        // ExceptionInfo (does not have to support)
        // ReadMemory (does not have to support)
        // WriteMemory (does not have to support)
        // Disassemble (does not have to support)
        // Locations
        _ => {
            // Command could not be recognized or is not implemented
            return Err(create_error_response(&request, DapError::NotImplemented(request.command.clone())));
        }
    }
}