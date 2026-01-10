use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.x = cpu.a;
    cpu.update_zero_negative(cpu.x);
}
