use pixels::Pixels;
use rand::random;
use std::fs::File;
use std::io::prelude::*;
use winit::dpi::PhysicalSize;

//refactor todo list
// todo newtypes for address and registers and maybe program counter
// todo timer_counter decremented on side thread dedicated to just decrementing it at regular
//      interval (we'll just use arc and an atomic integer)

// bug we're not loading in numbers as big endian, not sure how this will affect things,
//      will probably mostly just affect addresses as they're the only number that exceeds 1 byte
//      may also affect saving registries to memory and vice versa
//      might affect drawing, but again, all drawing is within 1 byte :thonking face:
//      this will likely have to be implemented for the sake of the I register
//      a final (serious) implication as that the instructions being loaded in all have their
//      bytes swapped (not sure about this one, but would be hilarious) eg 0xaabb should actually
//      be processed as 0xbbaa

// 4kb memory, 512bytes reserved for system
// 4096 - 512 = 3584 max bytes for apps
const MAX_MEMORY: usize = 4096;
const MAX_STACK: usize = 12;

// black is the default window fill
// white is the default pixel fill while initializing
// https://coolors.co/89b6a5-4c3b4d-c9eddc-82968c-6a706e
// https://coolors.co/14110f-34312d-7e7f83-d9c5b2-f3f3f4
const SET_COLOUR: [u8; 4] = [0xF3, 0xF3, 0xF4, 0xFF];
const UNSET_COLOUR: [u8; 4] = [0x14, 0x11, 0x0F, 0xFF];

//built-in hex sprites, taken from http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.5
const BUILTIN_SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // zero
    0x20, 0x60, 0x20, 0x20, 0x70, // one
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // two
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // three
    0x90, 0x90, 0xF0, 0x10, 0x10, // four
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // five
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // six
    0xF0, 0x10, 0x20, 0x40, 0x40, // seven
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // eight
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // nine
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Copy, Clone)]
struct Opcode {
    full_opcode: u16,
    left_byte: u8,
    right_byte: u8,
    fourth_nibble: u8,
    third_nibble: u8,
    second_nibble: u8,
    first_nibble: u8,
}
enum OpcodeResult {
    Continue,
    Terminate,
    Jump(u16),
    SkipNext,
    Malformed,
    RequestRedraw,
}

#[derive(PartialEq)]
pub enum CycleResult {
    Working,
    RedrawRequested,
    Terminated,
}

pub struct Emulator {
    registers: [u8; 16],
    // although we need usize to access the array that the instructions are stored in
    // its better to explicitly say u16 as usize can technically be as small as u8
    address_register: u16,
    memory_space: [u8; MAX_MEMORY],
    timer_counter: u8,
    sound_counter: u8,
    program_counter: u16,
    subroutine_return_pointers: Vec<u16>,
    pixels_frame_buffer: Pixels,
    end_loop_reached: bool,
}

impl Emulator {
    pub fn new(p: Pixels) -> Emulator {
        let mut emu = Emulator {
            registers: [0_u8; 16],
            address_register: 0_u16,
            memory_space: [0_u8; MAX_MEMORY],
            timer_counter: 0_u8,
            sound_counter: 0_u8,
            program_counter: 0x200_u16,
            subroutine_return_pointers: vec![0_u16; MAX_STACK],
            pixels_frame_buffer: p,
            end_loop_reached: false,
        };
        // fill first 80 bytes of memory with out built-in hex digit sprites
        emu.memory_space[..80].copy_from_slice(&BUILTIN_SPRITES);
        emu.clear_screen();
        emu
    }

    pub fn load_program(&mut self, file_name: &str) -> usize {
        let mut rom_file = File::open(file_name).unwrap();
        rom_file.read(&mut self.memory_space[0x200..]).unwrap()
    }

    pub fn pixels_render(&mut self) {
        self.pixels_frame_buffer.render().unwrap()
    }

    pub fn pixels_surface_resize(&mut self, size: PhysicalSize<u32>) {
        self.pixels_frame_buffer
            .resize_surface(size.width, size.height);
    }

    pub fn execute_cycle(&mut self) -> CycleResult {
        // since we're sleep for 16 ms per cycle, this will very roughly approximate 60hz
        if self.timer_counter > 0 {
            self.timer_counter -= 1;
        }
        if self.sound_counter > 0 {
            self.sound_counter -= 1;
        }

        let opcode = self.load_opcode();
        let opcode_result = self.process_opcode(opcode);

        match opcode_result {
            OpcodeResult::Terminate => {
                println!("Terminating");
                return CycleResult::Terminated;
            }
            OpcodeResult::Jump(target) => {
                if target < self.memory_space.len() as u16 - 1 {
                    self.program_counter = target;
                } else {
                    panic!("Jump instruction attempted to access out of bounds memory");
                }
            }
            OpcodeResult::SkipNext => {
                if self.program_counter + 4 < self.memory_space.len() as u16 - 1 {
                    self.program_counter += 4;
                } else {
                    panic!("Skip instruction attempted to access out of bounds memory");
                }
            }
            OpcodeResult::Malformed => {
                panic!(
                    "Malformed opcode 0x{:#06x}{:#06x}",
                    opcode.left_byte, opcode.right_byte
                );
            }
            OpcodeResult::Continue => {
                if self.program_counter + 2 < self.memory_space.len() as u16 - 1 {
                    self.program_counter += 2;
                } else {
                    panic!("Program counter exceeded memory bounds");
                }
            }
            OpcodeResult::RequestRedraw => {
                if self.program_counter + 2 < self.memory_space.len() as u16 - 1 {
                    self.program_counter += 2;
                } else {
                    panic!("Program counter exceeded memory bounds");
                }
                return CycleResult::RedrawRequested;
            }
        }
        CycleResult::Working
    }

    fn load_opcode(&self) -> Opcode {
        let translated_address = self.program_counter as usize;
        let mut opcode = Opcode {
            full_opcode: 0,
            left_byte: self.memory_space[translated_address],
            right_byte: self.memory_space[translated_address as usize + 1],
            fourth_nibble: 0,
            third_nibble: 0,
            second_nibble: 0,
            first_nibble: 0,
        };

        opcode.fourth_nibble = (0xF0 & opcode.left_byte) >> 4;
        opcode.third_nibble = 0x0F & opcode.left_byte;
        opcode.second_nibble = (0xF0 & opcode.right_byte) >> 4;
        opcode.first_nibble = 0x0F & opcode.right_byte;

        opcode.full_opcode = (opcode.left_byte as u16) << 8;
        opcode.full_opcode |= opcode.right_byte as u16;

        // print!("{:02x?}{:02x?}, ", opcode.left_byte, opcode.right_byte,);

        opcode
    }

    // returns desired program counter location
    fn process_opcode(&mut self, opcode: Opcode) -> OpcodeResult {
        // NNN refers to 0x0NNN parts of the opcode being processed

        // https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set
        // This gives an explanation of the instruction set implemented here

        match opcode.fourth_nibble {
            0x0 => {
                match opcode.full_opcode {
                    0x00E0 => self.clear_screen(),
                    0x00EE => self.return_from_subroutine(),
                    0x0000 => {
                        // 0x0000 EOF
                        OpcodeResult::Terminate
                    }
                    _ => {
                        // 0x0NNN Execute machine language subroutine at address NNN
                        // This emulator will not support machine language subroutines!!!
                        OpcodeResult::Continue
                    }
                }
            }
            0x1 => self.jump(opcode),
            0x2 => self.call_subroutine(opcode),
            0x3 => self.skip_next_if_x_reg_equal(opcode),
            0x4 => self.skip_next_if_x_reg_not_equal(opcode),
            0x5 => self.skip_next_if_regs_equal(opcode),
            0x6 => self.x_reg_store_value(opcode),
            0x7 => self.x_reg_add_value(opcode),
            0x8 => match opcode.first_nibble {
                0x0 => self.y_reg_copy_to_x_reg(opcode),
                0x1 => self.x_reg_or_y_reg(opcode),
                0x2 => self.x_reg_and_y_reg(opcode),
                0x3 => self.x_reg_xor_reg_y(opcode),
                0x4 => self.x_reg_plus_y_reg(opcode),
                0x5 => self.x_reg_minus_y_reg(opcode),
                0x6 => self.shift_register_right(opcode),
                0x7 => self.y_reg_minus_x_reg(opcode),
                0xE => self.shift_register_left(opcode),
                _ => OpcodeResult::Malformed,
            },
            0x9 => self.skip_next_if_not_equal(opcode),
            0xA => self.store_address(opcode),
            0xB => self.jump_with_offset(opcode),
            0xC => self.generate_rnd_num(opcode),
            0xD => self.draw_sprite(opcode),
            0xE => self.skip_next_if_key_is_down(opcode),
            0xF => match opcode.right_byte {
                0x07 => self.load_delay_counter_value(opcode),
                0x0A => self.wait_for_key_and_store(opcode),
                0x15 => self.set_delay_counter(opcode),
                0x18 => self.set_sound_counter(opcode),
                0x1E => self.add_to_address_reg(opcode),
                0x29 => self.lookup_sprite_for_digit(opcode),
                0x33 => self.store_bcd_at_address(opcode),
                0x55 => self.store_registers_to_address(opcode),
                0x65 => self.load_registers_from_address(opcode),

                _ => OpcodeResult::Malformed,
            },
            _ => OpcodeResult::Malformed,
        }
    }

    fn return_from_subroutine(&mut self) -> OpcodeResult {
        // 0x00EE Return from a subroutine
        let return_address = self.subroutine_return_pointers.pop().unwrap_or_else(|| {
            println!("could not return from subroutine, no return pointers");
            0_u16
        });
        if return_address != 0 {
            OpcodeResult::Jump(return_address)
        } else {
            OpcodeResult::Malformed
        }
    }

    fn call_subroutine(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0x2NNN Execute subroutine starting at address NNN
        // +2 so that we don't loop on return
        self.subroutine_return_pointers
            .push(self.program_counter + 2);
        OpcodeResult::Jump(opcode.full_opcode & 0x0FFF)
    }

    fn skip_next_if_x_reg_equal(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0x3XNN Skip the following instruction if the value of register VX equals NN
        if self.registers[opcode.third_nibble as usize] == opcode.right_byte {
            OpcodeResult::SkipNext
        } else {
            OpcodeResult::Continue
        }
    }

    fn skip_next_if_x_reg_not_equal(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0x4XNN Skip the following instruction if the value of register VX is NOT equal to NN
        if self.registers[opcode.third_nibble as usize] != opcode.right_byte {
            OpcodeResult::SkipNext
        } else {
            OpcodeResult::Continue
        }
    }

    fn skip_next_if_regs_equal(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0x5XY0 Skip the following instruction if the value of register VX is equal to the
        // value of register VY
        if self.registers[opcode.third_nibble as usize]
            == self.registers[opcode.second_nibble as usize]
        {
            OpcodeResult::SkipNext
        } else {
            OpcodeResult::Continue
        }
    }

    fn x_reg_store_value(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x6XNN store number NN in register VX
        self.registers[opcode.third_nibble as usize] = opcode.right_byte;
        OpcodeResult::Continue
    }

    fn x_reg_add_value(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x7XNN Add the value NN to register VX
        let result =
            self.registers[opcode.third_nibble as usize].overflowing_add(opcode.right_byte);
        self.registers[opcode.third_nibble as usize] = result.0;
        OpcodeResult::Continue
    }

    fn y_reg_copy_to_x_reg(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x8XY0 Store the value of register VY in register VX
        self.registers[opcode.third_nibble as usize] =
            self.registers[opcode.second_nibble as usize];
        OpcodeResult::Continue
    }

    fn x_reg_or_y_reg(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x8XY1 Set VX to VX OR VY
        self.registers[opcode.third_nibble as usize] |=
            self.registers[opcode.second_nibble as usize];
        OpcodeResult::Continue
    }

    fn x_reg_and_y_reg(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x8XY2 Set VX to VX AND VY
        self.registers[opcode.third_nibble as usize] &=
            self.registers[opcode.second_nibble as usize];
        OpcodeResult::Continue
    }

    fn x_reg_xor_reg_y(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x8XY3 Set VX to VX XOR VY
        self.registers[opcode.third_nibble as usize] ^=
            self.registers[opcode.second_nibble as usize];
        OpcodeResult::Continue
    }

    fn x_reg_plus_y_reg(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x8XY4 Add the value of register VY to register VX
        // Set VF to 01 if a carry occurs
        // Set VF to 00 if a carry does not occur
        // By "carry" we're talking about OVERFLOW
        let result = self.registers[opcode.third_nibble as usize]
            .overflowing_add(self.registers[opcode.second_nibble as usize]);
        self.registers[opcode.third_nibble as usize] = result.0;
        self.registers[0xF_usize] = result.1 as u8;
        OpcodeResult::Continue
    }

    fn x_reg_minus_y_reg(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0x8XY5 Subtract the value of register VY from register VX
        //Set VF to 00 if a borrow occurs
        //Set VF to 01 if a borrow does not occur
        //By borrow we're talking about UNDERFLOW
        let result = self.registers[opcode.third_nibble as usize]
            .overflowing_sub(self.registers[opcode.second_nibble as usize]);
        self.registers[opcode.third_nibble as usize] = result.0;
        self.registers[0xF_usize] = result.1 as u8;
        OpcodeResult::Continue
    }

    fn shift_register_right(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x8XY6 Store the value of register VY shifted right one bit in register VX
        //Set register VF to the least significant bit prior to the shift
        //VY is unchanged
        let val = self.registers[opcode.second_nibble as usize];
        self.registers[0xF_usize] = val & 0xFE;
        self.registers[opcode.third_nibble as usize] =
            self.registers[opcode.second_nibble as usize] >> 1;
        OpcodeResult::Continue
    }

    fn y_reg_minus_x_reg(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x8XY7 Set register VX to the value of VY minus VX
        //Set VF to 00 if a borrow occurs
        //Set VF to 01 if a borrow does not occur
        let result = self.registers[opcode.second_nibble as usize]
            .overflowing_sub(self.registers[opcode.third_nibble as usize]);
        self.registers[opcode.third_nibble as usize] = result.0;
        self.registers[0xF_usize] = result.1 as u8;
        OpcodeResult::Continue
    }

    fn shift_register_left(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x8XYE Store the value of register VY shifted left one bit in register VX
        //Set register VF to the most significant bit prior to the shift
        //VY is unchanged
        let val = self.registers[opcode.second_nibble as usize];
        self.registers[0xF_usize] = val >> 7;
        self.registers[opcode.third_nibble as usize] =
            self.registers[opcode.second_nibble as usize] << 1;
        OpcodeResult::Continue
    }

    fn skip_next_if_not_equal(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0x9XY0 Skip the following instruction if the value of register VX is NOT equal to the
        // value of register VY
        if self.registers[opcode.third_nibble as usize]
            != self.registers[opcode.second_nibble as usize]
        {
            OpcodeResult::SkipNext
        } else {
            OpcodeResult::Continue
        }
    }

    fn store_address(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0xANNN Store memory address NNN in register I (address register)
        // extract address from opcode
        self.address_register = opcode.full_opcode & 0x0FFF;
        OpcodeResult::Continue
    }

    fn jump_with_offset(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xBNNN Jump to address NNN + V0
        // let mut jump_target = jump_to_opcode_nnn();
        let mut jump_addr = opcode.full_opcode & 0x0FFF;
        jump_addr += self.registers[0] as u16;
        OpcodeResult::Jump(jump_addr)
    }

    fn generate_rnd_num(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0xCXNN Set VX to a random number with a mask of NN
        let mut rand_val: u8 = random();
        rand_val &= opcode.right_byte;
        self.registers[opcode.third_nibble as usize] = rand_val;
        OpcodeResult::Continue
    }

    fn load_delay_counter_value(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xFX07 Store the current value of the delay timer in register VX
        self.registers[opcode.third_nibble as usize] = self.timer_counter;
        OpcodeResult::Continue
    }

    fn set_delay_counter(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xFX15 Set the delay timer to the value of register VX
        self.timer_counter = self.registers[opcode.third_nibble as usize];
        OpcodeResult::Continue
    }

    fn set_sound_counter(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xFX18 Set the sound timer to the value of register VX
        self.sound_counter = self.registers[opcode.third_nibble as usize];
        OpcodeResult::Continue
    }

    fn add_to_address_reg(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xFX1E Add the value stored in register VX to register I

        let result = self
            .address_register
            .overflowing_add(self.registers[opcode.third_nibble as usize] as u16);
        self.address_register = result.0;
        OpcodeResult::Continue
    }

    fn store_bcd_at_address(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xFX33 Store the binary-coded decimal equivalent of the value stored in register
        //VX at addresses I, I + 1, and I + 2

        let mut value = self.registers[opcode.third_nibble as usize];
        let ones = value % 0xA;
        value -= ones;
        let tens = value % 0x64;
        value -= tens;
        let hundreds = value;

        let base_address = self.address_register as usize;
        self.memory_space[base_address] = hundreds;
        self.memory_space[base_address + 1] = tens;
        self.memory_space[base_address + 2] = ones;
        OpcodeResult::Continue
    }

    fn store_registers_to_address(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xFX55 Store the values of registers V0 to VX inclusive in memory starting at
        // address I
        //I is set to I + X + 1 after operation
        for reg_index in 0..(opcode.third_nibble + 1) {
            let write_address = self.address_register + reg_index as u16;
            self.memory_space[write_address as usize] = self.registers[reg_index as usize];
        }
        self.address_register += opcode.third_nibble as u16 + 1;
        OpcodeResult::Continue
    }

    fn load_registers_from_address(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xFX65 Fill registers V0 to VX inclusive with the values stored in memory starting
        // at address I
        //I is set to I + X + 1 after operation
        for reg_index in 0..(opcode.third_nibble + 1) {
            let read_address = self.address_register + reg_index as u16;
            self.registers[reg_index as usize] = self.memory_space[read_address as usize];
        }
        self.address_register += opcode.third_nibble as u16 + 1;
        OpcodeResult::Continue
    }

    fn lookup_sprite_for_digit(&mut self, opcode: Opcode) -> OpcodeResult {
        //0xFX29 Set I to the memory address of the sprite data corresponding to the
        // hexadecimal digit stored in register VX

        // we have our built-in sprites at the start of the system memory,
        // they're in order of the hex values (0-F), and each one is 5 bytes big
        // so we can just use the requested hex sprite as an address multiplier
        self.address_register = self.registers[opcode.third_nibble as usize] as u16 * 5;

        OpcodeResult::Continue
    }

    fn wait_for_key_and_store(&mut self, _opcode: Opcode) -> OpcodeResult {
        //0xFX0A Wait for a keypress and store the result in register VX
        //TODO implement input
        println!("NOT IMPLEMENTED wait for key and store");
        OpcodeResult::Continue
    }

    fn skip_next_if_key_is_down(&mut self, _opcode: Opcode) -> OpcodeResult {
        //TODO implement input
        println!("NOT IMPLEMENTED skip next if key is down");
        // 0xEX9E Skip the following instruction if the key corresponding to the hex value
        // currently stored in register VX is pressed

        // 0xEXA1 Skip the following instruction if the key corresponding to the hex value
        // currently stored in register VX is not pressed
        OpcodeResult::Continue
    }

    fn draw_sprite(&mut self, opcode: Opcode) -> OpcodeResult {
        // 0xDXYN Draw a sprite at position VX, VY with N bytes of sprite data starting at the
        //address stored in I
        //Set VF to 01 if any set pixels are changed to unset, and 00 otherwise

        //a sprite can only be a max of 8 pixels wide, and 15 pixels tall
        //8 bits = 1 byte, the width
        //15 bytes, the height
        //we jump down to the next line of pixels at the end of each byte

        let frame = self.pixels_frame_buffer.get_frame();
        let x_origin = self.registers[opcode.third_nibble as usize] as usize;
        let y_origin = self.registers[opcode.second_nibble as usize] as usize;
        // println!("x: {} y: {}", x_origin, y_origin);
        self.registers[0xF] = 0x0;

        let start = self.address_register as usize;
        let end = start + opcode.first_nibble as usize;
        let sprite_slice = &self.memory_space[start..end];

        for (row, byte) in sprite_slice.iter().enumerate() {
            for bit_index in 0..8u8 {
                let bit = byte & (1 << bit_index);
                let pixel_ind = (x_origin + (7 - bit_index) as usize + ((y_origin + row) * 64)) * 4;
                let p = &mut frame[pixel_ind..pixel_ind + 4];

                if bit != 0 {
                    //set the pixel
                    p.copy_from_slice(&SET_COLOUR);
                } else {
                    //unset the pixel
                    if p == SET_COLOUR {
                        self.registers[0xF] = 0x1;
                        p.copy_from_slice(&UNSET_COLOUR);
                    }
                }
            }
        }

        OpcodeResult::RequestRedraw
    }

    fn jump(&mut self, opcode: Opcode) -> OpcodeResult {
        //0x1NNN Jump to address NNN
        let target = opcode.full_opcode & 0x0FFF;
        if target == self.program_counter && !self.end_loop_reached {
            self.end_loop_reached = true;
            println!("End of program loop reached");
        }
        OpcodeResult::Jump(target)
    }

    fn clear_screen(&mut self) -> OpcodeResult {
        //0x00E0 Clear the screen
        //TODO implement clear screen

        let frame = self.pixels_frame_buffer.get_frame();
        for rgba_chunk in frame.chunks_exact_mut(4) {
            // rgba
            rgba_chunk.copy_from_slice(&UNSET_COLOUR);
        }
        OpcodeResult::RequestRedraw
    }
}
