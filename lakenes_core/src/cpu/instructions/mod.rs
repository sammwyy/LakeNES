pub mod arithmetic;
pub mod branch;
pub mod compare;
pub mod control;
pub mod illegal;
pub mod incdec;
pub mod loadstore;
pub mod logical;
pub mod shift;
pub mod stack;
pub mod system;
pub mod transfer;

pub use arithmetic::*;
pub use branch::*;
pub use compare::*;
pub use control::*;
pub use incdec::*;
pub use loadstore::*;
pub use logical::*;
pub use shift::*;
pub use stack::*;
pub use system::*;
pub use transfer::*;

use crate::bus::Bus;
pub use crate::cpu::*;

pub fn get_address_zeropage(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    cpu.fetch_byte(bus) as u16
}

pub fn get_address_zeropage_x(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    cpu.fetch_byte(bus).wrapping_add(cpu.x) as u16
}

pub fn get_address_zeropage_y(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    cpu.fetch_byte(bus).wrapping_add(cpu.y) as u16
}

pub fn get_address_absolute(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    cpu.fetch_word(bus)
}

pub fn get_address_absolute_x(cpu: &mut CPU, bus: &mut Bus) -> (u16, bool) {
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.x as u16);
    let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
    if page_crossed {
        let dummy = (base & 0xFF00) | (addr & 0x00FF);
        let _ = bus.read_cpu(dummy);
    }
    (addr, page_crossed)
}

pub fn get_address_absolute_y(cpu: &mut CPU, bus: &mut Bus) -> (u16, bool) {
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.y as u16);
    let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
    if page_crossed {
        let dummy = (base & 0xFF00) | (addr & 0x00FF);
        let _ = bus.read_cpu(dummy);
    }
    (addr, page_crossed)
}

pub fn get_address_indirect_x(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    let ptr = cpu.fetch_byte(bus).wrapping_add(cpu.x);
    let lo = bus.read_cpu(ptr as u16) as u16;
    let hi = bus.read_cpu(ptr.wrapping_add(1) as u16) as u16;
    (hi << 8) | lo
}

pub fn get_address_indirect_y(cpu: &mut CPU, bus: &mut Bus) -> (u16, bool) {
    let ptr = cpu.fetch_byte(bus);
    let lo = bus.read_cpu(ptr as u16) as u16;
    let hi = bus.read_cpu(ptr.wrapping_add(1) as u16) as u16;

    let base = (hi << 8) | lo;
    let addr = base.wrapping_add(cpu.y as u16);
    let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
    if page_crossed {
        let dummy = (base & 0xFF00) | (addr & 0x00FF);
        let _ = bus.read_cpu(dummy);
    }
    (addr, page_crossed)
}

pub fn get_address_absolute_x_write(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.x as u16);
    let dummy = (base & 0xFF00) | (addr & 0x00FF);
    let _ = bus.read_cpu(dummy);
    addr
}

pub fn get_address_absolute_y_write(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.y as u16);
    let dummy = (base & 0xFF00) | (addr & 0x00FF);
    let _ = bus.read_cpu(dummy);
    addr
}

pub fn get_address_indirect_y_write(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    let ptr = cpu.fetch_byte(bus);
    let lo = bus.read_cpu(ptr as u16) as u16;
    let hi = bus.read_cpu(ptr.wrapping_add(1) as u16) as u16;
    let base = (hi << 8) | lo;
    let addr = base.wrapping_add(cpu.y as u16);
    let dummy = (base & 0xFF00) | (addr & 0x00FF);
    let _ = bus.read_cpu(dummy);
    addr
}

/// Read-modify-write: the 6502 writes the original byte, then the modified byte.
pub fn rmw_store(cpu: &CPU, bus: &mut Bus, addr: u16, original: u8, new_value: u8) {
    bus.write_cpu(addr, original, cpu.cycles);
    bus.write_cpu(addr, new_value, cpu.cycles);
}
