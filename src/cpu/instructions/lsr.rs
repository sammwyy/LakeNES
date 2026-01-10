use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

const FLAG_C: u8 = 0b00000001;

pub fn execute(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    match opcode {
        0x4A => {
            cpu.cycles += 2;
            let carry = cpu.a & 0x01 != 0;
            cpu.a >>= 1;
            cpu.set_flag(FLAG_C, carry);
            cpu.update_zero_negative(cpu.a);
        }
        _ => {
            let (addr, value) = match opcode {
                0x46 => {
                    cpu.cycles += 5;
                    let addr = get_address_zeropage(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x56 => {
                    cpu.cycles += 6;
                    let addr = get_address_zeropage_x(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x4E => {
                    cpu.cycles += 6;
                    let addr = get_address_absolute(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x5E => {
                    cpu.cycles += 7;
                    let addr = get_address_absolute_x(cpu, bus).0;
                    (addr, bus.read(addr))
                }
                _ => unreachable!(),
            };

            let carry = value & 0x01 != 0;
            let result = value >> 1;
            bus.write(addr, result);
            cpu.set_flag(FLAG_C, carry);
            cpu.update_zero_negative(result);
        }
    }
}
