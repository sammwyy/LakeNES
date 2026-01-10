use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.a = cpu.y;
    cpu.update_zero_negative(cpu.a);
}
