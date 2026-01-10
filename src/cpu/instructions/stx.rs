use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let addr = match opcode {
        0x86 => {
            cpu.cycles += 3;
            get_address_zeropage(cpu, bus)
        }
        0x96 => {
            cpu.cycles += 4;
            get_address_zeropage_y(cpu, bus)
        }
        0x8E => {
            cpu.cycles += 4;
            get_address_absolute(cpu, bus)
        }
        _ => unreachable!(),
    };
    bus.write(addr, cpu.x);
}
