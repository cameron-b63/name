import * as vscode from 'vscode';
import { runAssembler, runLinker, runWithoutDebugging } from './simple_commands';

const path = require('path');	// For OS-agnostic paths

let chained = false;	// For UX (if commands are chained, they won't switch to their output channels automatically to avoid confusing output)

// Register commands

export function registerCommands(context: vscode.ExtensionContext) {
    // Get bin directory for name binaries
    const name_bin_directory = path.join(context.extensionPath, 'bin');

	// Driver code for spawning assembler process
	vscode.commands.registerCommand('name-ext.assemblecurrentfile', () => {
		// Get file name
		let currently_open_file: string | undefined = vscode.window.activeTextEditor?.document.fileName;
		if (currently_open_file === undefined) {
			vscode.window.showErrorMessage('No file open.');
			return;
		}

		// Create output file name
		let output_file = currently_open_file ? path.format({ ...path.parse(currently_open_file), base: undefined, ext: '.o' }) : undefined;

		// Call runner
		runAssembler(name_bin_directory, currently_open_file, output_file, chained).then((success_message) => {
			vscode.window.showInformationMessage(success_message);
		}).catch((error_message) => {
			vscode.window.showErrorMessage(error_message);
		});

	});

	// Driver code for spawning linker process
	vscode.commands.registerCommand('name-ext.linkcurrentfile', async () => {
		// Get file name
		let currently_open_file: string | undefined = vscode.window.activeTextEditor?.document.fileName;
		if (currently_open_file === undefined) {
			vscode.window.showErrorMessage('No file open.');
			return;
		}

		// Get directory of the currently open file
		let current_dir = path.dirname(currently_open_file);

		// Get all .asm files in the same directory
		let infiles: string[] = [];
		const files = await vscode.workspace.fs.readDirectory(vscode.Uri.file(current_dir));
		for (const [file, fileType] of files) {
			if (fileType === vscode.FileType.File && file.endsWith('.o')) {
				infiles.push(path.join(current_dir, file));
			}
		}

		// Create output file name
		let output_file = currently_open_file ? path.format({ ...path.parse(currently_open_file), base: undefined, ext: '' }) : undefined;

		// Call runner
		runLinker(name_bin_directory, infiles, output_file, chained).then((success_message) => {
			vscode.window.showInformationMessage(success_message);
		}).catch((error_message) => {
			vscode.window.showErrorMessage(error_message);
		});

	});

	// Driver code for spawning emulator with no debugging
	vscode.commands.registerCommand('name-ext.runnodebug', () => {
		// Get file name
		let currently_open_file: string | undefined = vscode.window.activeTextEditor?.document.fileName;
		if (currently_open_file === undefined) {
			vscode.window.showErrorMessage('No file open.');
			return;
		}

		let file_to_run: string = path.format({ ...path.parse(currently_open_file), base: undefined, ext: '' });

		// Run the file
		runWithoutDebugging(name_bin_directory, file_to_run).then((success_message) => {
			vscode.window.showInformationMessage(success_message);
		}).catch((error_message) => {
			vscode.window.showErrorMessage(error_message);
		});
	});

	// Assemble, link, and run the current file all in one go
	vscode.commands.registerCommand('name-ext.assemblerunnodebug', () => {
		// Indicate to commands that they are chained and do not need to show their output channels
		chained = true;

		// Assemble, link, execute
		vscode.commands.executeCommand('name-ext.assemblecurrentfile')
			.then(() => vscode.commands.executeCommand('name-ext.linkcurrentfile'))
			.then(() => vscode.commands.executeCommand('name-ext.runnodebug'));

		// Reset chained flag so as not to affect future commands
		chained = false;
	});
}