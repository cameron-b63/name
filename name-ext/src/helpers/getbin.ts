// Generalization of os-specific binary grabbing
export function getBinName(nomenclature: string) : string {
    const isWindows = process.platform === 'win32';
    return isWindows ? `${nomenclature}.exe` : nomenclature;
}