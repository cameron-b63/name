use name_debug_adapter::dap_server::{start_dap_server, DapServer};

// This module exists to provide a translation layer between VSCode's DAP and NAME's CLI debugger.
// VSCode communicates through JSON. NAME expects some standard CLI commands to be issued through stdin.
// It is the burden of the debug adapter to translate requests issued with DAP to commands NAME can interpret.
fn main() {
    // Parse any cli arguments (not yet specified but leaving room)

    // Initialize any server stuff
    let dap_server: DapServer = start_dap_server();

    // Setup async I/O (send normal output through user terminal, debug info to DAP)


    loop {
        // handle a DAP message
        match dap_server.read_message() {
            Some(message) => {
                // Call the message handler (minimizing main logic because I don't want to read all of that)
                match dap_server.handle_message(message) {
                    // If Ok, send response back to client.
                    Ok(response) => match dap_server.send_response(response) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Error occurred when sending response: {}", e),
                    }
                    
                    // If Err, properly format err and send it back to client.
                    Err(e) => match dap_server.send_error(e) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Error occurred when sending error (ironically): {}", e),
                    }
                }
            },
            None => break,  // Unrecoverable error encountered
        }

        if dap_server.is_terminated() {
            break;
        }
    }  

    // shutdown gracefully
}