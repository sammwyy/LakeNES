use super::*;
use crate::bus::Bus;
use crate::cpu::CPU;

pub fn lda(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xA9 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xA5 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0xB5 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0xAD => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        0xBD => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0xB9 => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        0xA1 => {
            cpu.cycles += 6;
            let addr = get_address_indirect_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0xB1 => {
            let (addr, crossed) = get_address_indirect_y(cpu, bus);
            cpu.cycles += if crossed { 6 } else { 5 };
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };
    cpu.a = value;
    cpu.update_zero_negative(value);
}

pub fn ldx(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xA2 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xA6 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0xB6 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_y(cpu, bus);
            bus.read_cpu(addr)
        }
        0xAE => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        0xBE => {
            let (addr, crossed) = get_address_absolute_y(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };
    cpu.x = value;
    cpu.update_zero_negative(value);
}

pub fn ldy(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let value = match opcode {
        0xA0 => {
            cpu.cycles += 2;
            cpu.fetch_byte(bus)
        }
        0xA4 => {
            cpu.cycles += 3;
            let addr = get_address_zeropage(cpu, bus);
            bus.read_cpu(addr)
        }
        0xB4 => {
            cpu.cycles += 4;
            let addr = get_address_zeropage_x(cpu, bus);
            bus.read_cpu(addr)
        }
        0xAC => {
            cpu.cycles += 4;
            let addr = get_address_absolute(cpu, bus);
            bus.read_cpu(addr)
        }
        0xBC => {
            let (addr, crossed) = get_address_absolute_x(cpu, bus);
            cpu.cycles += if crossed { 5 } else { 4 };
            bus.read_cpu(addr)
        }
        _ => unreachable!(),
    };
    cpu.y = value;
    cpu.update_zero_negative(value);
}

pub fn sta(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
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
            get_address_absolute_x_write(cpu, bus)
        }
        0x99 => {
            cpu.cycles += 5;
            get_address_absolute_y_write(cpu, bus)
        }
        0x81 => {
            cpu.cycles += 6;
            get_address_indirect_x(cpu, bus)
        }
        0x91 => {
            cpu.cycles += 6;
            get_address_indirect_y_write(cpu, bus)
        }
        _ => unreachable!(),
    };
    bus.write_cpu(addr, cpu.a);
}

pub fn stx(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
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
    bus.write_cpu(addr, cpu.x);
}

pub fn sty(cpu: &mut CPU, opcode: u8, bus: &mut Bus) {
    let addr = match opcode {
        0x84 => {
            cpu.cycles += 3;
            get_address_zeropage(cpu, bus)
        }
        0x94 => {
            cpu.cycles += 4;
            get_address_zeropage_x(cpu, bus)
        }
        0x8C => {
            cpu.cycles += 4;
            get_address_absolute(cpu, bus)
        }
        _ => unreachable!(),
    };
    bus.write_cpu(addr, cpu.y);
}
