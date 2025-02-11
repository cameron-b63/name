// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import { runAssembler, runLinker, runWithoutDebugging } from './simple_commands';
import { NAMETreeDataProvider } from './tree';

const path = require('path');

let chained = false;

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	const name_bin_directory = path.join(context.extensionPath, 'bin');
	console.log(name_bin_directory);

	const treeDataProvider = new NAMETreeDataProvider();
	vscode.window.createTreeView('name-ext.tree', { treeDataProvider });

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

	// Assemble, link, and run the current file all in one go.
	vscode.commands.registerCommand('name-ext.assemblerunnodebug', () => {
		// Indicate to commands that they are chained and do not need to show their output channels
		chained = true;

		vscode.commands.executeCommand('name-ext.assemblecurrentfile')
			.then(() => vscode.commands.executeCommand('name-ext.linkcurrentfile'))
			.then(() => vscode.commands.executeCommand('name-ext.runnodebug'));

		// Reset chained flag so as not to affect future commands
		chained = false;
	});

}

// This method is called when your extension is deactivated
export function deactivate() {}
