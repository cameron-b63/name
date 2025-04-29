use std::{
    io::{self, BufRead, Read},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::Value;

use crate::{
    dap_structs::{DapMessage, DapResponse, LaunchArguments},
    request_handler::handle_request,
    tables::{capabilities::get_capabilities, error_definitions::DapError},
};
use std::io::Write;

// This code is responsible for managing the DAP server struct.

/// The DapServer struct contains the necessary information to manage the DAP server.
pub struct DapServer {
    debugger_process: Option<Subprocess>,
    is_terminated: bool,
    is_initialized: bool,
}

/// Private struct encapsulating all the child process functionality
struct Subprocess {
    process: Child,
    stdin: ChildStdin,
    _stdout: ChildStdout,
    _stderr: ChildStderr,
}

impl DapServer {
    /// Create a new DapServer struct to keep all server information in one place
    fn new() -> DapServer {
        return DapServer {
            debugger_process: None,
            is_terminated: false,
            is_initialized: false,
        };
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
                }
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
            }
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
            }
            Ok(DapMessage::Response(res)) => {
                // Responses should never be sent from client to server.
                eprintln!("Error: Response received from client.");
                return Err(format!("Response received from client.\n{res:?}"));
            }
            Err(e) => {
                eprintln!("Error occurred while parsing JSON: {e}");
                return Err(String::from("Error occurred while parsing JSON."));
            }
        }
    }

    /// Send a response through the appropriate output channel
    /// Expects properly formatted JSON.
    pub fn send_response(&self, response: String) {
        let formatted: String = {
            let content = response;
            let length = content.len();
            format!("Content-Length: {length}\r\n\r\n{content}")
        };

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

    pub fn has_child(&self) -> bool {
        return self.debugger_process.is_some();
    }

    /// Edit the DapServer configuration here.
    pub fn initialize(&mut self) -> Result<Value, DapError> {
        return match serde_json::to_value(get_capabilities()) {
            Ok(v) => {
                self.is_initialized = true;
                Ok(v)
            }
            Err(e) => {
                eprintln!("Error occurred while serializing capabilities: {e}");
                Err(DapError::CapabilitiesNotLoaded)
            }
        };
    }

    /// Launch the debugging subprocess using the supplied arguments.
    pub fn launch(&mut self, arguments: LaunchArguments) -> Result<Value, DapError> {
        let mut child = match Command::new(arguments.name_emu_path)
            .args(&[arguments.exe_name, String::from("--debug")])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error occurred while launching subprocess: {e}");
                return Err(DapError::LaunchFailed);
            }
        };

        // Get the stdin, stdout, and stderr of the child process
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Store the child process and its I/O in a Subprocess struct
        let subprocess = Subprocess {
            process: child,
            stdin,
            _stdout: stdout,
            _stderr: stderr,
        };

        // Store the Subprocess in the DapServer struct
        self.debugger_process = Some(subprocess);

        return Ok(serde_json::json!({"message": "Subprocess launched successfully"}));
    }

    /// Kill the child process and prepare to kill server.
    pub fn disconnect(&mut self) -> Result<(), DapError> {
        // Send the debugger process a graceful kill message
        if let Some(subprocess) = &mut self.debugger_process {
            if let Err(e) = subprocess.stdin.write_all(b"q\n") {
                eprintln!("Error occurred while writing to subprocess stdin: {e}");
            }
        }

        // Terminate the debugger process
        if let Some(mut subprocess) = self.debugger_process.take() {
            match subprocess.process.kill() {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error occurred while killing subprocess: {e}");
                    return Err(DapError::ImmortalChild);
                }
            };
        }

        // Set the is_terminated flag to true
        self.is_terminated = true;
        Ok(())
    }
}
