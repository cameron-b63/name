// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	vscode.window.showInformationMessage('The NAME formatter has been activated');

	let disposable = vscode.commands.registerCommand('name-fmt.tabKeyAction', () => {

		const editor = vscode.window.activeTextEditor;

		// User configuration
		const tabBoundaries = vscode.workspace.getConfiguration('nameFmt').get('tabBoundaries', [11, 18, 36]);

		// Get the user-configured default tab position
        const defaultTabPosition = vscode.workspace.getConfiguration().get('editor.tabSize', 4);

		if (editor) {
			const cursor = editor.selection.active;
			const currentCursorPosition = cursor.character;
			vscode.window.showInformationMessage(currentCursorPosition.toString());

			// Find the nearest tab boundary
			let nextBoundary = currentCursorPosition;
			for (const boundary of tabBoundaries) {
				if (boundary > currentCursorPosition) {
					nextBoundary = boundary;
					break;
				}
			}

			// Calculate number of spaces to insert
			const spacesToInsert = (nextBoundary - currentCursorPosition) || defaultTabPosition;

			const edit = new vscode.TextEdit(
				new vscode.Range(cursor, cursor),
				' '.repeat(spacesToInsert)
			);

			const workspaceEdit = new vscode.WorkspaceEdit();
			workspaceEdit.set(editor.document.uri, [edit]);
			vscode.workspace.applyEdit(workspaceEdit);
		}
	});

	context.subscriptions.push(disposable);
}

// This method is called when your extension is deactivated
export function deactivate() {}
