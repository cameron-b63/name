use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::process::Stdio;
use std::{cell::LazyCell, collections::HashMap};

const TESTS: LazyCell<HashMap<&'static str, (&'static str, &'static str)>> = LazyCell::new(|| {
    let mut tests = HashMap::new();
    tests.insert(
        "fib",
        (
            "",
            "The Fibonacci numbers are:\n1 1 2 3 5 8 13 21 34 55 89 144 \n",
        ),
    );
    tests.insert("hello_world", ("", "Hello, World!\n"));
    tests.insert(
        "char_test",
        ("a", "\n\t\\aahello\nworlde\ti am swagalicious\\\\'\\"),
    );
    tests.insert("mips_test", ("", "Cello, World!\n-3.14\n3.14\n"));
    tests
});

#[test]
fn assemble_to_emu_test() {
    // build the executables
    assert!(process::Command::new(env!("CARGO"))
        .args(["build"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run cargo build")
        .wait()
        .expect("failed to wait on cargo build")
        .success());

    // find the
    let [assembler, linker, emulator] = ["name-as", "name-ld", "name-emu"]
        .into_iter()
        .map(|exe| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("target")
                .join("debug")
                .join(exe)
        })
        .collect::<Vec<PathBuf>>()
        .try_into()
        .expect("failed to find binary paths");

    for (test, (input, output)) in TESTS.iter() {
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("tests")
            .join("samples")
            .join(format!("{}.asm", test));

        if !file_path.exists() {
            println!("File path \"{:?}\" leads nowhere. Continuing...", file_path);
            continue;
        }

        dbg!(&file_path);

        let file_paths = vec![
            file_path.with_extension("bin"),
            file_path.with_extension("o"),
            file_path,
        ];

        let [binary, object, asm] = file_paths
            .iter()
            .map(|fp| fp.to_str().expect("couldn't convert file_path to str"))
            .collect::<Vec<&str>>()
            .try_into()
            .expect("word");

        let mut process_asm = process::Command::new(&assembler)
            .args([asm, object])
            .spawn()
            .expect("name-as failed");

        assert!(process_asm
            .wait()
            .expect("name-as failed on wait")
            .success());

        let mut process_ld = process::Command::new(&linker)
            .args([object, "-o", binary])
            .spawn()
            .expect("name-ld failed");

        assert!(process_ld.wait().expect("name-ld failed on wait").success());

        let mut process_emu = process::Command::new(&emulator)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args([binary])
            .spawn()
            .expect(&format!("name-emu failed {:?}", binary));

        let mut emu_stdin = process_emu.stdin.take().expect("handle present");

        emu_stdin
            .write_all(input.as_bytes())
            .expect("can write to emu stdin");

        assert_eq!(
            &String::from_utf8_lossy(
                process_emu
                    .wait_with_output()
                    .expect("should wait with ouput")
                    .stdout
                    .as_slice()
            ),
            output
        );
    }
}
