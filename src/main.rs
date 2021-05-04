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

fn main() {
    let mut registers = [0_u8; 16];
    let mut address_register = 0_u16;

    // based on 4kb variant (hence 3215 bytes)
    // all memory accesses will be in big endian
    // chip 8 is big endian!!!
    // halt on bad memory access?
    let mut memory_space = [0_u8; 3215];

    // planning on making these count down on another thread
    // use a condition variable to wake the counting thread on counter set
    // just use thread sleep to try and get the right 60hz freq
    let mut timer_counter = 0_u8;
    let mut sound_counter = 0_u8;

    println!("Hello, world!");
}
