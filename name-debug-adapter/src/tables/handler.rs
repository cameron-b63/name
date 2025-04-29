use crate::{dap_server::DapServer, dap_structs::{DapRequest, DapResponse}};
use crate::dap_handlers::*;

// This file contains the data structures associating each DAP handler with its handler. 

pub type HandlerFn = fn(&mut DapServer, DapRequest) -> Result<DapResponse, DapResponse>;

pub struct Handler {
    pub command: &'static str,
    pub handler: HandlerFn,
}

/// A working list of all the handlers that the DAP server supports.
pub const HANDLERS: &[Handler] = &[
    // Initialize
    Handler {command: "initialize", handler: handle_initialize},
    // Launch
    Handler {command: "launch", handler: handle_launch},
    // Attach (NOT supported right now. Only launch is supported)
    // Restart (does not have to support)
    // Disconnect
    Handler {command: "disconnect", handler: handle_disconnect},
    // Terminate (does not have to support)
    // BreakpointLocations (does not have to support)
    // SetBreakpoints 
    Handler {command: "setBreakpoints", handler: handle_set_breakpoints},
    // SetFunctionBreakpoints (does not have to support)
    // SetExceptionBreakpoints (does not have to support)
    // DataBreakpointInfo (does not have to support)
    // SetDataBreakpoints (does not have to support)
    // SetInstructionBreakpoints (does not have to support)
    // Next
    Handler {command: "next", handler: handle_next},
    // StepIn
    Handler {command: "stepIn", handler: handle_step_in},
    // StepOut
    Handler {command: "stepOut", handler: handle_step_out},
    // StepBack (does not have to support)
    // ReverseContinue (does not have to support)
    // RestartFrame (does not have to support)
    // Goto (does not have to support)
    // Pause
    Handler {command: "pause", handler: handle_pause},
    // StackTrace
    Handler {command: "stackTrace", handler: handle_stack_trace},
    // Scopes
    Handler {command: "scopes", handler: handle_scopes},
    // Variables
    Handler {command: "variables", handler: handle_variables},
    // SetVariable (does not have to support)
    // Source
    Handler {command: "source", handler: handle_source},
    // Threads
    Handler {command: "threads", handler: handle_threads},
    // TerminateThreads (does not have to support)
    // Modules (does not have to support)
    // LoadedSources (does not have to support)
    // Evaluate
    Handler {command: "evaluate", handler: handle_evaluate},
    // SetExpression (does not have to support)
    // StepInTargets (does not have to support)
    // GotoTargets (does not have to support)
    // Completions (does not have to support)
    // ExceptionInfo (does not have to support)
    // ReadMemory (does not have to support)
    // WriteMemory (does not have to support)
    // Disassemble (does not have to support)
    // Locations
    Handler {command: "locations", handler: handle_locations},
];