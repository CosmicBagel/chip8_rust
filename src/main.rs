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
    address_register: u16,
    memory_space: [u8; MAX_MEMORY],
    timer_counter: u8,
    sound_counter: u8,
    program_counter: usize,
}

fn main() {
    let mut state = EmulatorState {
        registers: [0_u8; 16],
        address_register: 0_u16,
        memory_space: [0_u8; MAX_MEMORY],
        timer_counter: 0_u8,
        sound_counter: 0_u8,
        program_counter: 0_usize,
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
        - just keep executing next instruction in memory (even if we progress into data memory?)
        - will have to implement a stack for subroutines (or at least a return pointer)
        */

        let opcode_left_byte = state.memory_space[state.program_counter];
        let opcode_right_byte = state.memory_space[state.program_counter + 1];

        print!("{:02x?}{:02x?}, ", opcode_left_byte, opcode_right_byte,);

        let program_counter_target = process_opcode(opcode_left_byte, opcode_right_byte);
        // ideas to handle state mutation and opcode
        // a) context bag  with everything that just get borrowed
        // b) decode opcodes, create message enum, keep data in main and just pass
        //    whats needed to specific funcs
        // c)

        // if we have a 0x0000 opcode, terminate
        if opcode_left_byte == 0x00 && opcode_right_byte == 0x00 {
            break;
        }

        if let Some(target) = program_counter_target {
            if target < state.memory_space.len() - 1 {
                state.program_counter = target;
            } else {
                panic!("Attempted to access out of bounds memory");
            }
        } else if state.program_counter + 2 < state.memory_space.len() - 1 {
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
fn process_opcode(opcode_left_byte: u8, _: u8) -> Option<usize> {
    // process opcode
    let left_most_nibble = 0xF0 & opcode_left_byte;
    match left_most_nibble {
        0x00 => None,
        0x10 => None,
        0x20 => None,
        0x30 => None,
        0x40 => None,
        0x50 => None,
        0x60 => None,
        0x70 => None,
        0x80 => None,
        0x90 => None,
        0xA0 => None,
        0xB0 => None,
        0xC0 => None,
        0xD0 => None,
        0xE0 => None,
        0xF0 => None,
        _ => None,
    }
}

fn jump_to_address() {}
