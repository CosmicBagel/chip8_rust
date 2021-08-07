use std::fs::File;
use std::io::{prelude::*, stdout};
use std::thread;
use std::time;
/*
    - code ingestion
        - opcodes are 2 bytes 0xFFFF
    - registers
        data (16 0-F) 8 bits each
        address register 16bit
    - memory
        program address space 0x200 - 0x69F (2kb variant)
                              0x200 - 0xE8F (4kb variant) (3,215 bytes available)
        big-endian (x86_64 is little endian)
    - beep sound (lets make it a 4th octave G sound)
    - timers (run at 60hz)
        - delay timer
        - sound timer
    - display

    keyboard mapping:
    Keypad      Keyboard
    1 2 3 C     1 2 3 4
    4 5 6 D ->  Q W E R
    7 8 9 E     A S D F
    A 0 B F     Z X C V

    screen 64 x 32 pixels
    top left 00, 00
    bottom right 3F, 1F
*/
const DEFAULT_ROM: &str = "roms/test_opcode.ch8";
const MAX_MEMORY: usize = 3215;
const CYCLE_SLEEP_DURATION: time::Duration = time::Duration::from_millis(16);

struct EmulatorState {
    registers: [u8; 16],
    // although we need usize to access the array that the instructions are stored in
    // its better to explicitly say u16 as usize can technically be as small as u8
    address_register: u16,
    memory_space: [u8; MAX_MEMORY],
    timer_counter: u8,
    sound_counter: u8,
    program_counter: u16,
}

enum InstructionResult {
    Continue,
    Terminate,
    Jump(u16),
}

fn main() {
    let mut state = EmulatorState {
        registers: [0_u8; 16],
        address_register: 0_u16,
        memory_space: [0_u8; MAX_MEMORY],
        timer_counter: 0_u8,
        sound_counter: 0_u8,
        program_counter: 0x200_u16,
    };

    // based on 4kb variant (hence 3215 bytes) (wait shouldn't it be 3583???)
    // all memory accesses will be in big endian
    // chip 8 is big endian!!!
    // halt on bad memory access?

    // planning on making these count down on another thread
    // use a condition variable to wake the counting thread on counter set
    // just use thread sleep to try and get the right 60hz freq

    //1. load program into memory from file
    //  if it exceeds size than halt with error
    //2. begin running byte code
    // (code is allowed to be self modifying (ie no write protection region))
    // error on any address read/write below 0x200

    let bytes_read = load_program(&mut state, DEFAULT_ROM);
    println!("Loaded program, bytes {}", bytes_read);

    // run the code
    loop {
        /*
        - openOpen
        - just keep executing next instruction in memory (even if we progress into data memory?)
        - will have to implement a stack for subroutines (or at least a return pointer)
            */

        let translated_address = state.program_counter as usize - 0x200;

        let opcode_left_byte = state.memory_space[translated_address];
        let opcode_right_byte = state.memory_space[translated_address as usize + 1];

        print!("{:02x?}{:02x?}, ", opcode_left_byte, opcode_right_byte,);

        let program_counter_target = process_opcode(opcode_left_byte, opcode_right_byte);
        // ideas to handle state mutation and opcode
        // a) context bag  with everything that just get borrowed
        // b) decode opcodes, create message enum, keep data in main and just pass
        //    whats needed to specific funcs
        // c)

        // if we have a 0x0000 opcode, terminate
        if let InstructionResult::Terminate = program_counter_target {
            break;
        }

        if let InstructionResult::Jump(target) = program_counter_target {
            println!("executing jump {:#06x}", target);
            if target < state.memory_space.len() as u16 - 1 {
                state.program_counter = target;
            } else {
                panic!("Attempted to access out of bounds memory");
            }
        } else if state.program_counter + 2 < state.memory_space.len() as u16 - 1 {
            state.program_counter += 2;
        } else {
            break;
        }

        // so that stdout prints show up when printed
        stdout().flush().unwrap();
        thread::sleep(CYCLE_SLEEP_DURATION);
    }

    println!("End of program");
}

fn load_program(state: &mut EmulatorState, file_name: &str) -> usize {
    let mut rom_file = File::open(file_name).unwrap();
    rom_file.read(&mut state.memory_space).unwrap()
}

// returns desired program counter location
fn process_opcode(opcode_left_byte: u8, opcode_right_byte: u8) -> InstructionResult {
    use self::InstructionResult::*;

    // process opcode
    let fourth_nibble = (0xF0 & opcode_left_byte) >> 4;
    let _third_nibble = 0x0F & opcode_left_byte;
    let _second_nibble = (0xF0 & opcode_right_byte) >> 4;
    let _first_nibble = 0x0F & opcode_right_byte;

    // ewwwwwwwwwww
    let prepare_jump = || {
        let mut jump_addr = opcode_left_byte as u16;
        jump_addr <<= 8;
        jump_addr |= 0x00FF;
        let afsfdsa = opcode_right_byte as u16 | 0xFF00;
        jump_addr &= afsfdsa;
        jump_addr &= 0x0FFF;
        Jump(jump_addr)
    };

    match fourth_nibble {
        0x0 => {
            if _second_nibble != 0xE {
                if opcode_left_byte == 0x00 && opcode_right_byte == 0x00 {
                    println!("terminate the program");
                    return Terminate;
                }

                // not sure if 0000 should map to something specific, or result in particular
                // behaviour, could also just be padding bytes at the end of the rom?
                // it could be an exit or halt command essentially
                println!(
                    "execute machine language subroutine at address {}{}{}",
                    _third_nibble, _second_nibble, _first_nibble
                );
                prepare_jump()
            } else if _first_nibble == 0x0 {
                println!("clear the screen");
                Continue
            } else {
                println!("return from subroutine");
                Continue
            }
        }
        0x1 => {
            println!("jump");
            prepare_jump()
        }
        0x2 => Continue,
        0x3 => Continue,
        0x4 => Continue,
        0x5 => Continue,
        0x6 => Continue,
        0x7 => Continue,
        0x8 => Continue,
        0x9 => Continue,
        0xA => Continue,
        0xB => Continue,
        0xC => Continue,
        0xD => Continue,
        0xE => Continue,
        0xF => Continue,
        _ => Continue,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn jazz() {
        assert!(false);
    }
}
