use crate::{exception::definitions::ExceptionType, instruction::FpFourRegArgs, structs::ProgramState};

use super::{
    implementation_helpers::{
        extract_u64, is_register_aligned, pack_up_u64, perform_op_with_flush,
    },
    FpCCBranchArgs, FpRArgs,
};

/*


  ______ __  __ _______
 |  ____|  \/  |__   __|
 | |__  | \  / |  | |
 |  __| | |\/| |  | |
 | |    | |  | |  | |
 |_|    |_|  |_|  |_|




*/

// Special instructions: Facilitate movement GPR <-> FPU
// 0x02 - CF (Coprocessor from) - GPR -> FPU
pub fn cfc1(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("cfc1");
}

// 0x06 - CT (Coprocessor to) - GPR <- FPU
pub fn ctc1(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("ctc1");
}

// 0x08 (secondary funct code) - bc1<cond><nd>
/// All implementations (t/f, likely/unlikely) are contained in this function.
/// This simplifies the table.
pub fn bc1(program_state: &mut ProgramState, args: FpCCBranchArgs) -> () {
    // match on the type of instruction (update later to account for likely)
    match args.tf {
        0 => {
            // Branch on floating-point false (bc1f)
            if program_state.cp1.get_condition_code(args.cc) {
                return;
            }

            // Sign extend offset
            let offset: i32 = ((args.offset & 0xFFFF) as i16 as i32) << 2;
            let temp = (program_state.cpu.pc as i32 + offset) as u32;
            program_state.jump_if_valid(temp);
        }
        1 => {
            // Branch on floating-point true (bc1t)
            if !program_state.cp1.get_condition_code(args.cc) {
                return;
            }

            // Sign extend offset
            let offset: i32 = ((args.offset & 0xFFFF) as i16 as i32) << 2;
            let temp = (program_state.cpu.pc as i32 + offset) as u32;
            program_state.jump_if_valid(temp);
        }
        _ => {
            // Represents an impossible true/false. Should actually be unreachable!() but you never know...
            program_state.set_exception(ExceptionType::ReservedInstruction);
        }
    }
}

/*

   _____ ____  _____  __
  / ____/ __ \|  __ \/_ |
 | |   | |  | | |__) || |
 | |   | |  | |  ___/ | |
 | |___| |__| | |     | |
  \_____\____/|_|     |_|



*/

// 0x00 - add.fmt

// 0x00.d - add.d
pub fn add_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("add.d");
}

// 0x00.s - add.s
pub fn add_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("add.s");
}

// 0x03 - div.fmt

// 0x03.d - div.d
pub fn div_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let _ = is_register_aligned(program_state, args.fd);
    let _ = is_register_aligned(program_state, args.fs);
    let _ = is_register_aligned(program_state, args.ft);

    let numerator: f64 = f64::from_bits(extract_u64(program_state, args.fs));
    let denominator: f64 = f64::from_bits(extract_u64(program_state, args.ft));

    let result: f64 = perform_op_with_flush(program_state, numerator / denominator);

    pack_up_u64(program_state, args.fd, f64::to_bits(result));
}

// 0x05 - abs.fmt

// 0x05.d - abs.d
pub fn abs_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let _ = is_register_aligned(program_state, args.fd);
    let _ = is_register_aligned(program_state, args.fs);

    let temp: u64 = extract_u64(program_state, args.fs);

    // Simply clear the sign bit.
    // Refer to IEEE 754-2008 documentation if this appears non-sensical.
    let mask: u64 = 0x7FFF_FFFF_FFFF_FFFF;
    let result: u64 = temp & mask;

    let _ = pack_up_u64(program_state, args.fd, result);
}

// 0x05.s - abs.s
pub fn abs_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    program_state.cp1.registers[args.fd as usize] =
        f32::abs(program_state.cp1.registers[args.fs as usize]);
}

// 0x06.d - mov.d
pub fn mov_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    let _ = is_register_aligned(program_state, args.fd);
    let _ = is_register_aligned(program_state, args.fs);

    let temp: u64 = extract_u64(program_state, args.fs);
    let _ = pack_up_u64(program_state, args.fd, temp);
}

// 0x0a.d - ceil.l.d
pub fn ceil_l_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("ceil.l.d");
}

// 0x0a.s - ceil.l.s
pub fn ceil_l_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("ceil.l.s");
}

// 0x09.d - ceil.w.d
pub fn ceil_w_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("ceil.w.d");
}

// 0x09.s - ceil.w.s
pub fn ceil_w_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("ceil.w.s");
}

// 0x0b.d - floor.l.d
pub fn floor_l_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("floor.l.d");
}

// 0x0b.s - floor.l.s
pub fn floor_l_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("floor.l.s");
}

// 0x0f.d - floor.w.d
pub fn floor_w_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("floor.w.d");
}

// 0x0f.s - floor.w.s
pub fn floor_w_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("floor.w.s");
}

// 0x20.s - cvt.s.d
pub fn cvt_s_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("cvt.s.d");
}

// 0x21.s - cvt.d.s
pub fn cvt_d_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("cvt.d.s");
}

// 0x30.d - c.f.d
pub fn c_f_d(program_state: &mut ProgramState, args: FpRArgs) -> () {
    program_state.cp1.set_condition_code(args.fd >> 2, false);
}

// 0x30.d - c.f.s
pub fn c_f_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    program_state.cp1.set_condition_code(args.fd >> 2, false);
}

// 0x31.d - c.un.d
pub fn c_un_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.un.d");
}

// 0x31.s - c.un.s
pub fn c_un_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.un.s");
}

// 0x32.d - c.eq.d
pub fn c_eq_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.eq.d");
}

// 0x32.s - c.eq.s
pub fn c_eq_s(program_state: &mut ProgramState, args: FpRArgs) -> () {
    program_state.cp1.set_condition_code(
        args.fd >> 2,
        program_state.cp1.registers[args.ft as usize]
            == program_state.cp1.registers[args.fs as usize],
    );
}

// 0x33.d - c.ueq.d
pub fn c_ueq_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ueq.d");
}

// 0x33.s - c.ueq.s
pub fn c_ueq_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ueq.s");
}

// 0x34.d - c.olt.d
pub fn c_olt_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.olt.d");
}

// 0x34.s - c.olt.s
pub fn c_olt_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.olt.s");
}

// 0x35.d - c.ult.d
pub fn c_ult_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ult.d");
}

// 0x35.s - c.ult.s
pub fn c_ult_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ult.s");
}

// 0x36.d - c.ole.d
pub fn c_ole_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ole.d");
}

// 0x36.s - c.ole.s
pub fn c_ole_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ole.s");
}

// 0x37.d - c.ule.d
pub fn c_ule_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ule.d");
}

// 0x37.s - c.ule.s
pub fn c_ule_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ule.s");
}

// 0x38.d - c.sf.d
pub fn c_sf_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.sf.d");
}

// 0x38.s - c.sf.s
pub fn c_sf_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.sf.s");
}

// 0x39.d - c.ngle.d
pub fn c_ngle_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ngle.d");
}

// 0x39.s - c.ngle.s
pub fn c_ngle_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ngle.s");
}

// 0x3a.d - c.seq.d
pub fn c_seq_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.seq.d");
}

// 0x3a.s - c.seq.s
pub fn c_seq_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.seq.s");
}

// 0x3b.d - c.ngl.d
pub fn c_ngl_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ngl.d");
}

// 0x3b.s - c.ngl.s
pub fn c_ngl_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ngl.s");
}

// 0x3c.d - c.lt.d
pub fn c_lt_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.lt.d");
}

// 0x3c.s - c.lt.s
pub fn c_lt_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.lt.s");
}

// 0x3d.d - c.nge.d
pub fn c_nge_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.nge.d");
}

// 0x3d.s - c.nge.s
pub fn c_nge_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.nge.s");
}

// 0x3e.d - c.le.d
pub fn c_le_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.le.d");
}

// 0x3e.s - c.le.s
pub fn c_le_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.le.s");
}

// 0x3f.d - c.ngt.d
pub fn c_ngt_d(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ngt.d");
}

// 0x3f.s - c.ngt.s
pub fn c_ngt_s(_program_state: &mut ProgramState, _args: FpRArgs) -> () {
    todo!("c.ngt.s");
}


/*


   ____  _____  _  _   
  / __ \|  __ \| || |  
 | |  | | |__) | || |_ 
 | |  | |  ___/|__   _|
 | |__| | |       | |  
  \____/|_|       |_|  
                       
                       


*/

// 0x13/0x2_
pub fn madd_d(_program_state: &mut ProgramState, _args: FpFourRegArgs) -> () {
    todo!("madd.d");
}