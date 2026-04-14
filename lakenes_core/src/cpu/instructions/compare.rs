use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

fn compare(cpu: &mut CPU, val1: u8, val2: u8) {
    cpu.set_flag(FLAG_C, val1 >= val2);
    cpu.update_zero_negative(val1.wrapping_sub(val2));
}

pub fn cmp(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
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
    compare(cpu, cpu.a, value);
}

pub fn cpx(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xE0 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xE4 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read(addr)
        }
        0xEC => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read(addr)
        }
        _ => unreachable!(),
    };
    compare(cpu, cpu.x, value);
}

pub fn cpy(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xC0 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xC4 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read(addr)
        }
        0xCC => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read(addr)
        }
        _ => unreachable!(),
    };
    compare(cpu, cpu.y, value);
}
