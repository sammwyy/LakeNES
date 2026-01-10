use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    cpu.pc = cpu.pop_word(bus).wrapping_add(1);
}
