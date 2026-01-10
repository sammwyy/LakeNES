use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

const FLAG_C: u8 = 0b00000001;
const FLAG_Z: u8 = 0b00000010;
const FLAG_N: u8 = 0b10000000;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
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

    let result = cpu.x.wrapping_sub(value);
    cpu.set_flag(FLAG_C, cpu.x >= value);
    cpu.set_flag(FLAG_Z, cpu.x == value);
    cpu.set_flag(FLAG_N, result & 0x80 != 0);
}
