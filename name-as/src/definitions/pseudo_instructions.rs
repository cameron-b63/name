use crate::definitions::expandables::*;
use crate::definitions::structs::PseudoInstruction;

pub(crate) const PSEUDO_INSTRUCTION_SET: &[PseudoInstruction] = &[
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
    // // this pseudoinstruction doesn't seem to be officially defined but it's w/e for now
    // PseudoInstruction {
    //     mnemonic: "bnez",
    //     expand: expand_bnez,
    // },
];
