import * as vscode from 'vscode';
import { getBinName } from '../helpers';

const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');


// This function registers the "name-ext.runnodebug" command with vscode.
export function registerRunNoDebug(context: vscode.ExtensionContext, name_bin_directory: string) {
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
}

// This function simply invokes the emulator as a subprocess in a new terminal window on the given infile. It does NOT do debugging.
export function runWithoutDebugging(name_bin_dir: string, infile: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const runnerPath = path.join(name_bin_dir, getBinName('name-emu'));
        if (!fs.existsSync(runnerPath)) {
            console.log('Runner not found at path: ' + runnerPath);
            reject(new Error(`Runner not found at path: ${runnerPath}`));
            return;
        }

        const terminal = vscode.window.createTerminal('NAME Runner');
        terminal.sendText(`${runnerPath} "${infile}"`);
        terminal.show();

        resolve('Runner launched in terminal.');
    });
}