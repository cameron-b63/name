// This module exists to provide a translation layer between VSCode's DAP and NAME's CLI debugger.
// VSCode communicates through JSON. NAME expects some standard CLI commands to be issued through stdin.
// It is the burden of the debug adapter to translate requests issued with DAP to commands NAME can interpret.
fn main() {
    // Parse any cli arguments (not yet specified but leaving room)

    // Start a name-emu subprocess

    // Setup async I/O (send normal output through user terminal, debug info to DAP)

    // loop
        // handle a DAP message
        // if the message was a disconnect request, break    

    // shutdown gracefully
}