//! Unofficial 6502 opcodes (NES / Ricoh 2A03). Behaviour from nesdev.org `undocumented_opcodes.txt`.

use super::{
    get_address_absolute, get_address_absolute_x_write, get_address_absolute_y,
    get_address_absolute_y_write, get_address_indirect_x, get_address_indirect_y,
    get_address_indirect_y_write, get_address_zeropage, get_address_zeropage_x,
    get_address_zeropage_y, rmw_store,
};
use crate::bus::Bus;
use crate::cpu::{CPU, FLAG_C, FLAG_V};

pub fn execute_illegal(cpu: &mut CPU, bus: &mut Bus, opcode: u8) {
    match opcode {
        // DOP (double NOP) immediate — 2 bytes, 2 cycles ($02/$62 treated as DOP in emulators)
        0x02 | 0x62 | 0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => dop_imm(cpu, bus),

        // KIL / JAM — 1 byte on-chip; 2-cycle implied NOP so the machine keeps running
        0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2 => {
            cpu.cycles += 2;
        }

        // NOP implied
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => {
            cpu.cycles += 2;
        }

        // DOP zero page / zp,X
        0x04 | 0x44 | 0x64 => dop_zp(cpu, bus),
        0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => dop_zpx(cpu, bus),

        // TOP (triple NOP) absolute / abs,X
        0x0C => top_abs(cpu, bus),
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => top_abs_x(cpu, bus),

        // AAC / ANC
        0x0B | 0x2B => anc_imm(cpu, bus),

        // ALR / ASR
        0x4B => alr_imm(cpu, bus),

        // ARR
        0x6B => arr_imm(cpu, bus),

        // ATX / LAX #
        0xAB => lax_imm(cpu, bus),

        // AXS / SBX
        0xCB => axs_imm(cpu, bus),

        // Unofficial SBC immediate (same as $E9)
        0xEB => sbc_imm(cpu, bus),

        // XAA (approximate; real chip is unstable)
        0x8B => xaa_imm(cpu, bus),

        // SLO (ASO)
        0x07 => rmw_slo(cpu, bus, 5, RmwAddr::Zp),
        0x17 => rmw_slo(cpu, bus, 6, RmwAddr::ZpX),
        0x03 => rmw_slo(cpu, bus, 8, RmwAddr::IndX),
        0x13 => rmw_slo(cpu, bus, 8, RmwAddr::IndY),
        0x0F => rmw_slo(cpu, bus, 6, RmwAddr::Abs),
        0x1F => rmw_slo(cpu, bus, 7, RmwAddr::AbsX),
        0x1B => rmw_slo(cpu, bus, 7, RmwAddr::AbsY),

        // RLA
        0x27 => rmw_rla(cpu, bus, 5, RmwAddr::Zp),
        0x37 => rmw_rla(cpu, bus, 6, RmwAddr::ZpX),
        0x23 => rmw_rla(cpu, bus, 8, RmwAddr::IndX),
        0x33 => rmw_rla(cpu, bus, 8, RmwAddr::IndY),
        0x2F => rmw_rla(cpu, bus, 6, RmwAddr::Abs),
        0x3F => rmw_rla(cpu, bus, 7, RmwAddr::AbsX),
        0x3B => rmw_rla(cpu, bus, 7, RmwAddr::AbsY),

        // SRE (LSE)
        0x47 => rmw_sre(cpu, bus, 5, RmwAddr::Zp),
        0x57 => rmw_sre(cpu, bus, 6, RmwAddr::ZpX),
        0x43 => rmw_sre(cpu, bus, 8, RmwAddr::IndX),
        0x53 => rmw_sre(cpu, bus, 8, RmwAddr::IndY),
        0x4F => rmw_sre(cpu, bus, 6, RmwAddr::Abs),
        0x5F => rmw_sre(cpu, bus, 7, RmwAddr::AbsX),
        0x5B => rmw_sre(cpu, bus, 7, RmwAddr::AbsY),

        // RRA
        0x67 => rmw_rra(cpu, bus, 5, RmwAddr::Zp),
        0x77 => rmw_rra(cpu, bus, 6, RmwAddr::ZpX),
        0x63 => rmw_rra(cpu, bus, 8, RmwAddr::IndX),
        0x73 => rmw_rra(cpu, bus, 8, RmwAddr::IndY),
        0x6F => rmw_rra(cpu, bus, 6, RmwAddr::Abs),
        0x7F => rmw_rra(cpu, bus, 7, RmwAddr::AbsX),
        0x7B => rmw_rra(cpu, bus, 7, RmwAddr::AbsY),

        // SAX (AAX)
        0x87 => sax_zp(cpu, bus),
        0x97 => sax_zpy(cpu, bus),
        0x83 => sax_ind_x(cpu, bus),
        0x8F => sax_abs(cpu, bus),

        // LAX
        0xA7 => lax_zp(cpu, bus),
        0xB7 => lax_zpy(cpu, bus),
        0xAF => lax_abs(cpu, bus),
        0xBF => lax_abs_y(cpu, bus),
        0xA3 => lax_ind_x(cpu, bus),
        0xB3 => lax_ind_y(cpu, bus),

        // DCP (DCM)
        0xC7 => rmw_dcp(cpu, bus, 5, RmwAddr::Zp),
        0xD7 => rmw_dcp(cpu, bus, 6, RmwAddr::ZpX),
        0xC3 => rmw_dcp(cpu, bus, 8, RmwAddr::IndX),
        0xD3 => rmw_dcp(cpu, bus, 8, RmwAddr::IndY),
        0xCF => rmw_dcp(cpu, bus, 6, RmwAddr::Abs),
        0xDF => rmw_dcp(cpu, bus, 7, RmwAddr::AbsX),
        0xDB => rmw_dcp(cpu, bus, 7, RmwAddr::AbsY),

        // ISC / ISB / INS
        0xE7 => rmw_isc(cpu, bus, 5, RmwAddr::Zp),
        0xF7 => rmw_isc(cpu, bus, 6, RmwAddr::ZpX),
        0xE3 => rmw_isc(cpu, bus, 8, RmwAddr::IndX),
        0xF3 => rmw_isc(cpu, bus, 8, RmwAddr::IndY),
        0xEF => rmw_isc(cpu, bus, 6, RmwAddr::Abs),
        0xFF => rmw_isc(cpu, bus, 7, RmwAddr::AbsX),
        0xFB => rmw_isc(cpu, bus, 7, RmwAddr::AbsY),

        // SHA / AXA
        0x93 => sha_ind_y(cpu, bus),
        0x9F => sha_abs_y(cpu, bus),

        // SHY / SYA
        0x9C => shy_abs_x(cpu, bus),

        // SHX / SXA
        0x9E => shx_abs_y(cpu, bus),

        // SHS / TAS / XAS
        0x9B => tas_abs_y(cpu, bus),

        // LAS / LAE
        0xBB => las_abs_y(cpu, bus),

        _ => {
            log::warn!(
                "Unknown opcode: 0x{:02X} at PC: 0x{:04X}",
                opcode,
                cpu.pc.wrapping_sub(1)
            );
            cpu.cycles += 2;
        }
    }
}

fn dop_imm(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 2;
    let _ = cpu.fetch_byte(bus);
}

fn dop_zp(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 3;
    let a = get_address_zeropage(cpu, bus);
    let _ = bus.read_cpu(a);
}

fn dop_zpx(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    let a = get_address_zeropage_x(cpu, bus);
    let _ = bus.read_cpu(a);
}

fn top_abs(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    let a = get_address_absolute(cpu, bus);
    let _ = bus.read_cpu(a);
}

fn top_abs_x(cpu: &mut CPU, bus: &mut Bus) {
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.x as u16);
    let crossed = (base & 0xFF00) != (addr & 0xFF00);
    cpu.cycles += if crossed { 5 } else { 4 };
    let _ = bus.read_cpu(addr);
}

fn anc_imm(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 2;
    let imm = cpu.fetch_byte(bus);
    cpu.a &= imm;
    cpu.set_flag(FLAG_C, cpu.a & 0x80 != 0);
    cpu.update_zero_negative(cpu.a);
}

fn alr_imm(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 2;
    let imm = cpu.fetch_byte(bus);
    cpu.a &= imm;
    let c = cpu.a & 1;
    cpu.a >>= 1;
    cpu.set_flag(FLAG_C, c != 0);
    cpu.update_zero_negative(cpu.a);
}

fn arr_imm(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 2;
    let imm = cpu.fetch_byte(bus);
    cpu.a &= imm;
    let c_in = cpu.get_flag(FLAG_C);
    cpu.a = (cpu.a >> 1) | if c_in { 0x80 } else { 0 };

    let b = cpu.a;
    let bit5 = b & 0x20 != 0;
    let bit6 = b & 0x40 != 0;
    match (bit5, bit6) {
        (true, true) => {
            cpu.set_flag(FLAG_C, true);
            cpu.set_flag(FLAG_V, false);
        }
        (false, false) => {
            cpu.set_flag(FLAG_C, false);
            cpu.set_flag(FLAG_V, false);
        }
        (true, false) => {
            cpu.set_flag(FLAG_V, true);
            cpu.set_flag(FLAG_C, false);
        }
        (false, true) => {
            cpu.set_flag(FLAG_C, true);
            cpu.set_flag(FLAG_V, true);
        }
    }
    cpu.update_zero_negative(cpu.a);
}

fn lax_imm(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 2;
    let imm = cpu.fetch_byte(bus);
    cpu.a &= imm;
    cpu.x = cpu.a;
    cpu.update_zero_negative(cpu.a);
}

fn axs_imm(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 2;
    let imm = cpu.fetch_byte(bus);
    let ax = cpu.a & cpu.x;
    cpu.set_flag(FLAG_C, ax >= imm);
    cpu.x = ax.wrapping_sub(imm);
    cpu.update_zero_negative(cpu.x);
}

fn sbc_imm(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 2;
    let v = cpu.fetch_byte(bus);
    sbc_value(cpu, v);
}

fn xaa_imm(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 2;
    let imm = cpu.fetch_byte(bus);
    cpu.a = (cpu.a | 0xEE) & cpu.x & imm;
    cpu.update_zero_negative(cpu.a);
}

fn sax_zp(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 3;
    let a = get_address_zeropage(cpu, bus);
    bus.write_cpu(a, cpu.a & cpu.x, cpu.cycles);
}

fn sax_zpy(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    let a = get_address_zeropage_y(cpu, bus);
    bus.write_cpu(a, cpu.a & cpu.x, cpu.cycles);
}

fn sax_ind_x(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    let a = get_address_indirect_x(cpu, bus);
    bus.write_cpu(a, cpu.a & cpu.x, cpu.cycles);
}

fn sax_abs(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    let a = get_address_absolute(cpu, bus);
    bus.write_cpu(a, cpu.a & cpu.x, cpu.cycles);
}

fn lax_zp(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 3;
    let a = get_address_zeropage(cpu, bus);
    let v = bus.read_cpu(a);
    lax_store(cpu, v);
}

fn lax_zpy(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    let a = get_address_zeropage_y(cpu, bus);
    let v = bus.read_cpu(a);
    lax_store(cpu, v);
}

fn lax_abs(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 4;
    let a = get_address_absolute(cpu, bus);
    let v = bus.read_cpu(a);
    lax_store(cpu, v);
}

fn lax_abs_y(cpu: &mut CPU, bus: &mut Bus) {
    let (addr, crossed) = get_address_absolute_y(cpu, bus);
    cpu.cycles += if crossed { 5 } else { 4 };
    let v = bus.read_cpu(addr);
    lax_store(cpu, v);
}

fn lax_ind_x(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    let a = get_address_indirect_x(cpu, bus);
    let v = bus.read_cpu(a);
    lax_store(cpu, v);
}

fn lax_ind_y(cpu: &mut CPU, bus: &mut Bus) {
    let (addr, crossed) = get_address_indirect_y(cpu, bus);
    cpu.cycles += if crossed { 6 } else { 5 };
    let v = bus.read_cpu(addr);
    lax_store(cpu, v);
}

fn lax_store(cpu: &mut CPU, v: u8) {
    cpu.a = v;
    cpu.x = v;
    cpu.update_zero_negative(v);
}

/// SHA ($93) — (indirect),Y
/// Writes A & X & (base_high + 1).
/// If the Y addition crosses a page, the written address is also corrupted.
fn sha_ind_y(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 6;
    let ptr = cpu.fetch_byte(bus);
    let lo = bus.read_cpu(ptr as u16) as u16;
    let hi = bus.read_cpu(ptr.wrapping_add(1) as u16) as u16;
    let base = (hi << 8) | lo;
    let addr = base.wrapping_add(cpu.y as u16);
    let crossed = (base & 0xFF00) != (addr & 0xFF00);

    // Dummy read on the un-carried address
    let dummy = (base & 0xFF00) | (addr & 0x00FF);
    let _ = bus.read_cpu(dummy);

    let base_hi = (base >> 8) as u8;
    let val = cpu.a & cpu.x & base_hi.wrapping_add(1);
    let target = if crossed {
        (addr & 0x00FF) | ((val as u16) << 8)
    } else {
        addr
    };
    bus.write_cpu(target, val, cpu.cycles);
}

/// SHA ($9F) — absolute,Y
fn sha_abs_y(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 5;
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.y as u16);
    let crossed = (base & 0xFF00) != (addr & 0xFF00);

    // Dummy read on the un-carried address
    let dummy = (base & 0xFF00) | (addr & 0x00FF);
    let _ = bus.read_cpu(dummy);

    let base_hi = (base >> 8) as u8;
    let val = cpu.a & cpu.x & base_hi.wrapping_add(1);
    let target = if crossed {
        (addr & 0x00FF) | ((val as u16) << 8)
    } else {
        addr
    };
    bus.write_cpu(target, val, cpu.cycles);
}

/// SHY ($9C) — absolute,X
fn shy_abs_x(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 5;
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.x as u16);
    let crossed = (base & 0xFF00) != (addr & 0xFF00);

    // Dummy read on the un-carried address
    let dummy = (base & 0xFF00) | (addr & 0x00FF);
    let _ = bus.read_cpu(dummy);

    let base_hi = (base >> 8) as u8;
    let val = cpu.y & base_hi.wrapping_add(1);
    let target = if crossed {
        (addr & 0x00FF) | ((val as u16) << 8)
    } else {
        addr
    };
    bus.write_cpu(target, val, cpu.cycles);
}

/// SHX ($9E) — absolute,Y
fn shx_abs_y(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 5;
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.y as u16);
    let crossed = (base & 0xFF00) != (addr & 0xFF00);

    // Dummy read on the un-carried address
    let dummy = (base & 0xFF00) | (addr & 0x00FF);
    let _ = bus.read_cpu(dummy);

    let base_hi = (base >> 8) as u8;
    let val = cpu.x & base_hi.wrapping_add(1);
    let target = if crossed {
        (addr & 0x00FF) | ((val as u16) << 8)
    } else {
        addr
    };
    bus.write_cpu(target, val, cpu.cycles);
}

/// TAS / SHS ($9B) — absolute,Y
/// SP = A & X, then writes SP & (base_high + 1) with address corruption on page cross.
fn tas_abs_y(cpu: &mut CPU, bus: &mut Bus) {
    cpu.cycles += 5;
    let base = cpu.fetch_word(bus);
    let addr = base.wrapping_add(cpu.y as u16);
    let crossed = (base & 0xFF00) != (addr & 0xFF00);

    // Dummy read on the un-carried address
    let dummy = (base & 0xFF00) | (addr & 0x00FF);
    let _ = bus.read_cpu(dummy);

    let s = cpu.a & cpu.x;
    cpu.sp = s;
    let base_hi = (base >> 8) as u8;
    let val = s & base_hi.wrapping_add(1);
    let target = if crossed {
        (addr & 0x00FF) | ((val as u16) << 8)
    } else {
        addr
    };
    bus.write_cpu(target, val, cpu.cycles);
}

fn las_abs_y(cpu: &mut CPU, bus: &mut Bus) {
    let (addr, crossed) = get_address_absolute_y(cpu, bus);
    cpu.cycles += if crossed { 5 } else { 4 };
    let t = bus.read_cpu(addr);
    let v = t & cpu.sp;
    cpu.a = v;
    cpu.x = v;
    cpu.sp = v;
    cpu.update_zero_negative(v);
}

#[derive(Clone, Copy)]
enum RmwAddr {
    Zp,
    ZpX,
    Abs,
    AbsX,
    AbsY,
    IndX,
    IndY,
}

fn resolve_rmw(cpu: &mut CPU, bus: &mut Bus, mode: RmwAddr) -> u16 {
    match mode {
        RmwAddr::Zp => get_address_zeropage(cpu, bus),
        RmwAddr::ZpX => get_address_zeropage_x(cpu, bus),
        RmwAddr::Abs => get_address_absolute(cpu, bus),
        RmwAddr::AbsX => get_address_absolute_x_write(cpu, bus),
        RmwAddr::AbsY => get_address_absolute_y_write(cpu, bus),
        RmwAddr::IndX => get_address_indirect_x(cpu, bus),
        RmwAddr::IndY => get_address_indirect_y_write(cpu, bus),
    }
}

fn rmw_slo(cpu: &mut CPU, bus: &mut Bus, cycles: u64, mode: RmwAddr) {
    cpu.cycles += cycles;
    let addr = resolve_rmw(cpu, bus, mode);
    let v = bus.read_cpu(addr);
    let c = (v & 0x80) != 0;
    let r = v.wrapping_shl(1);
    rmw_store(cpu, bus, addr, v, r);
    cpu.a |= r;
    cpu.set_flag(FLAG_C, c);
    cpu.update_zero_negative(cpu.a);
}

fn rmw_rla(cpu: &mut CPU, bus: &mut Bus, cycles: u64, mode: RmwAddr) {
    cpu.cycles += cycles;
    let addr = resolve_rmw(cpu, bus, mode);
    let v = bus.read_cpu(addr);
    let c_in = cpu.get_flag(FLAG_C);
    let new_c = (v & 0x80) != 0;
    let r = (v << 1) | if c_in { 1 } else { 0 };
    rmw_store(cpu, bus, addr, v, r);
    cpu.a &= r;
    cpu.set_flag(FLAG_C, new_c);
    cpu.update_zero_negative(cpu.a);
}

fn rmw_sre(cpu: &mut CPU, bus: &mut Bus, cycles: u64, mode: RmwAddr) {
    cpu.cycles += cycles;
    let addr = resolve_rmw(cpu, bus, mode);
    let v = bus.read_cpu(addr);
    let c = (v & 1) != 0;
    let r = v >> 1;
    rmw_store(cpu, bus, addr, v, r);
    cpu.a ^= r;
    cpu.set_flag(FLAG_C, c);
    cpu.update_zero_negative(cpu.a);
}

fn rmw_rra(cpu: &mut CPU, bus: &mut Bus, cycles: u64, mode: RmwAddr) {
    cpu.cycles += cycles;
    let addr = resolve_rmw(cpu, bus, mode);
    let v = bus.read_cpu(addr);
    let c_in = cpu.get_flag(FLAG_C);
    let new_c = (v & 1) != 0;
    let r = (v >> 1) | if c_in { 0x80 } else { 0 };
    rmw_store(cpu, bus, addr, v, r);
    cpu.set_flag(FLAG_C, new_c);
    adc_value(cpu, r);
}

fn rmw_dcp(cpu: &mut CPU, bus: &mut Bus, cycles: u64, mode: RmwAddr) {
    cpu.cycles += cycles;
    let addr = resolve_rmw(cpu, bus, mode);
    let v = bus.read_cpu(addr);
    let r = v.wrapping_sub(1);
    rmw_store(cpu, bus, addr, v, r);
    cmp_value(cpu, cpu.a, r);
}

fn rmw_isc(cpu: &mut CPU, bus: &mut Bus, cycles: u64, mode: RmwAddr) {
    cpu.cycles += cycles;
    let addr = resolve_rmw(cpu, bus, mode);
    let v = bus.read_cpu(addr);
    let inc = v.wrapping_add(1);
    rmw_store(cpu, bus, addr, v, inc);
    sbc_value(cpu, inc);
}

fn cmp_value(cpu: &mut CPU, a: u8, m: u8) {
    let diff = a.wrapping_sub(m);
    cpu.set_flag(FLAG_C, a >= m);
    cpu.update_zero_negative(diff);
}

fn adc_value(cpu: &mut CPU, value: u8) {
    let carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
    let sum = cpu.a as u16 + value as u16 + carry;
    let overflow = (cpu.a ^ sum as u8) & (value ^ sum as u8) & 0x80 != 0;
    cpu.set_flag(FLAG_C, sum > 0xFF);
    cpu.set_flag(FLAG_V, overflow);
    cpu.a = sum as u8;
    cpu.update_zero_negative(cpu.a);
}

fn sbc_value(cpu: &mut CPU, value: u8) {
    let carry = if cpu.get_flag(FLAG_C) { 1 } else { 0 };
    let val_inv = !value;
    let sum = cpu.a as u16 + val_inv as u16 + carry;
    let overflow = (cpu.a ^ sum as u8) & (val_inv ^ sum as u8) & 0x80 != 0;
    cpu.set_flag(FLAG_C, sum > 0xFF);
    cpu.set_flag(FLAG_V, overflow);
    cpu.a = sum as u8;
    cpu.update_zero_negative(cpu.a);
}
