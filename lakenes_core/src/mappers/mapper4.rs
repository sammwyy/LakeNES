use super::{Mapper, Mirroring};
use alloc::{vec, vec::Vec};

pub struct Mapper4 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,

    // Registers
    target_register: u8,
    regs: [u8; 8],

    prg_bank_mode: bool,
    chr_inversion: bool,
    mirroring: Mirroring,

    // IRQ
    irq_counter: u8,
    irq_latch: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_active: bool,

    // Helper
    num_prg_banks: usize,
    num_chr_banks: usize,

    // A12 Filter
    prev_a12: bool,
    // We can't really time-filter without cycles.
    // We'll trust the access pattern isn't too noisy for now.
}

impl Mapper4 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let num_prg_banks = prg_rom.len() / 8192;
        let num_chr_banks = if !chr_rom.is_empty() {
            chr_rom.len() / 1024
        } else {
            0
        };

        Self {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; 8192],
            target_register: 0,
            regs: [0; 8],
            prg_bank_mode: false,
            chr_inversion: false,
            mirroring: Mirroring::Vertical,
            irq_counter: 0,
            irq_latch: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_active: false,
            num_prg_banks,
            num_chr_banks,
            prev_a12: false,
        }
    }

    fn clock_irq(&mut self) {
        if self.irq_reload || self.irq_counter == 0 {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter = self.irq_counter.saturating_sub(1);
        }

        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_active = true;
        }
    }

    fn check_a12(&mut self, addr: u16) {
        let a12 = (addr & 0x1000) != 0;
        // Rising edge
        if a12 && !self.prev_a12 {
            self.clock_irq();
        }
        self.prev_a12 = a12;
    }

    fn read_prg_bank(&self, addr: u16) -> usize {
        // 8KB Banks
        // Fixed: -1 is Last Bank. -2 is Second to Last.
        let last_bank = self.num_prg_banks.saturating_sub(1);
        let second_last = self.num_prg_banks.saturating_sub(2);

        let bank_idx = match (addr >> 13) & 0x03 {
            0 => {
                // $8000
                if self.prg_bank_mode {
                    second_last
                } else {
                    self.regs[6] as usize
                }
            }
            1 => {
                // $A000
                self.regs[7] as usize
            }
            2 => {
                // $C000
                if self.prg_bank_mode {
                    self.regs[6] as usize
                } else {
                    second_last
                }
            }
            3 => {
                // $E000
                last_bank
            }
            _ => 0,
        };

        bank_idx % self.num_prg_banks
    }

    fn read_chr_bank(&self, addr: u16) -> usize {
        // 1KB Banks
        // R0, R1 = 2KB blocks (index & FE)
        // R2, R3, R4, R5 = 1KB blocks

        let bank = if self.chr_inversion {
            // $0000-$0FFF: 4 x 1KB (R2,R3,R4,R5)
            // $1000-$1FFF: 2 x 2KB (R0,R1)
            match addr {
                0x0000..=0x03FF => self.regs[2] as usize,
                0x0400..=0x07FF => self.regs[3] as usize,
                0x0800..=0x0BFF => self.regs[4] as usize,
                0x0C00..=0x0FFF => self.regs[5] as usize,
                0x1000..=0x17FF => (self.regs[0] & 0xFE) as usize + ((addr >> 10) & 1) as usize,
                0x1800..=0x1FFF => (self.regs[1] & 0xFE) as usize + ((addr >> 10) & 1) as usize,
                _ => 0,
            }
        } else {
            // $0000-$0FFF: 2 x 2KB (R0,R1)
            // $1000-$1FFF: 4 x 1KB (R2,R3,R4,R5)
            match addr {
                0x0000..=0x07FF => (self.regs[0] & 0xFE) as usize + ((addr >> 10) & 1) as usize,
                0x0800..=0x0FFF => (self.regs[1] & 0xFE) as usize + ((addr >> 10) & 1) as usize,
                0x1000..=0x13FF => self.regs[2] as usize,
                0x1400..=0x17FF => self.regs[3] as usize,
                0x1800..=0x1BFF => self.regs[4] as usize,
                0x1C00..=0x1FFF => self.regs[5] as usize,
                _ => 0,
            }
        };
        bank % self.num_chr_banks
    }
}

impl Mapper for Mapper4 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize],
            0x8000..=0xFFFF => {
                let bank = self.read_prg_bank(addr);
                let offset = (bank * 8192) + (addr as usize & 0x1FFF);
                if offset < self.prg_rom.len() {
                    self.prg_rom[offset]
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => {
                self.prg_ram[(addr - 0x6000) as usize] = data;
            }
            0x8000..=0x9FFF => {
                if (addr & 1) == 0 {
                    // Bank Select (Even)
                    self.target_register = data & 0x07;
                    self.prg_bank_mode = (data & 0x40) != 0;
                    self.chr_inversion = (data & 0x80) != 0;
                } else {
                    // Bank Data (Odd)
                    self.regs[self.target_register as usize] = data;
                }
            }
            0xA000..=0xBFFF => {
                if (addr & 1) == 0 {
                    // Mirroring
                    self.mirroring = if (data & 0x01) == 0 {
                        Mirroring::Vertical
                    } else {
                        Mirroring::Horizontal
                    };
                } else {
                    // PRG RAM Protect
                }
            }
            0xC000..=0xDFFF => {
                if (addr & 1) == 0 {
                    self.irq_latch = data;
                } else {
                    self.irq_reload = true;
                }
            }
            0xE000..=0xFFFF => {
                if (addr & 1) == 0 {
                    self.irq_enabled = false;
                    self.irq_active = false; // Acknowledge IRQ
                } else {
                    self.irq_enabled = true;
                }
            }
            _ => {}
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.check_a12(addr);
        if self.chr_rom.is_empty() {
            return 0;
        }

        let bank = self.read_chr_bank(addr);
        let offset = (bank * 1024) + (addr as usize & 0x03FF);

        if offset < self.chr_rom.len() {
            self.chr_rom[offset]
        } else {
            0
        }
    }

    fn write_chr(&mut self, addr: u16, _data: u8) {
        self.check_a12(addr);
        // CHR RAM support ignored
    }

    fn irq_flag(&self) -> bool {
        self.irq_active
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
