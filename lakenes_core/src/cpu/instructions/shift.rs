use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn asl(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    match opcode {
        0x0A => {
            cpu.cycles += 2;
            let carry = cpu.a & 0x80 != 0;
            cpu.a <<= 1;
            cpu.set_flag(FLAG_C, carry);
            cpu.update_zero_negative(cpu.a);
        }
        _ => {
            let (addr, value) = match opcode {
                0x06 => {
                    cpu.cycles += 5;
                    let addr = get_address_zeropage(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x16 => {
                    cpu.cycles += 6;
                    let addr = get_address_zeropage_x(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x0E => {
                    cpu.cycles += 6;
                    let addr = get_address_absolute(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x1E => {
                    cpu.cycles += 7;
                    let addr = get_address_absolute_x(cpu, bus).0;
                    (addr, bus.read(addr))
                }
                _ => unreachable!(),
            };

            let carry = value & 0x80 != 0;
            let result = value << 1;
            bus.write(addr, result);
            cpu.set_flag(FLAG_C, carry);
            cpu.update_zero_negative(result);
        }
    }
}

pub fn lsr(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
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

pub fn rol(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let old_carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
    match opcode {
        0x2A => {
            cpu.cycles += 2;
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

            let new_carry = value & 0x80 != 0;
            let result = (value << 1) | old_carry;
            bus.write(addr, result);
            cpu.set_flag(FLAG_C, new_carry);
            cpu.update_zero_negative(result);
        }
    }
}

pub fn ror(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let old_carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
    match opcode {
        0x6A => {
            cpu.cycles += 2;
            let new_carry = cpu.a & 0x01 != 0;
            cpu.a = (cpu.a >> 1) | (old_carry << 7);
            cpu.set_flag(FLAG_C, new_carry);
            cpu.update_zero_negative(cpu.a);
        }
        _ => {
            let (addr, value) = match opcode {
                0x66 => {
                    cpu.cycles += 5;
                    let addr = get_address_zeropage(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x76 => {
                    cpu.cycles += 6;
                    let addr = get_address_zeropage_x(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x6E => {
                    cpu.cycles += 6;
                    let addr = get_address_absolute(cpu, bus);
                    (addr, bus.read(addr))
                }
                0x7E => {
                    cpu.cycles += 7;
                    let addr = get_address_absolute_x(cpu, bus).0;
                    (addr, bus.read(addr))
                }
                _ => unreachable!(),
            };

            let new_carry = value & 0x01 != 0;
            let result = (value >> 1) | (old_carry << 7);
            bus.write(addr, result);
            cpu.set_flag(FLAG_C, new_carry);
            cpu.update_zero_negative(result);
        }
    }
}
