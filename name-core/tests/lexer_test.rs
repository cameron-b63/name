use std::{fs, path::PathBuf};

use name_core::parse::lexer::Lexer;

#[test]
fn lexer_no_fail_test() {
    let samples_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("[*] FATAL: No parent directory found (did you clone the entire repo?)")
        .join("tests")
        .join("samples");

    for file_path in fs::read_dir(samples_path)
        .expect("Could not read the samples directory")
        .filter_map(|mentry| {
            mentry.ok().and_then(|entry| {
                let path = entry.path();
                if let Some("asm") = path.extension().and_then(|e| e.to_str()) {
                    Some(path)
                } else {
                    None
                }
            })
        })
    {
        let cont: String =
            fs::read_to_string(&file_path).expect("couldn't read file path to string");
        let mut lexer = Lexer::new(&cont);

        loop {
            let tok = lexer
                .next_tok()
                .unwrap()
                .unwrap_or_else(|e| panic!("{:?} {:?}", &file_path, e));
            if tok.is_eof() {
                break;
            }
        }
    }
}
