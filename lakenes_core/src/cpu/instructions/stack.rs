use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn pha(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 3;
    cpu.push_byte(bus, cpu.a);
}

pub fn php(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 3;
    cpu.push_byte(bus, cpu.p | FLAG_B | FLAG_U);
}

pub fn pla(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    cpu.a = cpu.pop_byte(bus);
    cpu.update_zero_negative(cpu.a);
}

pub fn plp(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    cpu.p = (cpu.pop_byte(bus) & !FLAG_B) | FLAG_U;
}
