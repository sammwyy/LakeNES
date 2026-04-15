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
use alloc::collections::VecDeque;
use bus::Bus;
use cpu::CPU;
use joypad::Joypad;
use memory::RAM;
use ppu::PPU;
use rom::ROM;

pub struct NES {
    pub cpu: CPU,
    pub bus: Bus,
    audio_sample_rate: f64,
    audio_phase: f64,
    audio_buffer: VecDeque<f32>,
    last_audio_sample: f32,
    audio_gain: f32,
    master_cpu_cycles: u64,
    master_ppu_cycles: u64,
    master_apu_cycles: u64,
    /// Emulated-time scale for frame pacing (100 = ~60 FPS NES, 200 = 2×, 50 = ½×).
    emulation_speed_percent: u32,
}

impl NES {
    const CPU_FREQUENCY_HZ: f64 = 1_789_772.7272;
    const AUDIO_BUFFER_CAPACITY: usize = 16_384;
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

        Self {
            cpu,
            bus,
            audio_sample_rate: 44_100.0,
            audio_phase: 0.0,
            audio_buffer: VecDeque::with_capacity(Self::AUDIO_BUFFER_CAPACITY),
            last_audio_sample: 0.0,
            audio_gain: 0.4,
            master_cpu_cycles: 0,
            master_ppu_cycles: 0,
            master_apu_cycles: 0,
            emulation_speed_percent: 100,
        }
    }

    /// Set emulation speed for real-time pacing. `100` = normal, `200` = double, `50` = half, etc.
    pub fn set_speed(&mut self, percent: u32) {
        self.emulation_speed_percent = percent.clamp(1, 10_000);
    }

    pub fn speed_percent(&self) -> u32 {
        self.emulation_speed_percent
    }

    pub fn set_audio_sample_rate(&mut self, sample_rate: f64) {
        if sample_rate > 0.0 {
            self.audio_sample_rate = sample_rate;
            if let Some(ref mut apu) = self.bus.apu {
                apu.set_output_sample_rate(sample_rate as f32);
            }
        }
    }

    pub fn set_audio_gain(&mut self, gain: f32) {
        self.audio_gain = gain;
    }

    pub fn step_cycle(&mut self) -> u64 {
        let mut cpu_cycles = self.cpu.step(&mut self.bus);
        let dma_stall = self.bus.take_cpu_stall_cycles();
        cpu_cycles = cpu_cycles.saturating_add(dma_stall);

        if dma_stall > 0 {
            if let (Some(ppu), Some(rom)) = (self.bus.ppu.as_mut(), self.bus.rom.as_mut()) {
                ppu.step_many((dma_stall * 3) as usize, &mut *rom.mapper);
            }
        }

        let target_cpu = self.master_cpu_cycles.saturating_add(cpu_cycles);
        self.master_cpu_cycles = target_cpu;
        self.master_ppu_cycles = target_cpu.saturating_mul(3);

        let mut apu = self.bus.apu.take();
        if let Some(ref mut apu_ref) = apu {
            for _ in 0..cpu_cycles {
                apu_ref.step(|addr| self.bus.read(addr));
                self.master_apu_cycles = self.master_apu_cycles.saturating_add(1);

                self.audio_phase += self.audio_sample_rate;
                while self.audio_phase >= Self::CPU_FREQUENCY_HZ {
                    self.audio_phase -= Self::CPU_FREQUENCY_HZ;
                    let sample = apu_ref.output_sample() * self.audio_gain;
                    if self.audio_buffer.len() >= Self::AUDIO_BUFFER_CAPACITY {
                        self.audio_buffer.pop_front();
                    }
                    self.audio_buffer.push_back(sample);
                }
            }
            let dmc_stall = apu_ref.take_dmc_cpu_stall_cycles();
            if dmc_stall > 0 {
                self.bus.add_cpu_stall_cycles(dmc_stall);
            }
        }
        self.bus.apu = apu;

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
        if let Some(sample) = self.audio_buffer.pop_front() {
            self.last_audio_sample = sample;
            sample
        } else {
            self.last_audio_sample
        }
    }

    pub fn audio_buffer_len(&self) -> usize {
        self.audio_buffer.len()
    }

    pub fn audio_buffer_capacity(&self) -> usize {
        Self::AUDIO_BUFFER_CAPACITY
    }
}
