use name_core::{parse::parse::AstKind, structs::Register};

/*
Each pseudo instruction must implement its own `expand` fn. This function expands the pseudoinstruction's content into its respective instructions.

It does this either by mapping existing arguments, or by creating new ones based on existing. Take `li` and `la` as examples, respectively.

It does not need to check its own argument setup. It can just piggy-back off existing logic from the main instruction assembly.
Any errors will clearly have code ID10T on the part of the user attempting to use the pseudoinstruction.
*/

pub(crate) type ExpansionFn = fn(Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String>;

pub(crate) fn expand_b(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    let target = args[0].clone();

    let zero = AstKind::Register(Register::Zero);

    Ok(vec![
        // beq $zero, $zero, target
        ("beq", vec![zero.clone(), zero, target]),
    ])
}

pub(crate) fn expand_bnez(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    if args.len() < 2 {
        return Err(format!(" - `bnez` expected 1 argument, got {}", args.len()));
    }

    let rs = args[0].clone();
    let rt = AstKind::Register(Register::Zero);
    let label = args[1].clone();

    Ok(vec![
        // bne $rs, $zero, label
        ("bne", vec![rs, rt, label]),
    ])
}

pub(crate) fn expand_li(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    if args.len() < 2 {
        return Err(format!(" - `li` expected 2 arguments, got {}", args.len()));
    }

    let rd = args[0].clone();
    let imm = args[1].clone();
    let zero: AstKind = AstKind::Register(name_core::structs::Register::Zero);

    Ok(vec![
        // ori  $rd, $zero, imm
        ("ori", vec![rd, zero, imm]),
    ])
}

pub(crate) fn expand_la(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    if args.len() < 2 {
        return Err(format!(" - `la` expected 2 arguments, got {}", args.len()));
    }

    let rd = args[0].clone();
    let label = args[1].clone();

    // Prepare for assembly.
    Ok(vec![
        // lui  $rd, 0
        ("lui", vec![rd.clone(), label.clone()]),
        // ori  $rd, $rd, 0
        ("ori", vec![rd.clone(), rd.clone(), label]),
    ])
}

pub(crate) fn expand_move(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    if args.len() < 2 {
        return Err(format!(" - `mv` expected 2 arguments, got {}", args.len()));
    }

    let rd = args[0].clone();
    let rs = args[1].clone();

    let zero = AstKind::Register(Register::Zero);

    Ok(vec![
        // add  $rd, $rs, $0
        ("add", vec![rd, rs, zero]),
    ])
}

/// Expanded instruction, which is kind of like a pseudoinstruction but easier.
pub(crate) fn expand_s_d(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    Ok(vec![
        // sdc1 <args>
        ("sdc1", args),
    ])
}
