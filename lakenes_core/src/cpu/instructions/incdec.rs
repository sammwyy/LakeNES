use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn dec(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let (addr, value) = match opcode {
        0xC6 => {
            cpu.cycles += 5;
            let addr = get_address_zeropage(cpu, bus);
            (addr, bus.read(addr))
        }
        0xD6 => {
            cpu.cycles += 6;
            let addr = get_address_zeropage_x(cpu, bus);
            (addr, bus.read(addr))
        }
        0xCE => {
            cpu.cycles += 6;
            let addr = get_address_absolute(cpu, bus);
            (addr, bus.read(addr))
        }
        0xDE => {
            cpu.cycles += 7;
            let addr = get_address_absolute_x_write(cpu, bus);
            (addr, bus.read(addr))
        }
        _ => unreachable!(),
    };

    let result = value.wrapping_sub(1);
    super::rmw_store(bus, addr, value, result);
    cpu.update_zero_negative(result);
}

pub fn dex(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.x = cpu.x.wrapping_sub(1);
    cpu.update_zero_negative(cpu.x);
}

pub fn dey(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.y = cpu.y.wrapping_sub(1);
    cpu.update_zero_negative(cpu.y);
}

pub fn inc(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let (addr, value) = match opcode {
        0xE6 => {
            cpu.cycles += 5;
            let addr = get_address_zeropage(cpu, bus);
            (addr, bus.read(addr))
        }
        0xF6 => {
            cpu.cycles += 6;
            let addr = get_address_zeropage_x(cpu, bus);
            (addr, bus.read(addr))
        }
        0xEE => {
            cpu.cycles += 6;
            let addr = get_address_absolute(cpu, bus);
            (addr, bus.read(addr))
        }
        0xFE => {
            cpu.cycles += 7;
            let addr = get_address_absolute_x_write(cpu, bus);
            (addr, bus.read(addr))
        }
        _ => unreachable!(),
    };

    let result = value.wrapping_add(1);
    super::rmw_store(bus, addr, value, result);
    cpu.update_zero_negative(result);
}

pub fn inx(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.x = cpu.x.wrapping_add(1);
    cpu.update_zero_negative(cpu.x);
}

pub fn iny(cpu: &mut CPU) {
    cpu.cycles += 2;
    cpu.y = cpu.y.wrapping_add(1);
    cpu.update_zero_negative(cpu.y);
}
