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
