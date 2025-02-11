import * as vscode from 'vscode';
import { registerAssemble } from './assemble';
import { registerLink } from './link';
import { registerRunNoDebug } from './emulate';
import { registerAssembleRunNoDebug } from './assemble_run_no_debug';

const path = require('path');	// For OS-agnostic paths

// Register commands

export function registerCommands(context: vscode.ExtensionContext) {
    // Get bin directory for name binaries
    const name_bin_directory = path.join(context.extensionPath, 'bin');

	// Register assembler
	registerAssemble(context, name_bin_directory);

	// Register linker
	registerLink(context, name_bin_directory);

	// Register emulator
	registerRunNoDebug(context, name_bin_directory);

	// Register command for assembly, linking, and execution without debugging
	registerAssembleRunNoDebug(context, name_bin_directory);
}