use crate::apu::APU;
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
    pub rom: Option<ROM>,
    pub ppu: Option<PPU>,
    pub apu: Option<APU>,

    // External devices
    pub joypad1: Option<Joypad>,
    pub joypad2: Option<Joypad>,

    /// Last value on the CPU data bus (open bus). Write-only APU ports,
    /// CPU space $4018–$40FF (no chip select), and similar reads expose this.
    cpu_data_bus: u8,

    // Status
    nmi_pending: bool,
    irq_pending: bool,
    cpu_stall_cycles: u64,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ram: None,
            rom: None,
            ppu: None,
            apu: None,
            joypad1: None,
            joypad2: None,
            cpu_data_bus: 0,
            nmi_pending: false,
            irq_pending: false,
            cpu_stall_cycles: 0,
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

    pub fn attach_apu(&mut self, apu: APU) {
        self.apu = Some(apu);
    }

    pub fn attach_joypad(&mut self, joypad: Joypad, port: u8) {
        match port {
            1 => self.joypad1 = Some(joypad),
            2 => self.joypad2 = Some(joypad),
            _ => log::warn!("Invalid joypad port: {}", port),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let value = match addr {
            0x0000..=0x1FFF => {
                let mirrored = addr & 0x07FF;
                self.ram.as_mut().unwrap().read(mirrored)
            }
            0x2000..=0x3FFF => {
                if let Some(ppu) = &mut self.ppu {
                    if let Some(rom) = &mut self.rom {
                        ppu.read(addr, &mut *rom.mapper)
                    } else {
                        self.cpu_data_bus
                    }
                } else {
                    self.cpu_data_bus
                }
            }
            // APU write-only registers ($4000–$4013) and $4014: bus not driven.
            0x4000..=0x4014 => self.cpu_data_bus,
            0x4015 => {
                if let Some(ref mut apu) = self.apu {
                    // Bit 5 is not driven by the APU; comes from open bus.
                    (apu.read(addr) & 0xDF) | (self.cpu_data_bus & 0x20)
                } else {
                    self.cpu_data_bus
                }
            }
            0x4016 => {
                if let Some(ref mut joypad) = self.joypad1 {
                    (joypad.read() & 0x01) | (self.cpu_data_bus & 0xFE)
                } else {
                    self.cpu_data_bus
                }
            }
            0x4017 => {
                if let Some(ref mut joypad) = self.joypad2 {
                    (joypad.read() & 0x01) | (self.cpu_data_bus & 0xFE)
                } else {
                    // No controller: full open bus (forcing D0 high breaks cpu_exec_space tests).
                    self.cpu_data_bus
                }
            }
            // Unallocated CPU I/O ($4018–$40FF): nothing drives the bus (nesdev + cpu_exec_space).
            0x4018..=0x40FF => self.cpu_data_bus,
            0x4100..=0xFFFF => {
                if let Some(ref mut rom) = self.rom {
                    rom.mapper.read_prg(addr)
                } else {
                    self.cpu_data_bus
                }
            }
        };
        self.cpu_data_bus = value;
        value
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.cpu_data_bus = value;
        match addr {
            0x0000..=0x1FFF => {
                let mirrored = addr & 0x07FF;
                self.ram.as_mut().unwrap().write(mirrored, value);
            }
            0x2000..=0x3FFF => {
                if let Some(ppu) = &mut self.ppu {
                    if let Some(rom) = &mut self.rom {
                        ppu.write(addr, value, &mut *rom.mapper)
                    }
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
                // OAM DMA stalls CPU for 513 or 514 cycles depending on parity.
                // We model 513 here; parity refinement can be layered on top later.
                self.add_cpu_stall_cycles(513);
            }
            0x4016 => {
                if let Some(ref mut joypad) = self.joypad1 {
                    joypad.write(value);
                }
            }
            0x4020..=0xFFFF => {
                if let Some(ref mut rom) = self.rom {
                    rom.mapper.write_prg(addr, value);
                }
            }
            0x4000..=0x4013 | 0x4015 | 0x4017 => {
                if let Some(ref mut apu) = self.apu {
                    apu.write(addr, value);
                }
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

    pub fn check_mapper_irq(&mut self) {
        let mut irq_active = false;
        if let Some(ref rom) = self.rom {
            if rom.mapper.irq_flag() {
                irq_active = true;
            }
        }
        // Check APU IRQs (DMC end-of-sample + frame counter)
        if let Some(ref apu) = self.apu {
            if apu.irq_active() {
                irq_active = true;
            }
        }
        if irq_active {
            self.trigger_irq();
        }
    }

    pub fn poll_irq(&mut self) -> bool {
        let pending = self.irq_pending;
        self.irq_pending = false;
        pending
    }

    pub fn add_cpu_stall_cycles(&mut self, cycles: u64) {
        self.cpu_stall_cycles = self.cpu_stall_cycles.saturating_add(cycles);
    }

    pub fn take_cpu_stall_cycles(&mut self) -> u64 {
        let stall = self.cpu_stall_cycles;
        self.cpu_stall_cycles = 0;
        stall
    }
}
