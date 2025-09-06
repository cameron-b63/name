use super::expandables::*;

#[derive(Debug)]
pub struct PseudoInstruction {
    pub mnemonic: &'static str,
    pub expand: ExpansionFn,
    pub lines_expanded_to: usize,
}

/// Storing the pseudo-instruction information in another dope vector.
/// This will be formatted as a Hashmap linking the mnemonic to its expansion function.
/// The expansion functions operate on AST nodes.
pub const PSEUDO_INSTRUCTION_SET: &[PseudoInstruction] = &[
    PseudoInstruction {
        mnemonic: "b",
        expand: expand_b,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "bal",
        expand: expand_bal,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "bnez",
        expand: expand_bnez,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "ehb",
        expand: expand_ehb,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "li",
        expand: expand_li,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "la",
        expand: expand_la,
        lines_expanded_to: 2,
    },
    PseudoInstruction {
        mnemonic: "move",
        expand: expand_move,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "move",
        expand: expand_move,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "mv",
        expand: expand_move,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "nop",
        expand: expand_nop,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "pause",
        expand: expand_pause,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "s.d",
        expand: expand_s_d,
        lines_expanded_to: 1,
    },
    PseudoInstruction {
        mnemonic: "ssnop",
        expand: expand_ssnop,
        lines_expanded_to: 1,
    },
];
