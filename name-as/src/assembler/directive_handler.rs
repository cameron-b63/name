use crate::assembler::assembler::Assembler;

use crate::definitions::structs::LineComponent;

impl Assembler {
    pub(crate) fn _handle_directive(&mut self, ident: &String, arguments: &Vec<LineComponent>) {
        match ident.as_str() {
            ".asciiz" => {
                // self.add_new_asciiz(arguments);
            }
            ".data" => {
                self.switch_to_data_section();
            }
            ".eqv" => {
                // self.new_eqv(arguments);
            }
            ".include" => {
                // self.include_file_old(arguments);
            }
            ".text" => {
                // self.switch_to_text_section();
            }
            ".word" => {
                self._new_word(arguments);
            }
            _ => {
                self.string_error(format!("[*] On line {}:", self.line_number));
                self.string_error(format!(" - Unrecognized directive."));
            }
        }
    }
}
