use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

const FLAG_C: u8 = 0b00000001;
const FLAG_Z: u8 = 0b00000010;
const FLAG_N: u8 = 0b10000000;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
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

    let result = cpu.y.wrapping_sub(value);
    cpu.set_flag(FLAG_C, cpu.y >= value);
    cpu.set_flag(FLAG_Z, cpu.y == value);
    cpu.set_flag(FLAG_N, result & 0x80 != 0);
}
