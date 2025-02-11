import * as vscode from 'vscode';

// This is a "do-nothing" class as its functionality is all contained in the welcome screen, defined in package.json.
export class NAMETreeDataProvider implements vscode.TreeDataProvider<NAMECommandItem> {
	private commands: NAMECommandItem[] = [];

	getTreeItem(element: NAMECommandItem): vscode.TreeItem {
		return element;
	}

	getChildren(): NAMECommandItem[] {
		return this.commands;
	}
}

class NAMECommandItem extends vscode.TreeItem {
	constructor(label: string, commandId: string) {
		super(label, vscode.TreeItemCollapsibleState.None);
		this.command = { command: commandId, title: label };
	}
}