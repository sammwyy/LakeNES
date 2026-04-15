mod instructions;

use crate::bus::Bus;
use instructions::*;

const STACK_BASE: u16 = 0x0100;
const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC;
const IRQ_VECTOR: u16 = 0xFFFE;

pub(crate) const FLAG_C: u8 = 0b00000001;
pub(crate) const FLAG_Z: u8 = 0b00000010;
pub(crate) const FLAG_I: u8 = 0b00000100;
pub(crate) const FLAG_D: u8 = 0b00001000;
pub(crate) const FLAG_B: u8 = 0b00010000;
pub(crate) const FLAG_U: u8 = 0b00100000;
pub(crate) const FLAG_V: u8 = 0b01000000;
pub(crate) const FLAG_N: u8 = 0b10000000;

pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub p: u8,
    cycles: u64,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            p: FLAG_I | FLAG_U,
            cycles: 0,
        }
    }

    pub fn reset(&mut self, bus: &mut Bus) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.p = FLAG_I | FLAG_U;

        let lo = bus.read(RESET_VECTOR) as u16;
        bus.advance_ppu_dots(3);
        let hi = bus.read(RESET_VECTOR + 1) as u16;
        bus.advance_ppu_dots(3);
        self.pc = (hi << 8) | lo;

        self.cycles = 0;
        log::debug!("CPU reset, PC = 0x{:04X}", self.pc);
    }

    pub fn step(&mut self, bus: &mut Bus) -> u64 {
        let cycles_before = self.cycles;
        bus.begin_cpu_instruction();

        if bus.poll_nmi() {
            self.handle_nmi(bus);
        } else if bus.poll_irq() && !self.get_flag(FLAG_I) {
            self.handle_irq(bus);
        } else {
            let opcode = self.fetch_byte(bus);
            self.execute(opcode, bus);
        }

        let c = self.cycles - cycles_before;
        bus.ppu_end_instruction_catch_up(c);

        c
    }

    fn handle_nmi(&mut self, bus: &mut Bus) {
        self.push_word(bus, self.pc);
        self.push_byte(bus, (self.p | FLAG_U) & !FLAG_B);

        self.set_flag(FLAG_I, true);

        let lo = bus.read_cpu(NMI_VECTOR) as u16;
        let hi = bus.read_cpu(NMI_VECTOR + 1) as u16;
        self.pc = (hi << 8) | lo;
        self.cycles += 7;
        log::debug!("NMI handled, jumping to 0x{:04X}", self.pc);
    }

    fn handle_irq(&mut self, bus: &mut Bus) {
        self.push_word(bus, self.pc);
        self.push_byte(bus, self.p & !FLAG_B);
        self.set_flag(FLAG_I, true);

        let lo = bus.read_cpu(IRQ_VECTOR) as u16;
        let hi = bus.read_cpu(IRQ_VECTOR + 1) as u16;
        self.pc = (hi << 8) | lo;

        self.cycles += 7;
        log::debug!("IRQ handled, jumping to 0x{:04X}", self.pc);
    }

    fn execute(&mut self, opcode: u8, bus: &mut Bus) {
        /*
        log::debug!(
            "PC: 0x{:04X} | Opcode: 0x{:02X} | A: 0x{:02X} X: 0x{:02X} Y: 0x{:02X} P: 0x{:02X} SP: 0x{:02X}",
            self.pc - 1,
            opcode,
            self.a,
            self.x,
            self.y,
            self.p,
            self.sp
        ); */

        match opcode {
            0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => lda(self, opcode, bus),
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => ldx(self, opcode, bus),
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => ldy(self, opcode, bus),
            0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => sta(self, opcode, bus),
            0x86 | 0x96 | 0x8E => stx(self, opcode, bus),
            0x84 | 0x94 | 0x8C => sty(self, opcode, bus),
            0xAA => tax(self),
            0xA8 => tay(self),
            0xBA => tsx(self),
            0x8A => txa(self),
            0x9A => txs(self),
            0x98 => tya(self),
            0x48 => pha(self, bus),
            0x08 => php(self, bus),
            0x68 => pla(self, bus),
            0x28 => plp(self, bus),
            0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => and(self, opcode, bus),
            0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => eor(self, opcode, bus),
            0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => ora(self, opcode, bus),
            0x24 | 0x2C => bit(self, opcode, bus),
            0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => adc(self, opcode, bus),
            0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => sbc(self, opcode, bus),
            0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => cmp(self, opcode, bus),
            0xE0 | 0xE4 | 0xEC => cpx(self, opcode, bus),
            0xC0 | 0xC4 | 0xCC => cpy(self, opcode, bus),
            0xE6 | 0xF6 | 0xEE | 0xFE => inc(self, opcode, bus),
            0xE8 => inx(self),
            0xC8 => iny(self),
            0xC6 | 0xD6 | 0xCE | 0xDE => dec(self, opcode, bus),
            0xCA => dex(self),
            0x88 => dey(self),
            0x0A | 0x06 | 0x16 | 0x0E | 0x1E => asl(self, opcode, bus),
            0x4A | 0x46 | 0x56 | 0x4E | 0x5E => lsr(self, opcode, bus),
            0x2A | 0x26 | 0x36 | 0x2E | 0x3E => rol(self, opcode, bus),
            0x6A | 0x66 | 0x76 | 0x6E | 0x7E => ror(self, opcode, bus),
            0x4C | 0x6C => jmp(self, opcode, bus),
            0x20 => jsr(self, bus),
            0x60 => rts(self, bus),
            0x40 => rti(self, bus),
            0x90 => bcc(self, bus),
            0xB0 => bcs(self, bus),
            0xF0 => beq(self, bus),
            0x30 => bmi(self, bus),
            0xD0 => bne(self, bus),
            0x10 => bpl(self, bus),
            0x50 => bvc(self, bus),
            0x70 => bvs(self, bus),
            0x18 => clc(self),
            0xD8 => cld(self),
            0x58 => cli(self),
            0xB8 => clv(self),
            0x38 => sec(self),
            0xF8 => sed(self),
            0x78 => sei(self),
            0x00 => brk(self, bus),
            0xEA => nop(self),
            _ => illegal::execute_illegal(self, bus, opcode),
        }
    }

    pub fn fetch_byte(&mut self, bus: &mut Bus) -> u8 {
        let byte = bus.read_cpu(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    pub fn fetch_word(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.fetch_byte(bus) as u16;
        let hi = self.fetch_byte(bus) as u16;
        (hi << 8) | lo
    }

    pub fn push_byte(&mut self, bus: &mut Bus, value: u8) {
        bus.write_cpu(STACK_BASE + self.sp as u16, value, self.cycles);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn push_word(&mut self, bus: &mut Bus, value: u16) {
        self.push_byte(bus, (value >> 8) as u8);
        self.push_byte(bus, (value & 0xFF) as u8);
    }

    pub fn pop_byte(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read_cpu(STACK_BASE + self.sp as u16)
    }

    pub fn pop_word(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.pop_byte(bus) as u16;
        let hi = self.pop_byte(bus) as u16;
        (hi << 8) | lo
    }

    pub fn get_flag(&self, flag: u8) -> bool {
        (self.p & flag) != 0
    }

    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    pub fn update_zero_negative(&mut self, value: u8) {
        self.set_flag(FLAG_Z, value == 0);
        self.set_flag(FLAG_N, value & 0x80 != 0);
    }
}
