import { spawn } from 'child_process';
import path from 'path';
import fs from 'fs';
import * as vscode from 'vscode';

const outputChannel = vscode.window.createOutputChannel('NAME');

export function runAssembler(name_install_dir: string, infile: string, outfile: string): Promise<string> {
    // Wrap in promise because external process involved
    return new Promise((resolve, reject) => {
        const assemblerPath = path.join(name_install_dir, 'bin', 'name-as');
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
                outputChannel.show(true);
                console.log('Assembly failed.');
                reject(new Error('Assembly failed. Check output for details.'));
            } else {
                outputChannel.clear();
                outputChannel.append('File assembled successfully.');
                outputChannel.show(true);
                console.log('Successful assembly');
                resolve('Assembly was successful.');
            }
        });
    });
}

export function runLinker(name_install_dir: string, infiles: string[], outfile: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const linkerPath = path.join(name_install_dir, 'bin', 'name-ld');
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
                outputChannel.show(true);
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

export function runWithoutDebugging(name_install_dir: string, infile: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const runnerPath = path.join(name_install_dir, 'bin', 'name-emu');
        if (!fs.existsSync(runnerPath)) {
            console.log('Runner not found at path: ' + runnerPath);
            reject(new Error(`Runner not found at path: ${runnerPath}`));
            return;
        }

        const runner = spawn(runnerPath, [infile]);

        let hasErrors = false;
        let errorBuffer = '';
        let outputBuffer = '';

        runner.stderr.on('data', (data: Buffer) => {
            hasErrors = true;
            errorBuffer += data.toString();
        });

        runner.stdout.on('data', (data: Buffer) => {
            outputBuffer += data.toString();
        });

        runner.on('close', (code) => {
            outputChannel.clear();
            if (hasErrors) {
                outputChannel.append(errorBuffer);
                outputChannel.show(true);
                console.log('Running failed.');
                reject(new Error('Running failed. Check output for details.'));
            } else {
                outputChannel.append(outputBuffer);
                outputChannel.show(true);
                console.log('Successful run');
                resolve('File ran successfully.');
            }
        });
    });
}