// This code is responsible for handling DAP requests issued through VSCode.
// It should call helpers which perform the specified translation.
// The DAP specification is here: https://microsoft.github.io/debug-adapter-protocol/specification

// DAP requests:

// Initialize
// ConfigurationDone (does not have to support)
// Launch 
// Attach
// Restart (does not have to support)
// Disconnect
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