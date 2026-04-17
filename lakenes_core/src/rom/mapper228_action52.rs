use crate::rom::{Mapper, Mirroring};
use alloc::vec::Vec;

pub struct Mapper228 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    
    // Registers
    regs: [u8; 4],
    
    // Banking state
    prg_chip: u8,
    prg_page: u8,
    prg_mode: u8,
    mirroring: Mirroring,
    chr_bank: u8,
}

impl Mapper228 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mut mapper = Self {
            prg_rom,
            chr_rom,
            regs: [0; 4],
            prg_chip: 0,
            prg_page: 0,
            prg_mode: 0,
            mirroring: Mirroring::Vertical,
            chr_bank: 0,
        };
        mapper.reset();
        mapper
    }

    fn update_banks(&mut self, addr: u16, data: u8) {
        // A~[..MH HPPP PPO. CCCC]
        // M = Mirroring (0=Vert, 1=Horz) - Bit 13
        // H = PRG Chip Select - Bits 12, 11
        // P = PRG Page Select - Bits 10-6
        // O = PRG Mode - Bit 5
        // C = High 4 bits of CHR - Bits 3-0
        
        self.mirroring = if (addr & 0x2000) != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        self.prg_chip = ((addr >> 11) & 0x03) as u8;
        self.prg_page = ((addr >> 6) & 0x1F) as u8;
        self.prg_mode = ((addr >> 5) & 0x01) as u8;
        
        let chr_high = (addr & 0x000F) as u8;
        let chr_low = (data & 0x03) as u8;
        self.chr_bank = (chr_high << 2) | chr_low;
    }
}

impl Mapper for Mapper228 {
    fn read_ex(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x4020..=0x5FFF => {
                let idx = (addr as usize - 0x4020) % 4;
                Some(self.regs[idx])
            }
            _ => None,
        }
    }

    fn write_ex(&mut self, addr: u16, data: u8) {
        match addr {
            0x4020..=0x5FFF => {
                let idx = (addr as usize - 0x4020) % 4;
                self.regs[idx] = data & 0x0F; // 4 bits each
            }
            _ => {}
        }
    }

    fn read_prg(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x8000..=0xFFFF => {
                // Chip 2 is open bus
                if self.prg_chip == 2 {
                    return None;
                }

                // Chip index mapping (0, 1, 3 in file mapped to 0, 1, 2)
                let chip_idx = match self.prg_chip {
                    0 => 0,
                    1 => 1,
                    3 => 2,
                    _ => return None,
                };

                let chip_offset = (chip_idx as usize) * 512 * 1024;

                if self.prg_mode == 0 {
                    // Mode 0: 32k bank
                    let bank = (self.prg_page >> 1) as usize;
                    let offset = chip_offset + bank * 32 * 1024 + (addr as usize & 0x7FFF);
                    if offset < self.prg_rom.len() {
                        Some(self.prg_rom[offset])
                    } else {
                        None
                    }
                } else {
                    // Mode 1: 16k bank mirrored at $8000 and $C000
                    let bank = self.prg_page as usize;
                    let offset = chip_offset + bank * 16 * 1024 + (addr as usize & 0x3FFF);
                    if offset < self.prg_rom.len() {
                        Some(self.prg_rom[offset])
                    } else {
                        None
                    }
                }
            }
            _ => None,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0xFFFF => {
                self.update_banks(addr, data);
            }
            _ => {}
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        let offset = (self.chr_bank as usize) * 8192 + (addr as usize & 0x1FFF);
        if offset < self.chr_rom.len() {
            self.chr_rom[offset]
        } else {
            0
        }
    }

    fn write_chr(&mut self, _addr: u16, _data: u8) {
        // CHR is usually ROM for Action 52
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn reset(&mut self) {
        self.prg_chip = 0;
        self.prg_page = 0;
        self.prg_mode = 0;
        self.mirroring = Mirroring::Vertical;
        self.chr_bank = 0;
        self.regs = [0; 4];
    }

    // $4020-4023 RAM
    fn read_ppu(&mut self, _addr: u16, _vram: &[u8]) -> Option<u8> {
        None
    }
}
