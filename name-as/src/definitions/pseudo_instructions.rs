use crate::definitions::expandables::*;
use crate::definitions::structs::PseudoInstruction;

/// Storing the pseudo-instruction information in another dope vector.
/// This will be formatted as a Hashmap linking the mnemonic to its expansion function.
/// The expansion functions operate on AST nodes.
pub(crate) const PSEUDO_INSTRUCTION_SET: &[PseudoInstruction] = &[
    PseudoInstruction {
        mnemonic: "b",
        expand: expand_b,
    },
    PseudoInstruction {
        mnemonic: "bnez",
        expand: expand_bnez,
    },
    PseudoInstruction {
        mnemonic: "li",
        expand: expand_li,
    },
    PseudoInstruction {
        mnemonic: "la",
        expand: expand_la,
    },
    PseudoInstruction {
        mnemonic: "move",
        expand: expand_move,
    },
    PseudoInstruction {
        mnemonic: "mv",
        expand: expand_move,
    },
    PseudoInstruction {
        mnemonic: "s.d",
        expand: expand_s_d,
    }
];
