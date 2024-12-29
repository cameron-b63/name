use crate::assembler::assembler::Assembler;

use crate::assembler::assembly_helpers::{reverse_format_instruction, search_mnemonic};
use crate::definitions::structs::{BackpatchType, LineComponent, PseudoInstruction};

use name_core::instruction::information::InstructionInformation;

use crate::parser::parse_components;

/*

I can understand that this assemble function may at first seem to be kind of a behemoth.

The logic is as follows:
- Break each line into its components and specify by type what needs to happen for each component
- If an instruction was present, retrieve its information from the constant table
- If registers/immediates/identifiers are provided, push them to an arguments vector
- If symbols are encountered, attempt to resolve them. If unresolvable, save them to the environment's backpatches for fixing later.
- After all this is said and done, call the assemble_instruction helper with the arguments and symbol table if an instruction was present
- Instead, if a directive was present, call the appropriate handler.
- Update tracking variables (line_number, current_address, etc.) appropriately

*/

pub fn assemble_line(environment: &mut Assembler, line: &str, expanded_line: String) {
    println!(
        "{}{}: {}",
        environment.line_prefix, environment.line_number, line
    );

    let line_components_result = parse_components(expanded_line.to_string());

    // Unpack the line_components_result so we can process the line properly.
    let line_components;
    match line_components_result {
        Ok(components) => line_components = components,
        Err(e) => {
            environment.errors.push(format!(
                "[*] On line {}{}:",
                environment.line_prefix, environment.line_number
            ));
            environment.errors.push(e);
            return;
        }
    }

    if line_components.is_none() {
        return;
    }

    let mut instruction_information: Option<&'static InstructionInformation> = None;
    let mut pseudo_instruction_information: Option<&'static PseudoInstruction> = None;
    let mut found_directive: Option<String> = None;
    let mut arguments: Vec<LineComponent> = vec![];

    for component in line_components.unwrap() {
        match component {
            LineComponent::Mnemonic(mnemonic) => {
                (instruction_information, pseudo_instruction_information) =
                    search_mnemonic(mnemonic, environment);
            }
            LineComponent::Identifier(content) => {
                arguments.push(LineComponent::Identifier(content.clone()));

                // If the symbol does not exist in the symbol table, a backpatch must be created.
                if !environment.symbol_exists(&content) {
                    match instruction_information {
                        Some(instruction_info) => {
                            environment.add_backpatch(
                                instruction_info,
                                &arguments,
                                content,
                                BackpatchType::Standard,
                            );
                            println!(
                                " - Forward reference detected (line {}{}).",
                                environment.line_prefix, environment.line_number
                            );
                        }
                        None => {
                            // If there's no instruction information on this line, the identifier is likely associated with a preprocessor macro.
                            // Nothing else needs to be done at this time.
                        }
                    }
                }
            }
            LineComponent::Label(content) => {
                environment.add_label(&content);
                environment.resolve_backpatches(&content);
            }
            LineComponent::Directive(content) => {
                found_directive = Some(content.clone());
            }
            LineComponent::Register(_)
            | LineComponent::Immediate(_)
            | LineComponent::DoubleQuote(_)
            | LineComponent::Colon => {
                arguments.push(component);
            }
        }
    }

    // If a known instruction mnemonic was discovered, its contents will be assembled here.
    match instruction_information {
        None => {}
        Some(info) => {
            environment.handle_assemble_instruction(info, &arguments);
        }
    }

    // If a known pseudoinstruction mnemonic was discovered, its expansion will be assembled here.
    match pseudo_instruction_information {
        None => {}
        Some(info) => {
            // Unpack the pseudoinstruction's expansion
            // Pseudoinstructions have an associated instruction (the `expand` field) which is a fn
            // (info.expand)(&arguments) allows us to get and call that associated fn
            let resulting_tuples = match (info.expand)(environment, &arguments) {
                Ok(tuples) => tuples,
                Err(e) => {
                    environment.errors.push(format!(
                        "[*] On line {}{}",
                        environment.line_prefix, environment.line_number
                    ));
                    environment.errors.push(e);
                    return;
                }
            };

            // Create a new line_number and line_prefix scope since any printing will now be associated with the pseudoinstruction
            let old_line_prefix = environment.line_prefix.clone();

            // Of form "12->1->"
            environment.line_prefix = format!(
                "{}{}{}",
                environment.line_prefix, environment.line_number, "->"
            );

            let old_line_number = environment.line_number.clone();
            environment.line_number = 1;

            for (instr_info, args) in resulting_tuples {
                let reverse_formatted_instruction = reverse_format_instruction(instr_info, &args);
                println!(
                    "{}{}: {}",
                    environment.line_prefix, environment.line_number, reverse_formatted_instruction
                );
                environment.handle_assemble_instruction(instr_info, &args);
                environment.line_number += 1;
            }

            // Restore once multi-line scope exited
            environment.line_prefix = old_line_prefix;
            environment.line_number = old_line_number;
        }
    }

    match found_directive {
        Some(directive) => environment.handle_directive(&directive, &arguments),
        None => {}
    }
}
