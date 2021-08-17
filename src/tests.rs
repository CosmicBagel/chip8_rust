use super::*;

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
fn the_0x3XNN_test() {
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
fn the_0x4XNN_test() {
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
fn the_0x5XY0_test() {
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
fn the_0x6XNN_test() {
    let mut emu = Emulator::new_headless();
    emu.execute_instruction(0x60FF.into());
    assert!(emu.registers[0] == 0xFF);
}

#[test]
fn the_0x7XNN_test() {
    let mut emu = Emulator::new_headless();

    emu.execute_instruction(0x7001.into());
    assert!(emu.registers[0] == 0x1);

    emu.execute_instruction(0x70FF.into());
    assert!(emu.registers[0] == 0x0);
}

#[test]
fn the_0x8XY0_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0xFF;

    emu.execute_instruction(0x8100.into());
    assert!(emu.registers[1] == 0xFF);
}

#[test]
fn the_0x8XY1_test() {
    let mut emu = Emulator::new_headless();
    emu.registers[0] = 0x0E;
    emu.registers[1] = 0xF0;

    emu.execute_instruction(0x8101.into());
    assert!(emu.registers[1] == 0xFE);
}
