// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
'use strict';
import * as vscode from 'vscode';
import * as Net from 'net';
import { activateNameDebug } from './activateNameDebug';
import * as path from 'path';
// const { spawn } = require('child_process');

const termName = "NAME Emulator";

const runMode: 'external' | 'server' | 'namedPipeServer' | 'inline' = 'server';

// TODO: Allow this code to run on Windows, Linux, and macOS.
// The current issue is that the paths are made with linux in mind.
// There exist libraries which would resolve this. There are also known techniques specific to vscode. 
// Should not take much looking.
export function activate(context: vscode.ExtensionContext) {
	context.subscriptions.push(
		vscode.commands.registerCommand("extension.vsname.startEmu", () => {
			// User configuration
			var configuration = vscode.workspace.getConfiguration('name-ext');
			if (!configuration) {
				vscode.window.showErrorMessage("Failed to find NAME configurations");
				return;
			}

			// For the record, this one line is resulting in ENOENT in 'run npm' (as far as I can tell).
			const namePath = configuration.get('namePath', '');
			if (namePath.length < 1) {
				vscode.window.showErrorMessage(`Failed to find a path for NAME, please set the path in VSCode's User Settings under name-ext`);
				return;
			}

			const nameASPath = path.join(namePath, 'name-as');
			const nameDefaultCfgPath = path.join(nameASPath, 'configs/default.toml');
			const nameEMUPath = path.join(namePath, 'name-emu');
			const nameEXTPath = path.join(namePath, 'name-ext');
			console.log(nameEXTPath);

			var editor = vscode.window.activeTextEditor;			
			if (editor) {
				// Get currently-open file path
				var currentlyOpenTabFilePath = editor.document.fileName;
				var currentlyOpenTabFileName = path.basename(currentlyOpenTabFilePath);
				if (!vscode.workspace.workspaceFolders) {
					vscode.window.showInformationMessage("Open a folder/workspace first");
					return;
				}
				else {
					var currentlyOpenDirectory = vscode.workspace.workspaceFolders[0].uri.fsPath;
				}

				const terminalOptions = { name: termName, closeOnExit: true };
				var terminal = vscode.window.terminals.find(terminal => terminal.name === termName);
				terminal = terminal ? terminal : vscode.window.createTerminal(terminalOptions);
				terminal.show();

				// TODO: Create a bin/ dir which contains the compiled binaries for each OS
				// Build and run assembler
				terminal.sendText(`cd ${nameASPath}`);
				terminal.sendText(`cargo build --release`);
				terminal.sendText(`cargo run ${nameDefaultCfgPath} ${currentlyOpenTabFilePath} ${currentlyOpenDirectory}/${currentlyOpenTabFileName}.o`);
				
				// Build and run emulator
				terminal.sendText(`cd ${nameEMUPath}`);
				terminal.sendText('cargo build --release');
				terminal.sendText(`cargo run 63321 ${currentlyOpenTabFilePath} ${currentlyOpenDirectory}/${currentlyOpenTabFileName}.o ${currentlyOpenDirectory}/${currentlyOpenTabFileName}.o.li`);

			}
		})
	);
	context.subscriptions.push(
		vscode.commands.registerCommand("extension.vsname.startAndDebug", () => {
			vscode.commands.executeCommand('extension.vsname.startEmu');

			setTimeout(() => {
				vscode.commands.executeCommand('workbench.action.debug.start');
			}, 6000);
		})
	);

	// debug adapters can be run in different ways by using a vscode.DebugAdapterDescriptorFactory:
	switch (runMode) {
		case 'server':
			// run the debug adapter as a server inside the extension and communicate via a socket
			activateNameDebug(context, new NameDebugAdapterServerDescriptorFactory());
			break;

		case 'external': default:
			// run the debug adapter as a separate process
			//activateNameDebug(context, new DebugAdapterExecutableFactory());
			break;

		case 'inline':
			// run the debug adapter inside the extension and directly talk to it
			activateNameDebug(context);
			break;
	}

}

// This method is called when your extension is deactivated
export function deactivate() {}

class NameDebugAdapterServerDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {

	private server?: Net.Server;

	createDebugAdapterDescriptor(session: vscode.DebugSession, executable: vscode.DebugAdapterExecutable | undefined): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {

		// make VS Code connect to debug server
		return new vscode.DebugAdapterServer(63321);
	}

	dispose() {
		if (this.server) {
			this.server.close();
		}
	}
}
