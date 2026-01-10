use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    let addr = cpu.fetch_word(bus);
    let ret_addr = cpu.pc.wrapping_sub(1);
    cpu.push_word(bus, ret_addr);
    cpu.pc = addr;
}
