use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.y = cpu.y.wrapping_add(1);
    cpu.update_zero_negative(cpu.y);
}
