use crate::{parse::parse::AstKind, structs::Register};

/*
Each pseudo instruction must implement its own `expand` fn. This function expands the pseudoinstruction's content into its respective instructions.

It does this either by mapping existing arguments, or by creating new ones based on existing. Take `li` and `la` as examples, respectively.

It does not need to check its own argument setup. It can just piggy-back off existing logic from the main instruction assembly.
Any errors will clearly have code ID10T on the part of the user attempting to use the pseudoinstruction.
*/

pub(crate) type ExpansionFn = fn(Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String>;

// b label
pub(crate) fn expand_b(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    let target = args[0].clone();

    let zero = AstKind::Register(Register::Zero);

    Ok(vec![
        // beq $zero, $zero, target
        ("beq", vec![zero.clone(), zero.clone(), target]),
    ])
}

// bal label
pub(crate) fn expand_bal(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    let target = args[0].clone();

    let zero = AstKind::Register(Register::Zero);

    Ok(vec![
        // bgezal $0, offset
        ("bgezal", vec![zero.clone(), zero.clone(), target]),
    ])
}

// bnez $rs, label
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

// ehb
pub(crate) fn expand_ehb(_args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    let zero = AstKind::Register(Register::Zero);
    Ok(vec![
        // sll $0, $0, 3
        (
            "sll",
            vec![zero.clone(), zero.clone(), AstKind::Immediate(3)],
        ),
    ])
}

// li $rd, imm
pub(crate) fn expand_li(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    if args.len() < 2 {
        return Err(format!(" - `li` expected 2 arguments, got {}", args.len()));
    }

    let rd = args[0].clone();
    let imm = args[1].clone();
    let zero: AstKind = AstKind::Register(crate::structs::Register::Zero);

    Ok(vec![
        // ori  $rd, $zero, imm
        ("ori", vec![rd, zero, imm]),
    ])
}

// la $rd, label
pub(crate) fn expand_la(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    if args.len() < 2 {
        return Err(format!(" - `la` expected 2 arguments, got {}", args.len()));
    }

    let rd = args[0].clone();
    let label = args[1].clone();

    Ok(vec![
        // lui  $rd, 0
        ("lui", vec![rd.clone(), label.clone()]),
        // ori  $rd, $rd, 0
        ("ori", vec![rd.clone(), rd.clone(), label]),
    ])
}

// mv $rd, $rs
// move $rd, $rs
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

/// This shouldn't really be a pseudoinstruction, but it makes the most sense given how nop is defined.
pub(crate) fn expand_nop(_args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    let zero = AstKind::Register(Register::Zero);
    let imm_zero = AstKind::Immediate(0);

    Ok(vec![
        // sll $0, $0, 0
        ("sll", vec![zero.clone(), zero, imm_zero]),
    ])
}

/// This also shouldn't really be a pseudoinstruction, but it makes sense.
/// PAUSE is just a special case of SLL.
/// It doesn't do anything yet so I haven't handled any special funny business.
pub(crate) fn expand_pause(_args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    let zero = AstKind::Register(Register::Zero);
    let imm_five = AstKind::Immediate(5);

    Ok(vec![
        // sll $0, $0, 5
        ("sll", vec![zero.clone(), zero, imm_five]),
    ])
}

/// This is the same as the other SLL variations with rd=0...
/// SSNOP is a special case of SLL with SLL $0, $0, 1
pub(crate) fn expand_ssnop(_args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    let zero = AstKind::Register(Register::Zero);
    let imm_one = AstKind::Immediate(1);

    Ok(vec![
        // sll $0, $0, 1
        ("sll", vec![zero.clone(), zero, imm_one])
    ])
}

/// Expanded instruction, which is kind of like a pseudoinstruction but easier.
pub(crate) fn expand_s_d(args: Vec<AstKind>) -> Result<Vec<(&'static str, Vec<AstKind>)>, String> {
    Ok(vec![
        // sdc1 <args>
        ("sdc1", args),
    ])
}
