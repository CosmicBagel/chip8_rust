use crate::emulator::{Emulator, InstructionResult};

#[test]
fn jump_test() {
    // tests 0x1NNN
    let mut emu = Emulator::new_headless();
    let result = emu.execute_instruction(0x1200.into());

    assert!(result == InstructionResult::Working);
    assert!(emu.program_counter == 0x200);

    let result = emu.execute_instruction(0x1500.into());
    assert!(result == InstructionResult::Working);
    assert!(emu.program_counter == 0x500);
}

#[test]
fn subroutine_test() {
    // tests 0x2NNN and 0x00EE
    let mut emu = Emulator::new_headless();
    let result = emu.execute_instruction(0x2500.into());

    assert!(result == InstructionResult::Working);
    assert!(emu.program_counter == 0x500);
    assert!(emu.subroutine_return_pointers.len() == 1);
    assert!(emu.subroutine_return_pointers[0] == 0x202);

    let result = emu.execute_instruction(0x00EE.into());

    assert!(result == InstructionResult::Working);
    assert!(emu.program_counter == 0x202);
    assert!(emu.subroutine_return_pointers.is_empty());

    let mut emu = Emulator::new_headless();

    emu.execute_instruction(0x2100.into());
    emu.execute_instruction(0x2150.into());
    emu.execute_instruction(0x2200.into());
    emu.execute_instruction(0x2250.into());
    emu.execute_instruction(0x2300.into());
    emu.execute_instruction(0x2350.into());
    emu.execute_instruction(0x2400.into());
    emu.execute_instruction(0x2450.into());
    emu.execute_instruction(0x2500.into());
    emu.execute_instruction(0x2550.into());

    assert!(emu.program_counter == 0x550);
    assert!(emu.subroutine_return_pointers.len() == 10);

    for _ in 0..10 {
        emu.execute_instruction(0x00EE.into());
    }

    assert!(emu.program_counter == 0x202);
    assert!(emu.subroutine_return_pointers.is_empty());
}

#[test]
fn the_3xnn_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[2] = 0xFF;
    emu.execute_instruction(0x3215.into());

    assert!(emu.program_counter == 0x202);

    let mut emu = Emulator::new_headless();
    emu.registers[2] = 0x15;
    emu.execute_instruction(0x3215.into());

    assert!(emu.program_counter == 0x204);
}

#[test]
fn the_4xnn_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[2] = 0xFF;
    emu.execute_instruction(0x4215.into());

    assert!(emu.program_counter == 0x204);

    let mut emu = Emulator::new_headless();
    emu.registers[2] = 0x15;
    emu.execute_instruction(0x4215.into());

    assert!(emu.program_counter == 0x202);
}

#[test]
fn the_5xy0_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[1] = 0x00;
    emu.registers[2] = 0xFF;
    emu.execute_instruction(0x5120.into());

    assert!(emu.program_counter == 0x202);

    let mut emu = Emulator::new_headless();
    emu.registers[1] = 0x15;
    emu.registers[2] = 0x15;
    emu.execute_instruction(0x5120.into());

    assert!(emu.program_counter == 0x204);
}

#[test]
fn the_6xnn_test() {
    let mut emu = Emulator::new_headless();
    emu.execute_instruction(0x60FF.into());
    assert!(emu.registers[0] == 0xFF);
}

#[test]
fn the_7xnn_test() {
    let mut emu = Emulator::new_headless();

    emu.execute_instruction(0x7001.into());
    assert!(emu.registers[0] == 0x1);

    emu.execute_instruction(0x70FF.into());
    assert!(emu.registers[0] == 0x0);
}

#[test]
fn the_8xy0_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0xFF;

    emu.execute_instruction(0x8100.into());
    assert!(emu.registers[1] == 0xFF);
}

#[test]
fn the_8xy1_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x1E;
    emu.registers[1] = 0xF0;

    emu.execute_instruction(0x8101.into());
    assert!(emu.registers[1] == 0xFE);
}

#[test]
fn the_8xy2_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x1E;
    emu.registers[1] = 0xF0;

    emu.execute_instruction(0x8102.into());
    assert!(emu.registers[1] == 0x10);
}

#[test]
fn the_8xy3_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x1E;
    emu.registers[1] = 0xF0;

    emu.execute_instruction(0x8103.into());
    assert!(emu.registers[1] == 0xEE);
}

#[test]
fn the_8xy4_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x00;
    emu.registers[1] = 0x01;

    // x_reg_plus_y_reg
    emu.execute_instruction(0x8014.into());
    assert!(emu.registers[0] == 0x01);
    assert!(emu.registers[0xF] == 0x00);

    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x01;
    emu.registers[1] = 0xFF;

    // x_reg_plus_y_reg
    emu.execute_instruction(0x8014.into());
    assert!(emu.registers[0] == 0x00);
    assert!(emu.registers[0xF] == 0x01);
}

#[test]
fn the_8xy5_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x03;
    emu.registers[1] = 0x01;

    // x_reg_plus_y_reg
    emu.execute_instruction(0x8015.into());
    assert!(emu.registers[0] == 0x02);
    assert!(emu.registers[0xF] == 0x01);

    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x00;
    emu.registers[1] = 0x01;

    // x_reg_plus_y_reg
    emu.execute_instruction(0x8015.into());
    assert!(emu.registers[0] == 0xFF);
    assert!(emu.registers[0xF] == 0x00);
}

#[test]
fn the_8xy6_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0xFF;
    emu.registers[1] = 0x0;

    emu.execute_instruction(0x8016.into());
    assert!(emu.registers[0] == 0x7F);
    assert!(emu.registers[0xF] == 1);
    assert!(emu.registers[1] == 0x0);

    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0xFE;
    emu.registers[1] = 0x0;

    emu.execute_instruction(0x8016.into());
    assert!(emu.registers[0] == 0x7F);
    assert!(emu.registers[0xF] == 0);
    assert!(emu.registers[1] == 0x0);
}

#[test]
fn the_8xy7_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x03;
    emu.registers[1] = 0x01;

    // x_reg_plus_y_reg
    emu.execute_instruction(0x8107.into());
    assert!(emu.registers[1] == 0x02);
    assert!(emu.registers[0xF] == 0x01);

    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x00;
    emu.registers[1] = 0x01;

    // x_reg_plus_y_reg
    emu.execute_instruction(0x8107.into());
    assert!(emu.registers[1] == 0xFF);
    assert!(emu.registers[0xF] == 0x00);
}

#[test]
fn the_8xye_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0xFF;
    emu.registers[1] = 0x0;

    emu.execute_instruction(0x801E.into());
    assert!(emu.registers[0] == 0xFE);
    assert!(emu.registers[0xF] == 1);
    assert!(emu.registers[1] == 0x0);

    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x7F;
    emu.registers[1] = 0x0;

    emu.execute_instruction(0x801E.into());
    assert!(emu.registers[0] == 0xFE);
    assert!(emu.registers[0xF] == 0);
    assert!(emu.registers[1] == 0x0);
}

#[test]
fn the_9xy0_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0xFF;
    emu.registers[1] = 0x0;

    emu.execute_instruction(0x9010.into());
    assert!(emu.program_counter == 0x204);

    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0xFF;
    emu.registers[1] = 0xFF;

    emu.execute_instruction(0x9010.into());
    assert!(emu.program_counter == 0x202);
}

#[test]
fn the_annn_test() {
    let mut emu = Emulator::new_headless();
    assert!(emu.address_register == 0x0);
    emu.execute_instruction(0xA250.into());
    assert!(emu.address_register == 0x250);
}

#[test]
fn the_bnnn_test() {
    let mut emu = Emulator::new_headless();
    assert!(emu.program_counter == 0x200);
    emu.registers[0] = 0x0;

    emu.execute_instruction(0xB250.into());
    assert!(emu.program_counter == 0x250);

    let mut emu = Emulator::new_headless();
    assert!(emu.program_counter == 0x200);
    emu.registers[0] = 0x3;

    emu.execute_instruction(0xB250.into());
    assert!(emu.program_counter == 0x253);
}

#[test]
fn the_cxnn_test() {
    //test rng (can sometimes fail)
    let mut emu = Emulator::new_headless();
    assert!(emu.registers[0] == 0x0);
    emu.execute_instruction(0xC0FF.into());
    assert!(emu.registers[0] != 0);

    //test mask
    let mut emu = Emulator::new_headless();
    assert!(emu.registers[0] == 0x0);
    emu.execute_instruction(0xC0AA.into());
    assert!(emu.registers[0] & 0x55 == 0);
}

#[test]
fn the_fx07_test() {
    let mut emu = Emulator::new_headless();
    emu.timer_counter = 30;

    emu.registers[3] = 0;
    emu.execute_instruction(0xF307.into());
    assert!(emu.registers[3] == 30);
}

#[test]
fn the_f315_test() {
    let mut emu = Emulator::new_headless();
    assert!(emu.timer_counter == 0);

    emu.registers[3] = 30;
    emu.execute_instruction(0xF315.into());
    assert!(emu.timer_counter == 30);
}

#[test]
fn the_fx18_test() {
    let mut emu = Emulator::new_headless();
    assert!(emu.sound_counter == 0);

    emu.registers[3] = 30;
    emu.execute_instruction(0xF318.into());
    assert!(emu.sound_counter == 30);
}

#[test]
fn the_fx1e_test() {
    // typical add
    let mut emu = Emulator::new_headless();
    assert!(emu.address_register == 0);
    emu.registers[5] = 0xFF;
    emu.execute_instruction(0xF51E.into());
    assert!(emu.address_register == 0xFF);

    //overflowing add
    let mut emu = Emulator::new_headless();
    emu.address_register = 0xFFFF;
    emu.registers[5] = 0x1;
    emu.execute_instruction(0xF51E.into());
    assert!(emu.address_register == 0x0);
}

#[test]
fn the_fx33_test() {
    let mut emu = Emulator::new_headless();
    emu.address_register = 0x205;
    emu.registers[3] = 223;
    emu.execute_instruction(0xF333.into());
    assert!(emu.memory_space[0x205] == 2);
    assert!(emu.memory_space[0x206] == 2);
    assert!(emu.memory_space[0x207] == 3);

    let mut emu = Emulator::new_headless();
    emu.address_register = 0x205;
    emu.registers[3] = 13;
    emu.execute_instruction(0xF333.into());
    assert!(emu.memory_space[0x205] == 0);
    assert!(emu.memory_space[0x206] == 1);
    assert!(emu.memory_space[0x207] == 3);

    let mut emu = Emulator::new_headless();
    emu.address_register = 0x205;
    emu.registers[3] = 5;
    emu.execute_instruction(0xF333.into());
    assert!(emu.memory_space[0x205] == 0);
    assert!(emu.memory_space[0x206] == 0);
    assert!(emu.memory_space[0x207] == 5);
}

#[test]
fn the_fx55_test() {
    let mut emu = Emulator::new_headless();
    emu.address_register = 0x500;
    for i in 0..0xF {
        emu.registers[i] = i as u8;
    }
    emu.execute_instruction(0xFF55.into());
    for i in 0..0xF {
        assert!(emu.memory_space[0x500 + i] == i as u8);
    }
    assert!(emu.address_register == 0x500);
}

#[test]
fn the_fx65_test() {
    let mut emu = Emulator::new_headless();
    emu.address_register = 0x500;
    for i in 0..0xF {
        emu.memory_space[0x500 + i] = i as u8;
    }
    emu.execute_instruction(0xFF65.into());
    for i in 0..0xF {
        assert!(emu.registers[i] == i as u8);
    }
    assert!(emu.address_register == 0x500);
}
