// This file contains the implementations for each DAP handler, referenced in handler.rs.
// Each of them will have the same function signature, outlined in handler.rs.

use serde_json::{from_value, Value};

use crate::{dap_server::DapServer, dap_structs::{DapRequest, DapResponse}, response::{create_error_response, create_response}, tables::error_definitions::DapError};

/// This function handles the "initialize" request issued by the client. 
/// It is the first request sent by the client to the debug adapter, and returns the capabilities of the debug adapter.
/// The capabilities of the debug adapter are defined as a constant.
pub fn handle_initialize(dap_server: &mut DapServer, request: DapRequest) -> Result<DapResponse, DapResponse> {
    // Initialize the debug adapter (send configuration information).
    // This must only be done once.
    if dap_server.is_initialized() {
        return Err(create_error_response(&request, DapError::AlreadyInitialized));
    }

    // Set initialized to true and get configuration information
    let configuration: Value = match dap_server.initialize() {
        Ok(v) => v,
        Err(e) => return Err(create_error_response(&request, e)),
    };

    // Return configuration information
    return Ok(create_response(&request, configuration));
}


/// This function handles the "launch" request issued by the client.
/// It launches the debuggee process and attaches the debugger to it.
pub fn handle_launch(dap_server: &mut DapServer, request: DapRequest) -> Result<DapResponse, DapResponse> {
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

/// This function handles the "disconnect" request issued by the client.
/// It kills the child process and disconnnects the debugger.
pub fn handle_disconnect(dap_server: &mut DapServer, request: DapRequest) -> Result<DapResponse, DapResponse> {
    // Disconnect the debugger
    match dap_server.disconnect() {
        Ok(_) => return Ok(create_response(&request, Value::Null)),
        Err(e) => return Err(create_error_response(&request, e))
    }
}

/// This function handles the "setBreakpoints" request issued by the client.
/// It sets breakpoints in the debuggee process.
pub fn handle_set_breakpoints(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle setBreakpoints request");
    // Parse body of request to determine lines to break on
    // Deliver `b` requests
    // Each request may fail - create response accordingly
    // Return response
}

/// This function handles the "next" request issued by the client.
/// It steps over the next line of code.
pub fn handle_next(dap_server: &mut DapServer, request: DapRequest) -> Result<DapResponse, DapResponse> {
    // Stops at next line of source - synonymous with step_in for assembly code.
    return handle_step_in(dap_server, request);
}

/// This function handles the "stepIn" request issued by the client.
/// Executes a single line of assembly code.
pub fn handle_step_in(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle stepIn request");
    // Executes a single line of assembly code. Should literally just need to send `s` to the client.
    // Parsing response might need to be abstracted to function. This is the point where each parsing thing gets specific.
    // Package response/err as JSON
}

/// This function handles the "stepOut" request issued by the client.
/// It steps out of the current function. Little nebulous - should reasonably skip to the instruction after the next jump to $ra.
/// The actual functionality should be defined inside the name-emu debugger.
pub fn handle_step_out(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle stepOut request");
    // Send `so` to child process
    // Parse response
    // Package response/err as JSON
}

/// This function handles the "pause" request issued by the client.
/// It pauses the debuggee process.
pub fn handle_pause(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle pause request");
    // Pause the child process. Effectively should just do nothing and continue blocking for next request.
    // Still send acknowledgement.
}

/// This function handles the "stackTrace" request issued by the client.
/// It returns the stack trace of the debuggee process.
/// The debugger needs to implement a stack trace request for consecutive function calls.
pub fn handle_stack_trace(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle stackTrace request");
}

/// This function handles the "scopes" request issued by the client.
/// It returns the scopes of the current stack frame.
/// This may be meaningless for our purposes. Thought required.
pub fn handle_scopes(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle scopes request");
}

/// This function handles the "variables" request issued by the client.
/// It returns the variables of a given scope.
pub fn handle_variables(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle variables request");
}

/// This function handles the "source" request issued by the client.
/// It returns the source code of a given source reference.
pub fn handle_source(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle source request");
}

/// This function handles the "threads" request issued by the client.
/// It returns the threads of the debuggee process.
pub fn handle_threads(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle threads request");
}

/// This function handles the "evaluate" request issued by the client.
/// It evaluates an expression in the context of the current stack frame.
pub fn handle_evaluate(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle evaluate request");
}

/// This function handles the "locations" request issued by the client.
/// It returns the locations of a breakpoint.
pub fn handle_locations(_dap_server: &mut DapServer, _request: DapRequest) -> Result<DapResponse, DapResponse> {
    todo!("Handle locations request");
}