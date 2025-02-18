import * as vscode from 'vscode';
import { getBinName } from '../helpers';

const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');

export function registerAssemble(context: vscode.ExtensionContext, name_bin_directory: string) {
    // Driver code for spawning assembler process
	vscode.commands.registerCommand('name-ext.assemblecurrentfile', () => {
        let chained = false;    // Indicate that this command is not appearing in a chain of commands

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
}

const outputChannel = vscode.window.createOutputChannel('NAME-AS');

export function runAssembler(name_bin_dir: string, infile: string, outfile: string, chained: boolean): Promise<string> {
    // Wrap in promise because external process involved
    return new Promise((resolve, reject) => {
        const assemblerPath = path.join(name_bin_dir, getBinName('name-as'));
        if (!fs.existsSync(assemblerPath)) {

            console.log('Assembler not found at path: ' + assemblerPath);

            reject(new Error(`Assembler not found at path: ${assemblerPath}`));
            return;
        }

        const assembler = spawn(assemblerPath, [infile, outfile]);

        let hasErrors = false;
        let errorBuffer = '';

        assembler.stderr.on('data', (data: Buffer) => {
            hasErrors = true;
            errorBuffer += data.toString();
        });

        // Handle if error occurs
        assembler.on('close', (code: any) => {
            if (hasErrors) {
                outputChannel.clear();
                outputChannel.append(errorBuffer);
                outputChannel.show(true);

                console.log('Assembly failed.');
                reject(new Error('Assembly failed. Check output for details.'));
            } else {
                outputChannel.clear();
                outputChannel.append('File assembled successfully.');

                if (!chained) {
                    outputChannel.show(true);
                }
                
                console.log('Successful assembly');
                resolve('Assembly was successful.');
            }
        });
    });
}