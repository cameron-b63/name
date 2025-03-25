// This code is responsible for handling 'request' type DAP messages, issued through VSCode.
// It should call helpers which perform the specified translation.
// The DAP specification is here: https://microsoft.github.io/debug-adapter-protocol/specification

use crate::{dap_server::DapServer, dap_structs::{DapRequest, DapResponse}, response::create_error_response, tables::{error_definitions::DapError, handler::{Handler, HandlerFn, HANDLERS}}};

/// Handle a request issued by client.
/// Relies on a table of Handler types defined elsewhere.
pub fn handle_request(dap_server: &mut DapServer, request: DapRequest) -> Result<DapResponse, DapResponse> {
    // Find proper handler function in the lookup table
    let command: &str = &request.command.clone();
    let proper_function: HandlerFn = match look_up_handler(command) {
        // If an entry is found for given command, the lookup worked and this handler may execute.
        Some(handler) => handler.handler,
        // If no entry is found for given command, return a meaningful error
        None => return Err(create_error_response(&request, DapError::NotImplemented(String::from(command)))),
    };

    // Run found function to handle the request
    return proper_function(dap_server, request);
}

/// This function exists to abstract the logic of finding the proper client request handler.
/// Though this could be easily inlined, I chose to extract it for readability.
fn look_up_handler(command: &str) -> Option<&Handler> {
    return HANDLERS.iter().find(|handler| handler.command == command)
}
