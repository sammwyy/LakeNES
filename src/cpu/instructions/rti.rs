use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    cpu.p = cpu.pop_byte(bus) & 0b11001111 | 0b00100000;
    cpu.pc = cpu.pop_word(bus);
}
