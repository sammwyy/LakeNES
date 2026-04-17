use super::{Mapper, Mirroring};
use alloc::vec::Vec;
pub struct NJ0430 {
    prg_rom: Vec<u8>,
    chr_ram: Vec<u8>,
    prg_ram: Vec<u8>,

    // Registers
    mirroring: Mirroring,
    prg_mode: u8,
    bank_low: u8,
    bank_high: u8,
    ram_bank: u8,

    // Submapper features
    submapper: u8,
    irq_enabled: bool,
    irq_pending: bool,
    solder_pad_enabled: bool,
}

impl NJ0430 {
    pub fn new(prg_rom: Vec<u8>, submapper: u8) -> Self {
        Self {
            prg_rom,
            chr_ram: alloc::vec![0u8; 8192],
            prg_ram: alloc::vec![0u8; 128 * 1024], // 128 KiB of PRG-RAM

            mirroring: Mirroring::Vertical,
            prg_mode: 0,
            bank_low: 0,
            bank_high: 0,
            ram_bank: 0,

            submapper,
            irq_enabled: true, // "The I bit is set at power-on"
            irq_pending: false,
            solder_pad_enabled: false,
        }
    }

    fn get_prg_offset(&self, addr: u16) -> usize {
        let bank_base = ((self.bank_high as usize) << 3) | (self.bank_low as usize & 0x07);

        match self.prg_mode {
            0 => {
                // NROM-256/BNROM (PRG A14=CPU A14)
                let bank = bank_base & !0x01;
                (bank * 16384) + (addr as usize - 0x8000)
            }
            1 => {
                // UNROM (PRG A14..16=111b if CPU A14=1)
                if addr < 0xC000 {
                    bank_base * 16384 + (addr as usize - 0x8000)
                } else {
                    let outer = (self.bank_high as usize) << 3;
                    (outer | 7) * 16384 + (addr as usize - 0xC000)
                }
            }
            2 => {
                // NROM-128
                bank_base * 16384 + (addr as usize & 0x3FFF)
            }
            3 => {
                // UNROM alternate
                if addr < 0xC000 {
                    bank_base * 16384 + (addr as usize - 0x8000)
                } else {
                    let outer = (self.bank_high as usize) << 3;
                    let bank = outer | 0x06 | (self.bank_low as usize & 0x01);
                    bank * 16384 + (addr as usize - 0xC000)
                }
            }
            _ => 0,
        }
    }
}

impl Mapper for NJ0430 {
    fn read_ex(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x5000..=0x5FFF => {
                match self.submapper {
                    1 => Some(0), // IR sensor bit 0
                    2 => Some(0), // LPC Speech Chip
                    _ => None,
                }
            }
            0x6000..=0x7FFF => {
                let offset = (self.ram_bank as usize * 8192) + (addr as usize - 0x6000);
                if offset < self.prg_ram.len() {
                    Some(self.prg_ram[offset])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn write_ex(&mut self, addr: u16, data: u8) {
        match addr {
            0x4800 => {
                self.mirroring = if (data & 0x01) != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                self.prg_mode = (data >> 1) & 0x03;
            }
            0x4801 => {
                self.bank_low = data & 0x07;
            }
            0x4802 => {
                self.bank_high = data;
            }
            0x4803 => {
                self.ram_bank = data;
            }
            0x6000..=0x7FFF => {
                if self.submapper == 1 {
                    self.irq_enabled = (data & 0x80) != 0;
                    if !self.irq_enabled {
                        self.irq_pending = false;
                    }
                } else if self.submapper == 3 {
                    self.solder_pad_enabled = (data & 0x01) != 0;
                }

                let offset = (self.ram_bank as usize * 8192) + (addr as usize - 0x6000);
                if offset < self.prg_ram.len() {
                    self.prg_ram[offset] = data;
                }
            }
            _ => {}
        }
    }

    fn read_prg(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x8000..=0xFFFF => {
                if self.submapper == 3 && self.solder_pad_enabled {
                    // Solder path returns 2nd bit as well? Spec says 2-bit value.
                    // We'll return 0 for the menu selection.
                    Some(0)
                } else {
                    let offset = self.get_prg_offset(addr);
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

    fn write_prg(&mut self, _addr: u16, _data: u8) {}

    fn read_chr(&mut self, addr: u16) -> u8 {
        if (addr as usize) < self.chr_ram.len() {
            self.chr_ram[addr as usize]
        } else {
            0
        }
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if (addr as usize) < self.chr_ram.len() {
            self.chr_ram[addr as usize] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn irq_flag(&self) -> bool {
        self.irq_pending
    }
}
