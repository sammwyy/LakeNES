use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.x = cpu.sp;
    cpu.update_zero_negative(cpu.x);
}
