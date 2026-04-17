use super::{Mapper, Mirroring};
use alloc::{vec, vec::Vec};

pub struct MMC3 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,

    // Registers
    target_register: u8,
    regs: [u8; 8],

    prg_bank_mode: bool,
    chr_inversion: bool,
    mirroring: Mirroring,
    four_screen: bool,

    // $A001 — WRAM control
    wram_enabled: bool,
    wram_write_protect: bool,

    // IRQ
    irq_counter: u8,
    irq_latch: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_active: bool,

    // Helper
    num_prg_banks: usize,
    num_chr_banks: usize,

    /// Last A12 line state seen on the PPU bus (bit 12 of address).
    prev_a12: bool,
    /// PPU bus accesses with A12 low since last A12-high access (MMC3 needs ~3 low before a rising edge clocks the IRQ counter).
    a12_low_streak: u8,
}

impl MMC3 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, initial_mirroring: Mirroring) -> Self {
        let num_prg_banks = prg_rom.len() / 8192;
        let num_chr_banks = if !chr_rom.is_empty() {
            chr_rom.len() / 1024
        } else {
            0
        };
        let four_screen = initial_mirroring == Mirroring::FourScreen;

        Self {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; 8192],
            target_register: 0,
            regs: [0; 8],
            prg_bank_mode: false,
            chr_inversion: false,
            mirroring: initial_mirroring,
            four_screen,
            wram_enabled: true,
            wram_write_protect: false,
            irq_counter: 0,
            irq_latch: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_active: false,
            num_prg_banks,
            num_chr_banks,
            prev_a12: false,
            a12_low_streak: 0,
        }
    }

    fn clock_irq(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }
        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_active = true;
        }
    }

    fn read_prg_bank(&self, addr: u16) -> usize {
        let last_bank = self.num_prg_banks.saturating_sub(1);
        let second_last = self.num_prg_banks.saturating_sub(2);

        let bank_idx = match (addr >> 13) & 0x03 {
            0 => {
                if self.prg_bank_mode {
                    second_last
                } else {
                    self.regs[6] as usize
                }
            }
            1 => self.regs[7] as usize,
            2 => {
                if self.prg_bank_mode {
                    self.regs[6] as usize
                } else {
                    second_last
                }
            }
            3 => last_bank,
            _ => 0,
        };

        bank_idx % self.num_prg_banks
    }

    fn read_chr_bank(&self, addr: u16) -> usize {
        if self.num_chr_banks == 0 {
            return 0;
        }

        let bank = if self.chr_inversion {
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

impl Mapper for MMC3 {
    fn read_ex(&mut self, addr: u16) -> Option<u8> {
        match addr {
            // $6000-$7FFF: WRAM (only if enabled via $A001 bit 7)
            0x6000..=0x7FFF => {
                if self.wram_enabled {
                    Some(self.prg_ram[(addr - 0x6000) as usize])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn write_ex(&mut self, addr: u16, data: u8) {
        match addr {
            // $6000-$7FFF: WRAM (only if enabled and not write-protected via $A001)
            0x6000..=0x7FFF => {
                if self.wram_enabled && !self.wram_write_protect {
                    self.prg_ram[(addr - 0x6000) as usize] = data;
                }
            }
            _ => {}
        }
    }

    fn read_prg(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x8000..=0xFFFF => {
                let bank = self.read_prg_bank(addr);
                let offset = (bank * 8192) + (addr as usize & 0x1FFF);
                if offset < self.prg_rom.len() {
                    Some(self.prg_rom[offset])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => {
                if (addr & 1) == 0 {
                    self.target_register = data & 0x07;
                    self.prg_bank_mode = (data & 0x40) != 0;
                    self.chr_inversion = (data & 0x80) != 0;
                } else {
                    self.regs[self.target_register as usize] = data;
                }
            }
            0xA000..=0xBFFF => {
                if (addr & 1) == 0 {
                    // $A000: mirroring — ignored when 4-screen (hardwired on TR1ROM/TVROM)
                    if !self.four_screen {
                        self.mirroring = if (data & 0x01) == 0 {
                            Mirroring::Vertical
                        } else {
                            Mirroring::Horizontal
                        };
                    }
                } else {
                    // $A001: WRAM enable / write-protect
                    // Bit 7 (E): 0 = WRAM disabled, 1 = enabled
                    // Bit 6 (W): 0 = writable, 1 = write-protected
                    self.wram_enabled = (data & 0x80) != 0;
                    self.wram_write_protect = (data & 0x40) != 0;
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
                    self.irq_active = false;
                } else {
                    self.irq_enabled = true;
                }
            }
            _ => {}
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
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

    fn write_chr(&mut self, addr: u16, data: u8) {
        if self.chr_rom.is_empty() {
            return;
        }

        let bank = self.read_chr_bank(addr);
        let offset = (bank * 1024) + (addr as usize & 0x03FF);

        if offset < self.chr_rom.len() {
            self.chr_rom[offset] = data;
        }
    }

    fn irq_flag(&self) -> bool {
        self.irq_active
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn ppu_bus_address(&mut self, addr: u16) {
        let addr = addr & 0x3FFF;
        if addr >= 0x3F00 {
            return;
        }
        let a12 = (addr & 0x1000) != 0;

        if a12 {
            if !self.prev_a12 && self.a12_low_streak >= 3 {
                self.clock_irq();
            }
            self.a12_low_streak = 0;
        } else {
            self.a12_low_streak = self.a12_low_streak.saturating_add(1);
        }

        self.prev_a12 = a12;
    }
}
