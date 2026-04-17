use super::{Mapper, Mirroring};
use alloc::vec::Vec;

pub struct NTDec5in1 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,

    // Registers
    prg_bank: u8,
    chr_bank: u8,
    prg_mode: u8, // 0: 16k, 1: 32k
    mirroring: Mirroring,
}

impl NTDec5in1 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_bank: 0,
            chr_bank: 0,
            prg_mode: 0,
            mirroring: Mirroring::Vertical,
        }
    }

    fn update_registers(&mut self, addr: u16) {
        // A~[.... .... OPPP CCCM]
        // M = bit 0
        // C = bits 1, 2, 3
        // P = bits 4, 5, 6
        // O = bit 7
        self.mirroring = if (addr & 0x01) != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        self.chr_bank = ((addr >> 1) & 0x07) as u8;
        self.prg_bank = ((addr >> 4) & 0x07) as u8;
        self.prg_mode = ((addr >> 7) & 0x01) as u8;
    }
}

impl Mapper for NTDec5in1 {
    fn read_prg(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x8000..=0xFFFF => {
                let offset = if self.prg_mode == 0 {
                    // 16 KB mode: bank P at both $8000 and $C000
                    let bank = self.prg_bank as usize;
                    (bank * 16384) + (addr as usize & 0x3FFF)
                } else {
                    // 32 KB mode: bank (P & 0x06) at $8000
                    let bank = (self.prg_bank & 0x06) as usize;
                    (bank * 16384) + (addr as usize & 0x7FFF)
                };

                if offset < self.prg_rom.len() {
                    Some(self.prg_rom[offset])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn write_prg(&mut self, addr: u16, _data: u8) {
        self.update_registers(addr);
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            let offset = (self.chr_bank as usize * 8192) + (addr as usize);
            if offset < self.chr_rom.len() {
                self.chr_rom[offset]
            } else {
                0
            }
        } else {
            0
        }
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if addr < 0x2000 {
            let offset = (self.chr_bank as usize * 8192) + (addr as usize);
            if offset < self.chr_rom.len() {
                self.chr_rom[offset] = data;
            }
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
