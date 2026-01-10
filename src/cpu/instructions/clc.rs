use crate::cpu::CPU;

const FLAG_C: u8 = 0b00000001;

pub fn execute(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.set_flag(FLAG_C, false);
}
