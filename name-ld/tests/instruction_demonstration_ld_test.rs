use std::path::PathBuf;

use name_core::elf_def::Elf;
use name_core::elf_utils::{read_bytes_to_elf, write_elf_to_file};

use name_ld::linker::linker;

#[test]
fn instruction_demonstration_ld_test() {
    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("[*] FATAL: No parent directory found (did you clone the entire repo?)")
        .join("tests")
        .join("samples");

    let single_module_input_fn = base_path.join("mips_test.o");
    let single_module_output_fn = base_path.join("mips_test");

    let single_file_contents: Vec<u8> =
        std::fs::read(single_module_input_fn).expect("Unable to open object file");
    let constructed_elf: Elf = match read_bytes_to_elf(single_file_contents) {
        Ok(relocatable) => relocatable,
        Err(e) => panic!("{e}"),
    };

    let executable_contents: Elf = match linker(vec![constructed_elf]) {
        Ok(result) => result,
        Err(e) => {
            panic!("{e}");
        }
    };

    match write_elf_to_file(&single_module_output_fn, &executable_contents) {
        Ok(_) => println!("Single module linking performed successfully."),
        Err(e) => panic!("{e}"),
    }
}
