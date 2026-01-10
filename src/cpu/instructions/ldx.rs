use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xA2 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xA6 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read(addr)
        }
        0xB6 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_y(cpu, bus);
            bus.read(addr)
        }
        0xAE => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read(addr)
        }
        0xBE => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read(addr)
        }
        _ => unreachable!(),
    };
    cpu.x = value;
    cpu.update_zero_negative(value);
}
