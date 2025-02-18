import * as vscode from 'vscode';
import { registerCommands } from './commands';
import { NAMETreeDataProvider } from './tree';

const path = require('path');	// For OS-agnostic paths

// Activate method registers everything
export function activate(context: vscode.ExtensionContext) {

	// Register tree view for sidebar
	const treeDataProvider = new NAMETreeDataProvider();
	vscode.window.createTreeView('name-ext.tree', { treeDataProvider });

	// Register commands
	registerCommands(context);
}

// Not yet needed
export function deactivate() {}