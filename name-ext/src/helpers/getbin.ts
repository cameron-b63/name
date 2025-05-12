// Generalization of os-specific binary grabbing
// Creates a name for the binary file depending on platform and architecture.
export function getBinName(nomenclature: string): string {
    let binName: string;

    switch (process.platform) {
        case 'win32':
            binName = `${nomenclature}.exe`;
            break;
        case 'darwin':
            // Suffix the executable depending on which 
            // type of architecture the Mac uses
            if (process.arch === 'x64') {
                binName = `${nomenclature}_x86_64`;
            } else {
                binName = `${nomenclature}_arm64`;
            }
            
            // Add an .app extension for all Mac versions
            binName = binName + '.app';
            break;
        case 'linux':
            binName = `${nomenclature}.bin`;
            break;
        default:
            throw Error('Unknown platform');
    }

    return binName;
}