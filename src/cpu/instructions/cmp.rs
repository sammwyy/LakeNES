use super::*;
use crate::cpu::CPU;

const FLAG_C: u8 = 0b00000001;
const FLAG_Z: u8 = 0b00000010;
const FLAG_N: u8 = 0b10000000;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xC9 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xC5 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read(addr)
        }
        0xD5 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read(addr)
        }
        0xCD => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read(addr)
        }
        0xDD => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read(addr)
        }
        0xD9 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read(addr)
        }
        0xC1 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read(addr)
        }
        0xD1 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read(addr)
        }
        _ => unreachable!(),
    };

    let result = cpu.a.wrapping_sub(value);
    cpu.set_flag(FLAG_C, cpu.a >= value);
    cpu.set_flag(FLAG_Z, cpu.a == value);
    cpu.set_flag(FLAG_N, result & 0x80 != 0);
}
