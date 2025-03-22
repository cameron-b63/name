use std::fs::read;
use std::path::PathBuf;

use name_core::elf_def::Elf;
use name_core::elf_utils::read_bytes_to_elf;

use name_emu::simulator;

#[test]
fn fib_emu_test() {
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("[*] FATAL: No parent directory found (did you clone the entire repo?)")
        .join("tests")
        .join("samples");

    let input_fn: PathBuf = base_path.join("fib");

    let elf_contents: Vec<u8> =
        read(input_fn).expect("[*] FATAL: NAME cannot run files that don't exist...");

    let executable: Elf = match read_bytes_to_elf(elf_contents) {
        Ok(elf) => elf,
        Err(e) => panic!("{}", e),
    };

    // TODO: compliance check executable (format, existence, etc.)

    let simulator_result = simulator::simulate(executable, false, false);

    match simulator_result {
        Ok(_) => {}
        Err(e) => panic!("{e}"),
    }

    // NOTE: most important test case ever.
    assert_eq!(1 + 1, 2);
}
