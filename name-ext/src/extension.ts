// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import { runAssembler, runLinker, runWithoutDebugging } from './simple_commands';

const path = require('path');

const placeholder_for_config = '/home/cameron/Projects/name';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {

	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log('Congratulations, your extension "name-ext" is now active!');

	// The command has been defined in the package.json file
	// Now provide the implementation of the command with registerCommand
	// The commandId parameter must match the command field in package.json
	const disposable = vscode.commands.registerCommand('name-ext.helloWorld', () => {
		// The code you place here will be executed every time your command is executed
		// Display a message box to the user
		vscode.window.showInformationMessage('Hello World from name-ext! We made it mom!');
	});

	context.subscriptions.push(disposable);

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
		runAssembler(placeholder_for_config, currently_open_file, output_file).then((success_message) => {
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
		runLinker(placeholder_for_config, infiles, output_file).then((success_message) => {
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
		runWithoutDebugging(placeholder_for_config, file_to_run).then((success_message) => {
			vscode.window.showInformationMessage(success_message);
		}).catch((error_message) => {
			vscode.window.showErrorMessage(error_message);
		});
	});

}

// This method is called when your extension is deactivated
export function deactivate() {}