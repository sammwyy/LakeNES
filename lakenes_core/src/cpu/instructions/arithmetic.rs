use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn adc(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0x69 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0x65 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0x75 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0x6D => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        0x7D => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0x79 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0x61 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0x71 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };

    let carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
    let sum = cpu.a as u16 + value as u16 + carry;

    let overflow = (cpu.a ^ sum as u8) & (value ^ sum as u8) & 0x80 != 0;

    cpu.set_flag(FLAG_C, sum > 0xFF);
    cpu.set_flag(FLAG_V, overflow);
    cpu.a = sum as u8;
    cpu.update_zero_negative(cpu.a);
}

pub fn sbc(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xE9 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xE5 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0xF5 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0xED => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        0xFD => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0xF9 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0xE1 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0xF1 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };

    let carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
    // SBC is A - V - (1 - C) = A - V - 1 + C
    // We can use ADC logic with !value
    let val_inv = !value;
    let sum = cpu.a as u16 + val_inv as u16 + carry;

    let overflow = (cpu.a ^ sum as u8) & (val_inv ^ sum as u8) & 0x80 != 0;

    cpu.set_flag(FLAG_C, sum > 0xFF);
    cpu.set_flag(FLAG_V, overflow);
    cpu.a = sum as u8;
    cpu.update_zero_negative(cpu.a);
}
