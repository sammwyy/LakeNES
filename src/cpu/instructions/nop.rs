use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU) {
    cpu.cycles += 2;
}
