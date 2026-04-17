use alloc::boxed::Box;

pub mod mapper000_nrom;
pub mod mapper001_mmc1;
pub mod mapper002_uxrom;
pub mod mapper003_cnrom;
pub mod mapper004_mmc3;
pub mod mapper005_mmc5;
pub mod mapper007_axrom;
pub mod mapper009_mmc2;
pub mod mapper010_mmc4;
pub mod mapper174_ntdec;
pub mod mapper178_nj0430;
pub mod mapper228_action52;

use mapper000_nrom::NROM;
use mapper001_mmc1::MMC1;
use mapper002_uxrom::UxROM;
use mapper003_cnrom::CNROM;
use mapper004_mmc3::MMC3;
use mapper005_mmc5::MMC5;
use mapper007_axrom::AxROM;
use mapper009_mmc2::MMC2;
use mapper010_mmc4::MMC4;
use crate::fds::FDS;
use mapper174_ntdec::NTDec5in1;
use mapper178_nj0430::NJ0430;
use mapper228_action52::Mapper228;

const NES_HEADER_SIZE: usize = 16;
const PRG_ROM_BANK_SIZE: usize = 16384;
const CHR_ROM_BANK_SIZE: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    OneScreenLow,
    OneScreenHigh,
    FourScreen,
}

/// TV system / timing mode parsed from the iNES/NES 2.0 header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimingMode {
    Ntsc,
    Pal,
    Multi,
    Dendy,
}

#[derive(Debug)]
pub enum ROMError {
    InvalidHeader,
    IncompleteData,
}

pub trait Mapper: Send {
    fn read_prg(&mut self, addr: u16) -> Option<u8>;
    fn write_prg(&mut self, addr: u16, data: u8);

    /// Expansion registers / RAM ($4020–$7FFF).
    fn read_ex(&mut self, _addr: u16) -> Option<u8> {
        None
    }
    /// Expansion registers / RAM ($4020–$7FFF).
    fn write_ex(&mut self, _addr: u16, _data: u8) {}

    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, data: u8);
    /// PPU-driven address bus (0x0000–0x3FFF). Used by MMC3 for A12 IRQ timing;
    /// nametable/attribute fetches must be included so A12 can go low between pattern fetches.
    fn ppu_bus_address(&mut self, _addr: u16) {}
    fn irq_flag(&self) -> bool {
        false
    }
    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }
    /// Allows the mapper to override PPU memory access (e.g. MMC5 nametable mapping).
    /// If it returns Some(data), the PPU will use that instead of its internal VRAM.
    fn read_ppu(&mut self, _addr: u16, _vram: &[u8]) -> Option<u8> {
        None
    }
    /// Allows the mapper to handle PPU memory writes.
    /// If it returns true, the PPU will skip its internal VRAM write.
    fn write_ppu(&mut self, _addr: u16, _data: u8, _vram: &mut [u8]) -> bool {
        false
    }
    /// Loads an external BIOS (like DISKSYS.ROM for FDS) into the mapper.
    fn load_bios(&mut self, _bios_data: &[u8]) {}
    fn reset(&mut self) {}
    /// Clocked every CPU cycle, useful for FDS or other CPU cycle based mappers.
    fn step_cpu(&mut self, _cycles: u64) {}
}

pub struct ROM {
    pub mapper: Box<dyn Mapper + Send>,
    pub mapper_id: u16,
    pub prg_size: usize,
    pub chr_size: usize,
    pub timing: TimingMode,
}

impl ROM {
    pub fn load_from_bytes(bytes: &[u8]) -> core::result::Result<Self, ROMError> {
        if bytes.len() < NES_HEADER_SIZE {
            return Err(ROMError::IncompleteData);
        }

        let header = &bytes[0..4];

        let is_fds = header == b"FDS\x1A" || (bytes.len() >= 65500 && bytes[0] == 0x01 && bytes.len() % 65500 == 0);

        if is_fds {
            let mapper_id = 20;
            let prg_size = 32768; // Ram sizes
            let chr_size = 8192;
            let mapper = Box::new(FDS::new(bytes.to_vec()));
            log::info!("FDS format intercepted: Mapper 20");
            return Ok(Self {
                mapper,
                mapper_id,
                prg_size,
                chr_size,
                timing: TimingMode::Ntsc,
            });
        }

        if header != b"NES\x1A" {
            return Err(ROMError::InvalidHeader);
        }

        let header = &bytes[0..NES_HEADER_SIZE];

        // Detect NES 2.0: bits 2-3 of byte 7 == 0b10
        let is_nes2 = (header[7] & 0x0C) == 0x08;

        // --- Mapper ID ---
        // iNES 1.0: 8-bit mapper from bytes 6+7
        // NES 2.0:  12-bit mapper from bytes 6+7+8
        let mapper_lo = ((header[7] & 0xF0) | (header[6] >> 4)) as u16;
        let mapper_id = if is_nes2 {
            mapper_lo | ((header[8] as u16 & 0x0F) << 8)
        } else {
            mapper_lo
        };

        let submapper = if is_nes2 { header[8] >> 4 } else { 0 };

        // --- PRG/CHR bank counts ---
        let prg_banks = if is_nes2 {
            // NES 2.0: byte 9 bits 0-3 are the MSB of PRG-ROM size
            header[4] as usize | ((header[9] as usize & 0x0F) << 8)
        } else {
            header[4] as usize
        };

        let chr_banks = if is_nes2 {
            // NES 2.0: byte 9 bits 4-7 are the MSB of CHR-ROM size
            header[5] as usize | ((header[9] as usize & 0xF0) << 4)
        } else {
            header[5] as usize
        };

        // --- Mirroring ---
        // Bit 3 of byte 6: four-screen VRAM (overrides bit 0)
        let four_screen = (header[6] & 0x08) != 0;
        let mirroring_mode = if four_screen {
            Mirroring::FourScreen
        } else if (header[6] & 0x01) != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        // --- Timing mode ---
        let timing = if is_nes2 {
            match header[12] & 0x03 {
                0 => TimingMode::Ntsc,
                1 => TimingMode::Pal,
                2 => TimingMode::Multi,
                3 => TimingMode::Dendy,
                _ => TimingMode::Ntsc,
            }
        } else {
            // iNES 1.0: byte 9 bit 0
            if (header[9] & 0x01) != 0 {
                TimingMode::Pal
            } else {
                TimingMode::Ntsc
            }
        };

        // --- Battery / PRG-RAM flags (informational) ---
        let _has_battery = (header[6] & 0x02) != 0;

        // --- Trainer ---
        let mut offset = NES_HEADER_SIZE;
        if header[6] & 0x04 != 0 {
            offset += 512; // Trainer
        }

        // --- PRG ROM ---
        let prg_size = prg_banks * PRG_ROM_BANK_SIZE;
        if bytes.len() < offset + prg_size {
            return Err(ROMError::IncompleteData);
        }
        let prg_rom = bytes[offset..offset + prg_size].to_vec();
        offset += prg_size;

        // --- CHR ROM ---
        let chr_size_on_disk = if chr_banks == 0 {
            CHR_ROM_BANK_SIZE
        } else {
            chr_banks * CHR_ROM_BANK_SIZE
        };

        let chr_rom = if chr_banks > 0 {
            if bytes.len() < offset + chr_size_on_disk {
                return Err(ROMError::IncompleteData);
            }
            let data = bytes[offset..offset + chr_size_on_disk].to_vec();
            data
        } else {
            alloc::vec![0u8; CHR_ROM_BANK_SIZE]
        };

        log::info!("PRG ROM: {} banks ({} bytes)", prg_banks, prg_rom.len());
        log::info!("CHR ROM: {} banks ({} bytes)", chr_banks, chr_rom.len());
        log::info!(
            "Mapper ID: {} (NES{}{})",
            mapper_id,
            if is_nes2 { " 2.0" } else { "" },
            if submapper != 0 {
                alloc::format!(", submapper {}", submapper)
            } else {
                alloc::string::String::new()
            }
        );
        log::info!("Mirroring: {:?}", mirroring_mode);
        log::info!("Timing: {:?}", timing);

        let prg_len = prg_rom.len();
        let chr_len = chr_rom.len();

        let mapper: Box<dyn Mapper + Send> = match mapper_id {
            0 => Box::new(NROM::new(
                prg_rom,
                chr_rom,
                prg_banks,
                chr_banks,
                mirroring_mode,
            )),
            1 => Box::new(MMC1::new(prg_rom, chr_rom)),
            2 => Box::new(UxROM::new(prg_rom, chr_rom, prg_banks, mirroring_mode)),
            3 => Box::new(CNROM::new(
                prg_rom,
                chr_rom,
                prg_banks,
                chr_banks == 0,
                mirroring_mode,
            )),
            4 => Box::new(MMC3::new(prg_rom, chr_rom, mirroring_mode)),
            5 => Box::new(MMC5::new(prg_rom, chr_rom)),
            7 => Box::new(AxROM::new(prg_rom, chr_rom)),
            9 => Box::new(MMC2::new(prg_rom, chr_rom, mirroring_mode)),
            10 => Box::new(MMC4::new(prg_rom, chr_rom, mirroring_mode)),
            20 => Box::new(FDS::new(bytes.to_vec())),
            228 => Box::new(Mapper228::new(prg_rom, chr_rom)),
            174 => Box::new(NTDec5in1::new(prg_rom, chr_rom)),
            178 => Box::new(NJ0430::new(prg_rom, submapper)),
            _ => {
                log::warn!("Mapper {} not implemented, falling back to NROM", mapper_id);
                Box::new(NROM::new(
                    prg_rom,
                    chr_rom,
                    prg_banks,
                    chr_banks,
                    mirroring_mode,
                ))
            }
        };

        Ok(Self {
            mapper,
            mapper_id: mapper_id,
            prg_size: prg_len,
            chr_size: chr_len,
            timing,
        })
    }
}
