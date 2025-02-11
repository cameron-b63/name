import { spawn } from 'child_process';
import path from 'path';
import fs from 'fs';
import * as vscode from 'vscode';

const outputChannel = vscode.window.createOutputChannel('NAME');

export function runAssembler(name_bin_dir: string, infile: string, outfile: string, chained: boolean): Promise<string> {
    // Wrap in promise because external process involved
    return new Promise((resolve, reject) => {
        const assemblerPath = path.join(name_bin_dir, 'name-as');
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
        assembler.on('close', (code) => {
            if (hasErrors) {
                outputChannel.clear();
                outputChannel.append(errorBuffer);
                
                if(!chained){
                    outputChannel.show(true);
                }

                console.log('Assembly failed.');
                reject(new Error('Assembly failed. Check output for details.'));
            } else {
                outputChannel.clear();
                outputChannel.append('File assembled successfully.');

                if(!chained){
                    outputChannel.show(true);
                }

                console.log('Successful assembly');
                resolve('Assembly was successful.');
            }
        });
    });
}

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

        linker.on('close', (code) => {
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
                
                if(!chained){
                    outputChannel.show(true);
                }

                console.log('Successful linking');
                resolve('Linking was successful.');
            }
        });
    });
}

export function runWithoutDebugging(name_bin_dir: string, infile: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const runnerPath = path.join(name_bin_dir, 'name-emu');
        if (!fs.existsSync(runnerPath)) {
            console.log('Runner not found at path: ' + runnerPath);
            reject(new Error(`Runner not found at path: ${runnerPath}`));
            return;
        }

        const terminal = vscode.window.createTerminal('NAME Runner');
        terminal.sendText(`${runnerPath} ${infile}`);
        terminal.show();

        resolve('Runner launched in terminal.');
    });
}
