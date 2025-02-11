import * as vscode from 'vscode';

const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');

export function registerLink(context: vscode.ExtensionContext, name_bin_directory: string) {
	// Driver code for spawning linker process
	vscode.commands.registerCommand('name-ext.linkcurrentfile', async () => {
        let chained = false;    // Indicate that this command is not appearing in a chain of commands

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
		runLinker(name_bin_directory, infiles, output_file, chained).then((success_message: string) => {
			vscode.window.showInformationMessage(success_message);
		}).catch((error_message: string) => {
			vscode.window.showErrorMessage(error_message);
		});

	});
}

const outputChannel = vscode.window.createOutputChannel('NAME');

export function runLinker(name_bin_dir: string, infiles: string[], outfile: string, chained: boolean): Promise<string> {
    return new Promise((resolve, reject) => {
        const linkerPath = path.join(name_bin_dir, 'name-ld');
        if (!fs.existsSync(linkerPath)) {
            console.log('Linker not found at path: ' + linkerPath);
            reject(new Error(`Linker not found at path: ${linkerPath}`));
            return;
        }

        const linker = spawn(linkerPath, ['--output-filename', outfile, ...infiles]);

        let hasErrors = false;
        let errorBuffer = '';
        
        linker.stderr.on('data', (data: Buffer) => {
            hasErrors = true;
            errorBuffer += data.toString();
        });

        linker.on('close', (code: any) => {
            if (hasErrors) {
                outputChannel.clear();
                outputChannel.append(errorBuffer);
                                
                if(!chained){
                    outputChannel.show(true);
                }

                console.log('Linking failed.');
                reject(new Error('Linking failed. Check output for details.'));
            } else {
                outputChannel.clear();
                outputChannel.append('Files linked successfully.');
                outputChannel.show(true);
                console.log('Successful linking');
                resolve('Linking was successful.');
            }
        });
    });
}