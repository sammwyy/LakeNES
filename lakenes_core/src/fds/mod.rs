pub mod audio;
pub mod disk;

use crate::fds::audio::FDSAudio;
use crate::fds::disk::FDSDrive;
use crate::rom::{Mapper, Mirroring};
use alloc::vec::Vec;

pub struct FDS {
    pub prg_ram: [u8; 32768], // 32KB RAM at $6000-$DFFF
    pub chr_ram: [u8; 8192],  // 8KB CHR RAM
    pub bios: [u8; 8192],     // 8KB BIOS at $E000-$FFFF

    pub drive: FDSDrive,
    pub audio: FDSAudio,

    // Internal RAM adapter state
    pub timer_reload: u16,
    pub timer_counter: u16,
    pub timer_enabled: bool,
    pub timer_repeat: bool,
    pub timer_irq_pending: bool,

    pub disk_io_enabled: bool,
    pub sound_io_enabled: bool,
    pub mirroring: Mirroring,
}

impl FDS {
    pub fn new(fds_data: Vec<u8>) -> Self {
        Self {
            prg_ram: [0; 32768],
            chr_ram: [0; 8192],
            bios: [0; 8192],
            drive: FDSDrive::new(fds_data),
            audio: FDSAudio::new(),
            timer_reload: 0,
            timer_counter: 0,
            timer_enabled: false,
            timer_repeat: false,
            timer_irq_pending: false,
            disk_io_enabled: false,
            sound_io_enabled: false,
            mirroring: Mirroring::Vertical,
        }
    }
}

impl Mapper for FDS {
    fn read_prg(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x8000..=0xDFFF => Some(self.prg_ram[(addr - 0x6000) as usize]),
            0xE000..=0xFFFF => Some(self.bios[(addr - 0xE000) as usize]),
            _ => None,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if let 0x8000..=0xDFFF = addr {
            self.prg_ram[(addr - 0x6000) as usize] = data;
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.chr_ram[(addr & 0x1FFF) as usize]
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        self.chr_ram[(addr & 0x1FFF) as usize] = data;
    }

    fn read_ex(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x4030 => {
                let mut res = 0;
                if self.timer_irq_pending {
                    res |= 0x01;
                }
                if self.drive.byte_transfer_flag {
                    res |= 0x80;
                }
                if self.drive.end_of_head {
                    res |= 0x40;
                }
                self.timer_irq_pending = false;
                self.drive.disk_irq_pending = false;
                self.drive.byte_transfer_flag = false;
                log::trace!(
                    "$4030 read → 0x{:02X} (timer_irq={}, byte_transfer={}, end_of_head={})",
                    res,
                    self.timer_irq_pending,
                    self.drive.byte_transfer_flag,
                    self.drive.end_of_head
                );
                Some(res)
            }
            0x4031 => {
                let res = self.drive.data_register;
                log::trace!(
                    "$4031 read → 0x{:02X}  head_pos={}",
                    res,
                    self.drive.head_position
                );
                self.drive.byte_transfer_flag = false;
                self.drive.disk_irq_pending = false;
                Some(res)
            }
            0x4032 => {
                let mut res = 0;
                if self.drive.current_side_idx.is_none() {
                    res |= 0x01;
                }
                if !self.drive.ready_flag {
                    res |= 0x02;
                }
                if self.drive.current_side_idx.is_none() {
                    res |= 0x04;
                }
                self.drive.disk_irq_pending = false;

                // Log every status read to help debugging
                log::trace!(
                    "$4032 read → 0x{:02X}  ready={} motor={} head_pos={} transfer_reset={}",
                    res,
                    self.drive.ready_flag,
                    self.drive.motor_on,
                    self.drive.head_position,
                    self.drive.transfer_reset
                );
                Some(res)
            }
            0x4033 => Some(0x80), // Battery OK
            0x4040..=0x407F => Some(self.audio.read(addr)),
            0x6000..=0x7FFF => Some(self.prg_ram[(addr - 0x6000) as usize]),
            _ => None,
        }
    }

    fn write_ex(&mut self, addr: u16, data: u8) {
        match addr {
            0x4020 => self.timer_reload = (self.timer_reload & 0xFF00) | (data as u16),
            0x4021 => self.timer_reload = (self.timer_reload & 0x00FF) | ((data as u16) << 8),
            0x4022 => {
                self.timer_repeat = (data & 0x01) != 0;
                self.timer_enabled = (data & 0x02) != 0;
                if self.timer_enabled {
                    self.timer_counter = self.timer_reload;
                } else {
                    self.timer_irq_pending = false;
                }
            }
            0x4023 => {
                self.disk_io_enabled = (data & 0x01) != 0;
                self.sound_io_enabled = (data & 0x02) != 0;
                if !self.disk_io_enabled {
                    self.timer_irq_pending = false;
                    self.drive.disk_irq_pending = false;
                }
            }
            0x4024 => {
                self.drive.data_register = data;
                self.drive.byte_transfer_flag = false;
                self.drive.disk_irq_pending = false;
                log::trace!("$4024 write → 0x{:02X}", data);
            }
            0x4025 => {
                self.drive.disk_irq_enabled = (data & 0x80) != 0;
                self.mirroring = if (data & 0x08) != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                self.drive.mode_read = (data & 0x04) != 0;
                // bit 0: 1 = motor ON (no stop), 0 = motor off
                self.drive.motor_on = (data & 0x01) != 0;
                self.drive.transfer_reset = (data & 0x02) != 0;

                log::trace!(
                    "$4025 = 0x{:02X} → motor={} reset={} read={} irq_en={}",
                    data,
                    self.drive.motor_on,
                    self.drive.transfer_reset,
                    self.drive.mode_read,
                    self.drive.disk_irq_enabled
                );

                if !self.drive.disk_irq_enabled {
                    self.drive.disk_irq_pending = false;
                }
            }
            0x4040..=0x4092 => self.audio.write(addr, data),
            0x6000..=0x7FFF => self.prg_ram[(addr - 0x6000) as usize] = data,
            _ => {}
        }
    }

    fn irq_flag(&self) -> bool {
        self.timer_irq_pending || self.drive.disk_irq_pending
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn load_bios(&mut self, bios_data: &[u8]) {
        let len = bios_data.len().min(8192);
        self.bios[..len].copy_from_slice(&bios_data[..len]);
    }

    fn step_cpu(&mut self, cycles: u64) {
        if self.timer_enabled && self.disk_io_enabled {
            for _ in 0..cycles {
                if self.timer_counter == 0 {
                    self.timer_irq_pending = true;
                    self.timer_counter = self.timer_reload;
                    if !self.timer_repeat {
                        self.timer_enabled = false;
                    }
                } else {
                    self.timer_counter -= 1;
                }
            }
        }

        self.drive.step(cycles);
        self.audio.step(cycles);
    }
}
