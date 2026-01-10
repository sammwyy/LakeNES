use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xA9 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xA5 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read(addr)
        }
        0xB5 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read(addr)
        }
        0xAD => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read(addr)
        }
        0xBD => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read(addr)
        }
        0xB9 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read(addr)
        }
        0xA1 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read(addr)
        }
        0xB1 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read(addr)
        }
        _ => unreachable!(),
    };

    cpu.a = value;
    cpu.update_zero_negative(value);
}
