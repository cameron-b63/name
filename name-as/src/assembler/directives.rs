use std::fs::read_to_string;
use std::path::PathBuf;

use crate::assembler::assemble_file::assemble;
use crate::assembler::assembler::Assembler;

use crate::definitions::structs::LineComponent;

use name_core::structs::Section;

impl Assembler {
    // .asciiz
    pub(crate) fn add_new_asciiz(&mut self, arguments: &Vec<LineComponent>) {
        if arguments.len() != 1 {
            self.string_error(format!(
                " - `.asciiz` directive expects only one argument, received {}",
                arguments.len()
            ));
            return;
        }

        let mut to_push: Vec<u8> = arguments[0]
            .to_string()
            // Escape sequences
            .replace(r"\n", "\n")
            .replace(r"\t", "\t")
            .replace(r"\\", "\\")
            .chars()
            .map(|c| c as u8)
            .collect::<Vec<u8>>();

        to_push.push(b'\0');

        // TODO: This should really be refactored to implement.
        match self
            .symbol_table
            .iter_mut()
            .find(|s| s.identifier == self.most_recent_label)
        {
            Some(res) => res.size = to_push.len() as u32,
            None => {}
        }
    }

    // .eqv
    pub(crate) fn new_eqv(&mut self, arguments: &Vec<LineComponent>) {
        if arguments.len() < 2 {
            self.string_error(format!(
                "[*] On line {}{}:",
                self.line_prefix, self.line_number
            ));
            self.string_error(format!(
                " - `.eqv` expected 2 or more arguments, received {}.",
                arguments.len()
            ));
            return;
        }

        let name: String;
        let expansion: String;

        match &arguments[0] {
            LineComponent::Identifier(ident) => name = ident.clone(),
            _ => {
                self.string_error(format!(
                    "[*] On line {}{}:",
                    self.line_prefix, self.line_number
                ));
                self.string_error(format!(
                    " - `.eqv` expected identifier, found {:?}",
                    &arguments[0]
                ));
                return;
            }
        }

        expansion = arguments
            .iter()
            .skip(1)
            .map(|component| component.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        // self.equivalences.insert(name, expansion);
    }

    // .include
    pub(crate) fn include_file_old(&mut self, arguments: &Vec<LineComponent>) {
        // if arguments.len() != 1 {
        //     self.string_error(format!(
        //         "[*] On line {}{}:",
        //         self.line_prefix, self.line_number
        //     ));
        //     self.string_error(format!(" - Too many arguments for .include directive provided on line {} (expected 1 argument)", self.line_number));
        //     return;
        // }
        //
        // let filename: PathBuf = match arguments[0].clone() {
        //     LineComponent::DoubleQuote(quoted_filename) => {
        //         self.current_dir.join(PathBuf::from(quoted_filename))
        //     }
        //     _ => {
        //         self.string_error(format!(
        //             "[*] On line {}{}:",
        //             self.line_prefix, self.line_number
        //         ));
        //         self.string_error(format!(
        //             " - .include expects a double quoted string filename"
        //         ));
        //         return;
        //     }
        // };
        //
        // println!("\n[+] Found .include, attempting to include {:?}", filename);
        //
        // let file_contents = match read_to_string(&filename) {
        //     Ok(content) => content,
        //     Err(_) => {
        //         self.string_error(format!(
        //             "[*] On line {}{}:",
        //             self.line_prefix, self.line_number
        //         ));
        //         self.string_error(format!(
        //             "Could not open file {:?} referenced on line {}.",
        //             filename, self.line_number
        //         ));
        //         return;
        //     }
        // };
        //
        // let line_prefix: String = format!("{}  {}->", self.line_prefix, self.line_number);
        //
        // let returned_assembler = assemble(&file_contents, filename, Some(line_prefix));
        // match returned_assembler {
        //     Ok(returned_env) => {
        //         self.equivalences.extend(returned_env.equivalences);
        //     }
        //     Err(errors) => {
        //         self.errors.extend(errors);
        //     }
        // }
        //
        // println!("[+] Module included.\n");
        todo!("remove this")
    }

    // .text
    pub(crate) fn switch_to_text_section(&mut self) {
        match self.current_section {
            Section::Null => {
                self.current_address = self.text_address;
            }
            Section::Text => {
                self.string_error(format!(
                    "[*] On line {}{}:",
                    self.line_prefix, self.line_number
                ));
                self.string_error(format!(" - Cannot declare current_section .text when already in current_section .text on line {}", self.line_number));
            }
            Section::Data => {
                self.data_address = self.current_address;
                self.current_address = self.text_address;
            }
        }

        self.current_section = Section::Text;
    }

    // .data
    pub(crate) fn switch_to_data_section(&mut self) {
        match self.current_section {
            Section::Null => self.current_address = self.data_address,
            Section::Text => {
                self.text_address = self.current_address;
                self.current_address = self.data_address;
            }
            Section::Data => {
                self.string_error(format!(
                    "[*] On line {}{}:",
                    self.line_prefix, self.line_number
                ));
                self.string_error(format!(" - Cannot declare current_section .data when already in current_section .data (line {})", self.line_number));
            }
        }

        self.current_section = Section::Data;
    }

    // .word
    pub(crate) fn new_word(&mut self, arguments: &Vec<LineComponent>) {
        if arguments.len() == 1 {
            let value = match arguments[0] {
                LineComponent::Immediate(imm) => imm,
                _ => {
                    self.string_error(format!(" - `.word` expected a word immediate."));
                    return;
                }
            };

            let to_push = value.to_be_bytes().to_vec();

            self.current_address += to_push.len() as u32;
            self.section_dot_data.extend(&to_push);

            // TODO: This should really be refactored to implement.
            match self
                .symbol_table
                .iter_mut()
                .find(|s| s.identifier == self.most_recent_label)
            {
                Some(res) => res.size = to_push.len() as u32,
                None => {}
            }

            return;
        }

        let repetition: bool = arguments
            .iter()
            .any(|arg| matches!(arg, LineComponent::Colon));

        if repetition {
            if arguments.len() != 3 {
                self.string_error(format!(" - When using `.word` with repetition, expected usage is `.word <value> : <repeat>; expected 3 args, got {}", arguments.len()));
            }

            let (value, repeat) = {
                let val = match arguments[0] {
                    LineComponent::Immediate(imm) => imm,
                    _ => {
                        self.string_error(format!(" - When using `.word` with repetition, expected usage is `.word <value> : <repeat>; expected value of type Immediate, got {:?}", arguments[0]));
                        return;
                    }
                };

                let rep = match arguments[2] {
                    LineComponent::Immediate(imm) => imm,
                    _ => {
                        self.string_error(format!(" - When using `.word` with repetition, expected usage is `.word <value> : <repeat>; expected repeat of type Immediate, got {:?}", arguments[0]));
                        return;
                    }
                };

                if rep < 1 {
                    self.string_error(format!(" - When using `.word` with repetition, one would think you'd think you wanna repeat a positive number of times greater than zero..."));
                }

                (val, rep)
            };

            let words: Vec<i32> = vec![value; repeat as usize];
            let mut to_push: Vec<u8> = vec![];
            words
                .iter()
                .for_each(|value| to_push.extend(value.to_be_bytes().to_vec()));

            self.current_address += to_push.len() as u32;
            self.section_dot_data.extend(&to_push);

            // TODO: This should really be refactored to implement.
            match self
                .symbol_table
                .iter_mut()
                .find(|s| s.identifier == self.most_recent_label)
            {
                Some(res) => res.size = to_push.len() as u32,
                None => {}
            }
        } else {
            todo!("Create word array given those values");
        }
    }
}
