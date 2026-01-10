use crate::bus::Bus;
use crate::cpu::CPU;

const FLAG_C: u8 = 0b00000001;

pub fn execute(cpu: &mut CPU, bus: &mut Bus) {
    let offset = cpu.fetch_byte(bus) as i8;
    if !cpu.get_flag(FLAG_C) {
        cpu.cycles += 3;
        let old_pc = cpu.pc;
        cpu.pc = cpu.pc.wrapping_add(offset as u16);
        if (old_pc & 0xFF00) != (cpu.pc & 0xFF00) {
            cpu.cycles += 1;
        }
    } else {
        cpu.cycles += 2;
    }
}
