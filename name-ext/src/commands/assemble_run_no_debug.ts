import * as vscode from 'vscode';

const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');

import { runAssembler } from './assemble';
import { runLinker } from './link';
import { runWithoutDebugging } from './emulate';


export function registerAssembleRunNoDebug(context: vscode.ExtensionContext, name_bin_directory: string) {
    // Assemble, link, and run the current file all in one go
    vscode.commands.registerCommand('name-ext.assemblerunnodebug', () => {

        let chained = true;

        // Get file name
        let currently_open_file: string | undefined = vscode.window.activeTextEditor?.document.fileName;
        if (currently_open_file === undefined) {
            vscode.window.showErrorMessage('No file open.');
            return;
        }

        // Create output file name
        let assembler_output_file = currently_open_file ? path.format({ ...path.parse(currently_open_file), base: undefined, ext: '.o' }) : undefined;

        // Call assembler
        runAssembler(name_bin_directory, currently_open_file, assembler_output_file, chained).then((success_message: string) => {
            // On assembly success:

            // Create output file name
            let linker_output_file = currently_open_file ? path.format({ ...path.parse(currently_open_file), base: undefined, ext: '.o' }) : undefined;

            // Get directory of the currently open file
            let current_dir = path.dirname(currently_open_file);

            // Get all .o files in the same directory
            let infiles: string[] = [];
            const files = fs.readdirSync(current_dir);
            for (const file of files) {
                if (file.endsWith('.o')) {
                    infiles.push(path.join(current_dir, file));
                }
            }

            // Call linker
            runLinker(name_bin_directory, infiles, linker_output_file, chained).then((success_message: string) => {
                // On link success:
                // Run the output file
                runWithoutDebugging(name_bin_directory, linker_output_file).then((success_message: string) => {
                    vscode.window.showInformationMessage(success_message);
                }).catch((error_message: string) => {
                    vscode.window.showErrorMessage(error_message);
                });

            }).catch((error_message: string) => {
                // Linker catch
                vscode.window.showErrorMessage(error_message);
            });

        }).catch((error_message: string) => {
            // Assembler catch
            vscode.window.showErrorMessage(error_message);
        });
    });
}