pub mod adc;
pub mod and;
pub mod asl;
pub mod bcc;
pub mod bcs;
pub mod beq;
pub mod bit;
pub mod bmi;
pub mod bne;
pub mod bpl;
pub mod brk;
pub mod bvc;
pub mod bvs;
pub mod clc;
pub mod cld;
pub mod cli;
pub mod clv;
pub mod cmp;
pub mod cpx;
pub mod cpy;
pub mod dec;
pub mod dex;
pub mod dey;
pub mod eor;
pub mod inc;
pub mod inx;
pub mod iny;
pub mod jmp;
pub mod jsr;
pub mod lda;
pub mod ldx;
pub mod ldy;
pub mod lsr;
pub mod nop;
pub mod ora;
pub mod pha;
pub mod php;
pub mod pla;
pub mod plp;
pub mod rol;
pub mod ror;
pub mod rti;
pub mod rts;
pub mod sbc;
pub mod sec;
pub mod sed;
pub mod sei;
pub mod sta;
pub mod stx;
pub mod sty;
pub mod tax;
pub mod tay;
pub mod tsx;
pub mod txa;
pub mod txs;
pub mod tya;

use crate::bus::Bus;
use crate::cpu::CPU;

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
    (addr, page_crossed)
}

pub fn get_address_absolute_y(cpu: &mut CPU, bus: &mut Bus) -> (u16, bool) {
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.y as u16);
    let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
    (addr, page_crossed)
}

pub fn get_address_indirect_x(cpu: &mut CPU, bus: &mut Bus) -> u16 {
    let ptr = cpu.fetch_byte(bus).wrapping_add(cpu.x);
    let lo = bus.read(ptr as u16) as u16;
    let hi = bus.read(ptr.wrapping_add(1) as u16) as u16;
    (hi << 8) | lo
}

pub fn get_address_indirect_y(cpu: &mut CPU, bus: &mut Bus) -> (u16, bool) {
    let ptr = cpu.fetch_byte(bus);
    let lo = bus.read(ptr as u16) as u16;
    let hi = bus.read(ptr.wrapping_add(1) as u16) as u16;

    let base = (hi << 8) | lo;
    let addr = base.wrapping_add(cpu.y as u16);
    let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
    (addr, page_crossed)
}
