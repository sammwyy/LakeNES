use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

const FLAG_Z: u8 = 0b00000010;
const FLAG_V: u8 = 0b01000000;
const FLAG_N: u8 = 0b10000000;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0x24 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read(addr)
        }
        0x2C => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read(addr)
        }
        _ => unreachable!(),
    };

    let result = cpu.a & value;
    cpu.set_flag(FLAG_Z, result == 0);
    cpu.set_flag(FLAG_V, value & 0x40 != 0);
    cpu.set_flag(FLAG_N, value & 0x80 != 0);
}
