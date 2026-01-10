use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

const FLAG_C: u8 = 0b00000001;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    match opcode {
        0x2A => {
            cpu.cycles += 2;
            let old_carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
            let new_carry = cpu.a & 0x80 != 0;
            cpu.a = (cpu.a << 1) | old_carry;
            cpu.set_flag(FLAG_C, new_carry);
            cpu.update_zero_negative(cpu.a);
        }
        _ => {
            let (addr, value) = match opcode {
                0x26 => {
                    cpu.cycles += 5;
                    let addr = get_address_zeropage(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x36 => {
                    cpu.cycles += 6;
                    let addr = get_address_zeropage_x(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x2E => {
                    cpu.cycles += 6;
                    let addr = get_address_absolute(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x3E => {
                    cpu.cycles += 7;
                    let addr = get_address_absolute_x(cpu, bus).0;
                    (addr, bus.read(addr))
                }
                _ => unreachable!(),
            };

            let old_carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
            let new_carry = value & 0x80 != 0;
            let result = (value << 1) | old_carry;
            bus.write(addr, result);
            cpu.set_flag(FLAG_C, new_carry);
            cpu.update_zero_negative(result);
        }
    }
}
