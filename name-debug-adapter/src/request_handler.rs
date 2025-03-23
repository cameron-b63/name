// This code is responsible for handling 'request' type DAP messages, issued through VSCode.
// It should call helpers which perform the specified translation.
// The DAP specification is here: https://microsoft.github.io/debug-adapter-protocol/specification

use crate::{dap_server::DapServer, dap_structs::{DapError, DapRequest, DapResponse}, handler::{HandlerFn, HANDLERS}, response::create_error_response};

/// Handle a request issued by client.
pub fn handle_request(dap_server: &mut DapServer, request: DapRequest) -> Result<DapResponse, DapResponse> {
    // Find proper handler function in the lookup table
    let command: &str = &request.command.clone();   // I don't like partial borrowing
    let proper_function: HandlerFn = match HANDLERS.iter().find(|handler| handler.command == command) {
        Some(handler) => handler.handler,
        // If no entry is found for given command, return a meaningful error
        None => return Err(create_error_response(&request, DapError::NotImplemented(String::from(command)))),
    };

    // Run found function to handle the request
    return proper_function(dap_server, request);
}