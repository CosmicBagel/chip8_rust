use std::io::{prelude::*, stdout};
use std::thread;
mod emulator;
use emulator::*;

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

fn main() {
    let mut emulator = Emulator {
        registers: [0_u8; 16],
        address_register: 0_u16,
        memory_space: [0_u8; MAX_MEMORY],
        timer_counter: 0_u8,
        sound_counter: 0_u8,
        program_counter: 0x200_u16,
        subroutine_return_pointers: vec![0_u16; MAX_STACK],
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

    let bytes_read = emulator.load_program(DEFAULT_ROM);
    println!("Loaded program, bytes {}", bytes_read);

    // run the code
    loop {
        if emulator.execute_cycle() == CycleResult::Terminated {
            break;
        }

        // so that stdout prints show up when printed
        stdout().flush().unwrap();
        thread::sleep(CYCLE_SLEEP_DURATION);
    }

    println!("End of program");
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn jazz() {
        assert!(false);
    }
}
