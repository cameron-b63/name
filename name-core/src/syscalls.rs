/*  System call functions.
 *
 *  Most of these are analogous to functions found in nearly every operating system,
 */

use std::io::{BufRead, Read, Write};

use crate::structs::{
    ProgramState,
    Register::{A0, A1, V0},
    FpRegister::F12,
};

// syscalls are implemented over io traits Read/Write/etc.. for testability

/// Syscall 1 - SysPrintInt
pub fn sys_print_int<W: Write>(
    program_state: &mut ProgramState,
    sys: &mut W,
) -> Result<(), String> {
    write!(
        sys,
        "{}",
        program_state.cpu.general_purpose_registers[A0 as usize]
    )
    .map_err(|_| "Failed to read")?;
    sys.flush().map_err(|_| "Failed to flush sys".to_string())
}

/// Syscall 2 - SysPrintFloat
pub fn sys_print_float<W: Write>(
    program_state: &mut ProgramState,
    sys: &mut W,
) -> Result<(), String> {
    write!(
        sys,
        "{}",
        program_state.cp1.registers[F12 as usize]
    )
    .map_err(|_| "Failed to read")?;
    sys.flush().map_err(|_| "Failed to flush sys".to_string())
}

/// Syscall 4 - SysPrintString
pub fn sys_print_string<W: Write>(
    program_state: &mut ProgramState,
    sys: &mut W,
) -> Result<(), String> {
    let mut address = program_state.cpu.general_purpose_registers[A0 as usize];
    let mut to_print: Vec<u8> = Vec::new();

    loop {
        let byte = program_state
            .memory
            .read_byte(address)
            .map_err(|e| format!("{e}"))?;

        if byte == 0 {
            break;
        }

        to_print.push(byte);
        address += 1;
    }

    let output_string: String =
        String::from_utf8(to_print).map_err(|_| "Supplied string is NOT utf-8")?;

    write!(sys, "{}", output_string).map_err(|_| "Failed to write to sys")?;
    sys.flush().map_err(|_| "Failed to flush sys".to_string())
}

/// Syscall 5 - SysReadInt
pub fn sys_read_int<R: BufRead>(
    program_state: &mut ProgramState,
    sys: &mut R,
) -> Result<(), String> {
    let mut input_text = String::new();
    sys.read_line(&mut input_text)
        .map_err(|_| "Failed to read from stdin")?;

    let trimmed = input_text.trim();
    match trimmed.parse::<u32>() {
        Ok(i) => {
            program_state.cpu.general_purpose_registers[V0 as usize] = i;
            Ok(())
        }
        Err(_) => Err(format!("Failed to convert input to Int")),
    }
}

// Syscall 8 - sys_read_string  -- Read a string from the keyboard one character at a time
// until we get either a \n character or run out of space.  We accept up to maxlength-1
// characters because the string is alwaays null-terminated.  If we get to the maximum
// length the \n will not be stored.
pub fn sys_read_string<R: Read>(
    program_state: &mut ProgramState,
    sys: &mut R,
) -> Result<(), String> {
    let mut buf = [0; 1];
    let mut count = 0;
    let mut address = program_state.cpu.general_purpose_registers[A0 as usize];
    let maxlength = program_state.cpu.general_purpose_registers[A1 as usize];
    while count < maxlength - 1 {
        match sys.read_exact(&mut buf) {
            Ok(_) => (),
            Err(_) => return Err(format!("Failed to access stdin.")),
        };

        match program_state.memory.set_byte(address, buf[0]) {
            Ok(_) => (),
            Err(e) => return Err(format!("Error occurred in read string syscall:\n - {e}")),
        };

        count += 1;
        address += 1;
        if buf[0] == b'\n' as u8 {
            break;
        }
    }
    buf[0] = 0;
    match program_state.memory.set_byte(address, buf[0]) {
        Ok(_) => (),
        Err(e) => return Err(format!("Error occurred in read string syscall:\n - {e}")),
    };

    Ok(())
}

/// Syscall 10 - SysExit
pub fn sys_exit(program_state: &mut ProgramState) -> Result<(), String> {
    program_state.should_continue_execution = false;
    Ok(())
}

/// Syscall 11 - SysPrintChar
pub fn sys_print_char<W: Write>(
    program_state: &mut ProgramState,
    sys: &mut W,
) -> Result<(), String> {
    match char::from_u32(program_state.cpu.general_purpose_registers[A0 as usize]) {
        Some(valid_char) => {
            write!(sys, "{}", valid_char).map_err(|_| "Failed to write to sys")?;
            Ok(())
        }
        None => Err(format!("Failed to convert given character to byte value.")),
    }
}

/// Syscall 12 - SysReadChar
pub fn sys_read_char<R: Read>(program_state: &mut ProgramState, sys: &mut R) -> Result<(), String> {
    let mut buf = [0; 1];
    sys.read_exact(&mut buf)
        .map_err(|_| "Failed to read from stdin")?;

    program_state.cpu.general_purpose_registers[V0 as usize] = buf[0] as u32;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::structs::{Memory, Processor, ProgramState};
    use std::io::Cursor;

    fn test_print(
        syscall: fn(&mut ProgramState, &mut Vec<u8>) -> Result<(), String>,
        program_state: &mut ProgramState,
        expected: &str,
    ) {
        let mut buf = Vec::<u8>::new();
        assert_eq!(Ok(()), syscall(program_state, &mut buf));
        assert_eq!(expected.as_bytes().to_vec(), buf);
    }

    #[test]
    fn test_sys_print_int() {
        let mut program_state = ProgramState::default();

        program_state.cpu.general_purpose_registers[A0 as usize] = 4;

        test_print(sys_print_int, &mut program_state, "4");
    }

    #[test]
    fn test_sys_print_char() {
        let mut program_state = ProgramState::default();

        program_state.cpu.general_purpose_registers[A0 as usize] = 'c' as u32;

        test_print(sys_print_char, &mut program_state, "c");
    }

    #[test]
    fn test_sys_print_string() {
        let mut program_state = ProgramState::new(
            Processor::default(),
            Memory::new("hello world\0".as_bytes().to_vec(), vec![]),
        );

        program_state.cpu.general_purpose_registers[A0 as usize] =
            crate::constants::MIPS_DATA_START_ADDR;
        test_print(sys_print_string, &mut program_state, "hello world");
    }

    #[test]
    fn test_sys_read_char() {
        let mut program_state = ProgramState::default();
        let mut cursor = Cursor::new("name".as_bytes());

        assert_eq!(Ok(()), sys_read_char(&mut program_state, &mut cursor));
        assert_eq!(
            program_state.cpu.general_purpose_registers[V0 as usize],
            'n' as u32
        );
    }

    #[test]
    fn test_sys_read_int() {
        let mut program_state = ProgramState::default();
        let mut cursor = Cursor::new("8675309".as_bytes());

        assert_eq!(Ok(()), sys_read_int(&mut program_state, &mut cursor));
        assert_eq!(
            program_state.cpu.general_purpose_registers[V0 as usize],
            8675309,
        );
    }

    #[test]
    fn test_sys_read_sting() {
        let mut str = "just throw yourself at the ground and miss"
            .as_bytes()
            .to_vec();

        let mut cursor = Cursor::new(&str);

        let mut program_state =
            ProgramState::new(Processor::default(), Memory::new([0; 16].to_vec(), vec![]));

        program_state.cpu.general_purpose_registers[A0 as usize] =
            crate::constants::MIPS_DATA_START_ADDR;
        program_state.cpu.general_purpose_registers[A1 as usize] = 16;

        assert_eq!(Ok(()), sys_read_string(&mut program_state, &mut cursor));

        // read string adds a null terminator as last char
        str[15] = 0;

        assert_eq!(program_state.memory.data[..16], str[..16]);
    }
}
