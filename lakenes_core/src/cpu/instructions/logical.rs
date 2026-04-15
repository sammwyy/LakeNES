use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn and(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0x29 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0x25 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0x35 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0x2D => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        0x3D => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0x39 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0x21 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0x31 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };

    cpu.a &= value;
    cpu.update_zero_negative(cpu.a);
}

pub fn bit(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0x24 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0x2C => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };

    cpu.set_flag(FLAG_Z, (cpu.a & value) == 0);
    cpu.set_flag(FLAG_N, (value & 0x80) != 0);
    cpu.set_flag(FLAG_V, (value & 0x40) != 0);
}

pub fn eor(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0x49 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0x45 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0x55 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0x4D => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        0x5D => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0x59 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0x41 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0x51 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };

    cpu.a ^= value;
    cpu.update_zero_negative(cpu.a);
}

pub fn ora(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0x09 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0x05 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0x15 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0x0D => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        0x1D => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0x19 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0x01 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0x11 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };

    cpu.a |= value;
    cpu.update_zero_negative(cpu.a);
}
