use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    match opcode {
        0x4C => {
            cpu.cycles += 3;
            cpu.pc = cpu.fetch_word(bus);
        }
        0x6C => {
            cpu.cycles += 5;
            let ptr = cpu.fetch_word(bus);
            let lo = bus.read(ptr) as u16;
            let hi = if ptr & 0x00FF == 0x00FF {
                bus.read(ptr & 0xFF00) as u16
            } else {
                bus.read(ptr + 1) as u16
            };
            cpu.pc = (hi << 8) | lo;
        }
        _ => unreachable!(),
    }
}
