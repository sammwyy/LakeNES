use super::{Mapper, Mirroring};
use alloc::vec::Vec;

pub struct AxROM {
    prg_rom: Vec<u8>,
    chr: Vec<u8>,
    selected_prg_bank: usize,
    one_screen_high: bool,
}

impl AxROM {
    pub fn new(prg_rom: Vec<u8>, chr: Vec<u8>) -> Self {
        Self {
            prg_rom,
            chr,
            selected_prg_bank: 0,
            one_screen_high: false,
        }
    }
}

impl Mapper for AxROM {
    fn read_prg(&mut self, addr: u16) -> u8 {
        if addr < 0x8000 {
            return 0;
        }

        let bank_base = self.selected_prg_bank * 32 * 1024;
        let idx = bank_base + (addr as usize & 0x7FFF);
        self.prg_rom.get(idx % self.prg_rom.len()).copied().unwrap_or(0)
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if addr >= 0x8000 {
            self.selected_prg_bank = (data & 0x07) as usize;
            self.one_screen_high = (data & 0x10) != 0;
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.chr.get(addr as usize & 0x1FFF).copied().unwrap_or(0)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let idx = addr as usize & 0x1FFF;
        if idx < self.chr.len() {
            self.chr[idx] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        if self.one_screen_high {
            Mirroring::OneScreenHigh
        } else {
            Mirroring::OneScreenLow
        }
    }
}
