use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

const FLAG_C: u8 = 0b00000001;
const FLAG_V: u8 = 0b01000000;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0x69 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0x65 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read(addr)
        }
        0x75 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read(addr)
        }
        0x6D => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read(addr)
        }
        0x7D => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read(addr)
        }
        0x79 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read(addr)
        }
        0x61 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read(addr)
        }
        0x71 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read(addr)
        }
        _ => unreachable!(),
    };

    let carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
    let sum = cpu.a as u16 + value as u16 + carry;

    let overflow = (cpu.a ^ sum as u8) & (value ^ sum as u8) & 0x80 != 0;

    cpu.set_flag(FLAG_C, sum > 0xFF);
    cpu.set_flag(FLAG_V, overflow);
    cpu.a = sum as u8;
    cpu.update_zero_negative(cpu.a);
}
