use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let addr = match opcode {
        0x85 => {
            cpu.cycles += 3;
            get_address_zeropage(cpu, bus)
        }
        0x95 => {
            cpu.cycles += 4;
            get_address_zeropage_x(cpu, bus)
        }
        0x8D => {
            cpu.cycles += 4;
            get_address_absolute(cpu, bus)
        }
        0x9D => {
            cpu.cycles += 5;
            get_address_absolute_x(cpu, bus).0
        }
        0x99 => {
            cpu.cycles += 5;
            get_address_absolute_y(cpu, bus).0
        }
        0x81 => {
            cpu.cycles += 6;
            get_address_indirect_x(cpu, bus)
        }
        0x91 => {
            cpu.cycles += 6;
            get_address_indirect_y(cpu, bus).0
        }
        _ => unreachable!(),
    };

    bus.write(addr, cpu.a);
}
