use crate::joypad::Joypad;
use crate::memory::RAM;
use crate::ppu::PPU;
use crate::rom::ROM;

pub trait BusDevice {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub struct Bus {
    // Internal devices
    ram: Option<RAM>,
    rom: Option<ROM>,
    pub ppu: Option<PPU>,

    // External devices
    pub joypad1: Option<Joypad>,
    pub joypad2: Option<Joypad>,

    // Status
    nmi_pending: bool,
    irq_pending: bool,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ram: None,
            rom: None,
            ppu: None,
            joypad1: None,
            joypad2: None,
            nmi_pending: false,
            irq_pending: false,
        }
    }

    pub fn attach_ram(&mut self, ram: RAM) {
        self.ram = Some(ram);
    }

    pub fn attach_rom(&mut self, rom: ROM) {
        self.rom = Some(rom);
    }

    pub fn attach_ppu(&mut self, ppu: PPU) {
        self.ppu = Some(ppu);
    }

    pub fn attach_joypad(&mut self, joypad: Joypad, port: u8) {
        match port {
            1 => self.joypad1 = Some(joypad),
            2 => self.joypad2 = Some(joypad),
            _ => log::warn!("Invalid joypad port: {}", port),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let mirrored = addr & 0x07FF;
                self.ram.as_mut().unwrap().read(mirrored)
            }
            0x2000..=0x3FFF => {
                if let Some(ref mut ppu) = self.ppu {
                    ppu.read(addr)
                } else {
                    0
                }
            }
            0x4016 => {
                if let Some(ref mut joypad) = self.joypad1 {
                    joypad.read()
                } else {
                    0
                }
            }
            0x8000..=0xFFFF => self.rom.as_ref().unwrap().read(addr),
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let mirrored = addr & 0x07FF;
                self.ram.as_mut().unwrap().write(mirrored, value);
            }
            0x2000..=0x3FFF => {
                if let Some(ref mut ppu) = self.ppu {
                    ppu.write(addr, value)
                }
            }
            0x4014 => {
                let page = (value as u16) << 8;
                for i in 0..256 {
                    let data = self.read(page + i as u16);
                    if let Some(ref mut ppu) = self.ppu {
                        ppu.oam_data[ppu.oam_addr as usize] = data;
                        ppu.oam_addr = ppu.oam_addr.wrapping_add(1);
                    }
                }
            }
            0x4016 => {
                if let Some(ref mut joypad) = self.joypad1 {
                    joypad.write(value);
                }
            }
            0x8000..=0xFFFF => {
                log::debug!("Attempted write to ROM at 0x{:04X}", addr);
            }
            _ => {
                log::debug!("Unimplemented write at 0x{:04X} = 0x{:02X}", addr, value);
            }
        }
    }

    pub fn trigger_nmi(&mut self) {
        self.nmi_pending = true;
    }

    pub fn trigger_irq(&mut self) {
        self.irq_pending = true;
    }

    pub fn poll_nmi(&mut self) -> bool {
        let pending = self.nmi_pending;
        self.nmi_pending = false;
        pending
    }

    pub fn check_ppu_nmi(&mut self) {
        let mut nmi_to_trigger = false;
        if let Some(ref mut ppu) = self.ppu {
            if ppu.nmi_interrupt {
                ppu.nmi_interrupt = false;
                nmi_to_trigger = true;
            }
        }

        if nmi_to_trigger {
            self.trigger_nmi();
        }
    }

    pub fn poll_irq(&mut self) -> bool {
        let pending = self.irq_pending;
        self.irq_pending = false;
        pending
    }
}
