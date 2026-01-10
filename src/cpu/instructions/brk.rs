use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 7;

    let ret_addr = cpu.pc.wrapping_add(1);
    cpu.push_word(bus, ret_addr);
    cpu.push_byte(bus, cpu.p | 0b00110000);

    let lo = bus.read(0xFFFE) as u16;
    let hi = bus.read(0xFFFF) as u16;
    cpu.pc = (hi << 8) | lo;
}
