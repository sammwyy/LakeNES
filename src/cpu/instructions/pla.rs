use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    cpu.a = cpu.pop_byte(bus);
    cpu.update_zero_negative(cpu.a);
}
