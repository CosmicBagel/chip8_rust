use super::*;

#[test]
fn jump_test() {
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
