use name_core::instruction::{information::InstructionInformation, instruction_set::INSTRUCTION_SET};
use std::collections::HashMap;
use std::sync::LazyLock;

pub(crate) const BACKPATCH_PLACEHOLDER: u32 = 0;

pub(crate) const MIN_U16: i32 = -0xFFFF;
pub(crate) const MAX_U16: i32 = 0xFFFF;

pub(crate) static INSTRUCTION_TABLE: LazyLock<
    HashMap<&'static str, &'static InstructionInformation>,
> = LazyLock::new(|| {
    INSTRUCTION_SET
        .iter()
        .map(|info| (info.mnemonic, info))
        .collect()
});
