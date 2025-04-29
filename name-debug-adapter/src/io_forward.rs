// This code is responsible for handling the I/O splitting.
// Normal input/output should be sent to the user's terminal.
// Debug info should be sent to the DAP.
// This is going to be super easy, as the plan is to add a new flag to emulator which sends debug print to stderr
// and user prints issued by syscalls to stdout.
// This code will still make the split more graceful.