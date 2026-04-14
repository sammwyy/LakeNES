use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn brk(cpu: &mut CPU, bus: &mut Bus) {
    cpu.pc += 1;
    cpu.push_word(bus, cpu.pc);
    cpu.push_byte(bus, cpu.p | FLAG_B | FLAG_U);
    cpu.set_flag(FLAG_I, true);

    let lo = bus.read(0xFFFE) as u16;
    let hi = bus.read(0xFFFF) as u16;
    cpu.pc = (hi << 8) | lo;
    cpu.cycles += 7;
}

pub fn jmp(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    match opcode {
        0x4C => {
            cpu.cycles += 3;
            cpu.pc = cpu.fetch_word(bus);
        }
        0x6C => {
            cpu.cycles += 5;
            let ptr = cpu.fetch_word(bus);
            let lo = bus.read(ptr) as u16;
            // Page wrap bug in 6502
            let hi_addr = (ptr & 0xFF00) | (ptr.wrapping_add(1) & 0x00FF);
            let hi = bus.read(hi_addr) as u16;
            cpu.pc = (hi << 8) | lo;
        }
        _ => unreachable!(),
    }
}

pub fn jsr(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    let target = cpu.fetch_word(bus);
    cpu.push_word(bus, cpu.pc - 1);
    cpu.pc = target;
}

pub fn rti(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    cpu.p = (cpu.pop_byte(bus) & !FLAG_B) | FLAG_U;
    cpu.pc = cpu.pop_word(bus);
}

pub fn rts(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    cpu.pc = cpu.pop_word(bus) + 1;
}

pub fn nop(cpu: &mut CPU) {
    cpu.cycles += 2;
}
