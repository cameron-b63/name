use std::io::{self, BufRead, Read};

use serde_json::Value;

use crate::{dap_structs::{DapMessage, DapResponse}, request_handler::handle_request};

// This code is responsible for managing the DAP server struct. 

/// The DapServer struct contains the necessary information to manage the DAP server.
pub struct DapServer {
    // I don't yet know what goes here
    is_terminated: bool,
    is_initialized: bool,
}

impl DapServer {
    /// Create a new DapServer struct to keep all server information in one place
    fn new() -> DapServer {
        return DapServer {is_terminated: false, is_initialized: false};
    }
}

/// Wrapper over DapServer::new() - for managing API
pub fn start_dap_server() -> DapServer {
    // Instantiate a DapServer struct
    return DapServer::new();
}

impl DapServer {

    /// Read a message from stdin
    pub fn read_message(&self) -> Option<String> {
        // Setup I/O
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut content_length = 0;
        let mut line = String::new();

        // Find next Content-Length header
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error occurred when attempting to read from stdin: {e}");
                    return None;
                },
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                // Blank line between header and payload reached:
                break;
            }

            if trimmed.starts_with("Content-Length:") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() == 2 {
                    // Split on whitespace to get Content-Length
                    // Parse Content-Length as usize
                    content_length = parts[1].parse::<usize>().unwrap_or(0);
                }
            }
        }

        // Handle invalid Content-Length
        if content_length == 0 {
            return None;
        }

        // Read the JSON payload
        let mut buffer: Vec<u8> = vec![0; content_length];
        match reader.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error occurred while reading JSON payload: {e}");
                return None;
            },
        };

        // Return the JSON payload as a String
        return match String::from_utf8(buffer) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("Error occurred while converting JSON payload to String: {e}");
                None
            }
        };
    }

    /// Parse a read message and call appropriate handler
    pub fn handle_message(&mut self, message: String) -> Result<String, String> {
        match serde_json::from_str::<DapMessage>(&message) {
            Ok(DapMessage::Event(event)) => {
                // Events should never be sent from client to server.
                eprintln!("Error: Event received from client.");
                return Err(format!("Event received from client.\n{event:?}"));
            }
            Ok(DapMessage::Request(req)) => {
                // Invoke request handler
                let response: DapResponse = match handle_request(self, req) {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("Error occurred while handling request: {e:?}");
                        return Err(String::from("Error occurred while handling request."));
                    }
                };

                // Serialize response and return as string
                return match serde_json::to_string(&response) {
                    Ok(s) => Ok(s),
                    Err(e) => {
                        eprintln!("Error occurred while serializing response: {e}");
                        return Err(String::from("Error occurred while serializing response."));
                    }
                };
            },
            Ok(DapMessage::Response(res)) => {
                // Responses should never be sent from client to server.
                eprintln!("Error: Response received from client.");
                return Err(format!("Response received from client.\n{res:?}"));
            },
            Err(e) => {
                eprintln!("Error occurred while parsing JSON: {e}");
                return Err(String::from("Error occurred while parsing JSON."));
            }
        }
    }

    /// Send a response through the appropriate output channel
    /// Expects properly formatted JSON.
    pub fn send_response(&self, response: String) {
        let formatted: String = append_content_length_header(response);
        println!("{formatted}");
    }

    /// Return a boolean representing whether the emulator has terminated.
    pub fn is_terminated(&self) -> bool {
        return self.is_terminated;
    }

    /// Return a boolean representing whether an initialize request has already been handled.
    pub fn is_initialized(&self) -> bool {
        return self.is_initialized;
    }

    /// Edit the DapServer configuration here.
    pub fn initialize(&mut self) -> Value {
        return serde_json::json!({
            "supportsConfigurationDoneRequest": false,
            "supportsFunctionBreakpoints": false,
            "supportsConditionalBreakpoints": false,
            "supportsHitConditionalBreakpoints": false,
            "supportsEvaluateForHovers": false,
            "exceptionBreakpointFilters": [
                {
                    "filter": "filterID",
                    "label": "label",
                    "default": false
                }
            ],
            "supportsStepBack": false,
            "supportsSetVariable": false,
            "supportsRestartFrame": false,
            "supportsGotoTargetsRequest": false,
            "supportsStepInTargetsRequest": false,
            "supportsCompletionsRequest": false,
            "supportsModulesRequest": false,
            "additionalModuleColumns": [],
            "supportedChecksumAlgorithms": []
        });
    }
}

/// Format a response/error to be sent back to the client
fn append_content_length_header(content: String) -> String {
    let length = content.len();
    return format!("Content-Length: {length}\r\n\r\n{content}");
}