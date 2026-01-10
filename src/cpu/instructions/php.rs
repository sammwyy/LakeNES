use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 3;
    cpu.push_byte(bus, cpu.p | 0b00110000);
}
