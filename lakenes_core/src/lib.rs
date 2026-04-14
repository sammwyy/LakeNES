#![no_std]

extern crate alloc;

pub mod apu;
pub mod bus;
pub mod cpu;
pub mod joypad;
pub mod memory;
pub mod ppu;
pub mod rom;

use apu::APU;
use bus::Bus;
use cpu::CPU;
use joypad::Joypad;
use memory::RAM;
use ppu::PPU;
use rom::ROM;

pub struct NES {
    pub cpu: CPU,
    pub bus: Bus,
}

impl NES {
    pub fn new(rom_data: &[u8]) -> Self {
        let rom = ROM::load_from_bytes(rom_data).expect("Failed to load ROM");
        let mut bus = Bus::new();
        bus.attach_ram(RAM::new());
        bus.attach_rom(rom);
        bus.attach_ppu(PPU::new());
        bus.attach_apu(APU::new());
        bus.attach_joypad(Joypad::new(), 1);

        let mut cpu = CPU::new();
        cpu.reset(&mut bus);

        Self { cpu, bus }
    }

    pub fn step_cycle(&mut self) -> u64 {
        let cpu_cycles = self.cpu.step(&mut self.bus);

        for _ in 0..(cpu_cycles * 3) {
            if let Some(ref mut ppu) = self.bus.ppu {
                if let Some(ref mut rom) = self.bus.rom {
                    ppu.step(&mut *rom.mapper);
                }
            }
        }

        for _ in 0..cpu_cycles {
            let mut apu = self.bus.apu.take();
            if let Some(ref mut apu_ref) = apu {
                apu_ref.step(|addr| self.bus.read(addr));
            }
            self.bus.apu = apu;
        }

        self.bus.check_ppu_nmi();
        self.bus.check_mapper_irq();

        cpu_cycles
    }

    pub fn step_frame(&mut self) {
        let mut cycles = 0u64;
        while cycles < 29780 {
            cycles += self.step_cycle();
        }
    }

    pub fn get_frame_buffer(&self) -> &[u32] {
        &self.bus.ppu.as_ref().unwrap().frame_buffer
    }

    pub fn update_joypad(&mut self, player: u8, buttons: u8) {
        if player == 1 {
            if let Some(ref mut joypad) = self.bus.joypad1 {
                joypad.update(buttons);
            }
        }
    }

    pub fn get_audio_sample(&mut self) -> f32 {
        if let Some(ref mut apu) = self.bus.apu {
            apu.output_sample()
        } else {
            0.0
        }
    }
}
