import * as vscode from 'vscode';
import { registerCommands } from './commands';
import { NAMETreeDataProvider } from './tree';

// Activate method registers everything
// NOTE: ENTRY POINT
export function activate(context: vscode.ExtensionContext) {

	// Register dummy tree view for sidebar
	// This is actually unused because all the functionality we need is contained in the welcome screen for the sidebar.
	const treeDataProvider = new NAMETreeDataProvider();
	vscode.window.createTreeView('name-ext.tree', { treeDataProvider });

	// Register commands - keeping this in a separate file to keep extension.ts readable
	registerCommands(context);
}

// Not yet needed
export function deactivate() {}