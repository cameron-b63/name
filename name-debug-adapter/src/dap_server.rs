use std::io::{self, BufRead, Read};

// This code is responsible for managing the DAP server struct. 

/// The DapServer struct contains the necessary information to manage the DAP server.
pub struct DapServer {
    // I don't yet know what goes here
    is_terminated: bool,
}

impl DapServer {
    /// Create a new DapServer struct to keep all server information in one place
    fn new() -> DapServer {
        return DapServer {is_terminated: false};
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
        // Do-nothing for now
        // TODO: Read a single DAP message from stdin
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
    pub fn handle_message(&self, _message: String) -> Result<String, String> {
        // Do-nothing for now
        // TODO: Handle the message with appropriate thingy
        return Err(String::from("Not yet implemented."));
    }

    /// Send a response through the appropriate output channel
    pub fn send_response(&self, _response: String) -> Result<(), String> {
        // Do-nothing for now
        // TODO: Implement as specified
        return Err(String::from("Not yet implemented."));
    }

    /// Send an error through the appropriate output channel
    pub fn send_error(&self, _error: String) -> Result<(), String> {
        // Do-nothing for now
        // TODO: Implement as specified
        return Err(String::from("Not yet implemented."));
    }

    pub fn is_terminated(&self) -> bool {
        return self.is_terminated;
    }
}