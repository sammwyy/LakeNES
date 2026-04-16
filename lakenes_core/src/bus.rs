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

    /// PPU dots (1 dot = 1 `ppu.step`) advanced by `read_cpu`/`write_cpu` this instruction.
    ppu_dots_from_cpu_memory: u32,
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
            ppu_dots_from_cpu_memory: 0,
        }
    }

    /// Run 3 PPU dots per CPU memory cycle (NTSC). Used for CPU↔PPU alignment.
    fn run_ppu_three_dots(&mut self) {
        if let (Some(ppu), Some(rom)) = (self.ppu.as_mut(), self.rom.as_mut()) {
            ppu.step_many(3, &mut *rom.mapper);
        }
        self.ppu_dots_from_cpu_memory = self.ppu_dots_from_cpu_memory.saturating_add(3);
    }

    pub fn begin_cpu_instruction(&mut self) {
        self.ppu_dots_from_cpu_memory = 0;
    }

    /// After each CPU memory access (except during reset/DMA helpers that use `read` only).
    pub fn read_cpu(&mut self, addr: u16) -> u8 {
        let v = self.read(addr);
        self.run_ppu_three_dots();
        v
    }

    /// `cpu_cycles` is the CPU's cumulative cycle counter at the moment of the write
    /// (used for OAM DMA513 vs514 timing).
    pub fn write_cpu(&mut self, addr: u16, value: u8, cpu_cycles: u64) {
        self.write(addr, value, cpu_cycles);
        self.run_ppu_three_dots();
    }

    /// Remaining PPU dots for this instruction (dummy cycles, implied ops, etc.).
    pub fn ppu_end_instruction_catch_up(&mut self, cpu_cycles: u64) {
        let want = (cpu_cycles as u32).saturating_mul(3);
        let got = self.ppu_dots_from_cpu_memory;
        let need = want.saturating_sub(got);
        self.ppu_dots_from_cpu_memory = 0;
        if need > 0 {
            if let (Some(ppu), Some(rom)) = (self.ppu.as_mut(), self.rom.as_mut()) {
                ppu.step_many(need as usize, &mut *rom.mapper);
            }
        }
    }

    /// Advance PPU without a CPU memory access (e.g. CPU reset vector reads).
    pub fn advance_ppu_dots(&mut self, dots: usize) {
        if dots == 0 {
            return;
        }
        if let (Some(ppu), Some(rom)) = (self.ppu.as_mut(), self.rom.as_mut()) {
            ppu.step_many(dots, &mut *rom.mapper);
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
                    // Bit 5 is not driven by the APU; comes from open bus (last value on data bus).
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
            // Unallocated CPU I/O ($4018–$401F): nothing drives the bus.
            0x4018..=0x401F => self.cpu_data_bus,
            0x4020..=0x7FFF => {
                if let Some(ref mut rom) = self.rom {
                    rom.mapper.read_ex(addr)
                } else {
                    self.cpu_data_bus
                }
            }
            0x8000..=0xFFFF => {
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

    pub fn write(&mut self, addr: u16, value: u8, cpu_cycles: u64) {
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
                // OAM DMA: 513 CPU cycles if the write ends on an even cycle, 514 if odd (NTSC).
                let stall = 513 + (cpu_cycles & 1);
                self.add_cpu_stall_cycles(stall);
            }
            0x4016 => {
                if let Some(ref mut joypad) = self.joypad1 {
                    joypad.write(value);
                }
            }
            0x4020..=0x7FFF => {
                if let Some(ref mut rom) = self.rom {
                    rom.mapper.write_ex(addr, value);
                }
            }
            0x8000..=0xFFFF => {
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

    pub fn reset(&mut self, hard: bool) {
        if hard {
            if let Some(ref mut ram) = self.ram {
                *ram = RAM::new();
            }
            if let Some(ref mut ppu) = self.ppu {
                *ppu = PPU::new();
            }
            if let Some(ref mut apu) = self.apu {
                *apu = APU::new();
            }
        }

        if let Some(ref mut rom) = self.rom {
            rom.mapper.reset();
        }

        self.nmi_pending = false;
        self.irq_pending = false;
        self.cpu_data_bus = 0;
        self.cpu_stall_cycles = 0;
    }
}
