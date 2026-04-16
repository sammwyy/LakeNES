#![no_std]

extern crate alloc;

pub mod apu;
pub mod bus;
pub mod cpu;
pub mod joypad;
pub mod memory;
pub mod ppu;
pub mod rom;

use alloc::collections::VecDeque;
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
    audio_sample_rate: f64,
    audio_phase: f64,
    audio_buffer: VecDeque<f32>,
    last_audio_sample: f32,
    master_cpu_cycles: u64,
    master_ppu_cycles: u64,
    master_apu_cycles: u64,
    /// Emulated-time scale for frame pacing (100 = ~60 FPS NES, 200 = 2×, 50 = ½×).
    emulation_speed_percent: u32,
    total_frames: u64,
    total_cycles: u64,
    pub paused: bool,
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
            master_cpu_cycles: 0,
            master_ppu_cycles: 0,
            master_apu_cycles: 0,
            emulation_speed_percent: 100,
            total_frames: 0,
            total_cycles: 0,
            paused: false,
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

    pub fn set_apu_volumes(
        &mut self,
        master: f32,
        pulse1: f32,
        pulse2: f32,
        triangle: f32,
        noise: f32,
        dmc: f32,
    ) {
        if let Some(ref mut apu) = self.bus.apu {
            apu.set_volumes(master, pulse1, pulse2, triangle, noise, dmc);
        }
    }

    pub fn soft_reset(&mut self) {
        self.bus.reset(false);
        self.cpu.reset(&mut self.bus);
        self.master_cpu_cycles = 0;
        self.master_ppu_cycles = 0;
        self.master_apu_cycles = 0;
        self.total_cycles = 0;
        log::info!("Performed soft reset");
    }

    pub fn hard_reset(&mut self) {
        self.bus.reset(true);
        self.cpu = CPU::new();
        self.cpu.reset(&mut self.bus);
        self.master_cpu_cycles = 0;
        self.master_ppu_cycles = 0;
        self.master_apu_cycles = 0;
        self.total_cycles = 0;
        self.total_frames = 0;
        log::info!("Performed hard reset");
    }

    pub fn step_cycle(&mut self) -> u64 {
        let mut cpu_cycles = self.cpu.step(&mut self.bus);

        // Capture any DMC stalls generated during APU steps from the previous instruction's burst.
        // We add them to the current instruction's total cycle duration.
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
                    let sample = apu_ref.output_sample();
                    if self.audio_buffer.len() >= Self::AUDIO_BUFFER_CAPACITY {
                        self.audio_buffer.pop_front();
                    }
                    self.audio_buffer.push_back(sample);
                }
            }
            // If the APU steps triggered a DMC fetch, add those cycles to the stall pool.
            let dmc_stall = apu_ref.take_dmc_cpu_stall_cycles();
            if dmc_stall > 0 {
                self.bus.add_cpu_stall_cycles(dmc_stall);
            }
        }
        self.bus.apu = apu;

        self.bus.check_ppu_nmi();
        self.bus.check_mapper_irq();

        self.total_cycles = self.total_cycles.saturating_add(cpu_cycles);
        cpu_cycles
    }

    pub fn step_frame(&mut self) {
        if self.paused {
            return;
        }
        let mut cycles = 0u64;
        let target = (29780 * self.emulation_speed_percent as u64) / 100;
        while cycles < target {
            cycles += self.step_cycle();
        }
        self.total_frames = self.total_frames.saturating_add(1);
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

    pub fn get_ppu_mask(&self) -> u8 {
        self.bus.ppu.as_ref().map(|p| p.mask_bits()).unwrap_or(0)
    }

    pub fn write_ppu_mask(&mut self, mask: u8) {
        if let Some(ref mut ppu) = self.bus.ppu {
            ppu.write_mask(mask);
        }
    }

    pub fn set_ppu_mask_override(&mut self, mask: Option<u8>) {
        if let Some(ref mut ppu) = self.bus.ppu {
            ppu.set_mask_override(mask);
        }
    }

    pub fn get_rom_mapper_id(&self) -> u16 {
        self.bus.rom.as_ref().map(|r| r.mapper_id).unwrap_or(0)
    }

    pub fn get_rom_prg_size(&self) -> usize {
        self.bus.rom.as_ref().map(|r| r.prg_size).unwrap_or(0)
    }

    pub fn get_rom_chr_size(&self) -> usize {
        self.bus.rom.as_ref().map(|r| r.chr_size).unwrap_or(0)
    }

    pub fn get_prg_rom(&mut self) -> alloc::vec::Vec<u8> {
        if self.bus.rom.is_some() {
            let mut data = alloc::vec![0u8; 0x10000];
            for addr in 0x8000..=0xFFFF {
                data[addr as usize - 0x8000] = self.bus.read(addr);
            }
            data
        } else {
            alloc::vec![]
        }
    }

    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }

    pub fn get_ppu_palette(&self) -> [u32; 32] {
        let mut palette = [0u32; 32];
        if let Some(ppu) = self.bus.ppu.as_ref() {
            for i in 0..32 {
                let color_idx = ppu.palette_table[i] & 0x3F;
                palette[i] = ppu::NES_PALETTE[color_idx as usize];
            }
        }
        palette
    }

    pub fn get_ppu_oam(&self) -> [u8; 256] {
        self.bus
            .ppu
            .as_ref()
            .map(|p| p.oam_data)
            .unwrap_or([0; 256])
    }

    pub fn get_pattern_table(&mut self, table_idx: u8) -> alloc::vec::Vec<u8> {
        let mut pixels = alloc::vec![0u8; 128 * 128];
        if let (Some(ppu), Some(rom)) = (self.bus.ppu.as_mut(), self.bus.rom.as_mut()) {
            for tile_y in 0..16 {
                for tile_x in 0..16 {
                    let tile_idx = tile_y * 16 + tile_x;
                    let table_addr = (table_idx as u16) * 0x1000;
                    let tile_addr = table_addr + (tile_idx as u16) * 16;

                    for row in 0..8 {
                        let lo = ppu.ppu_read_debug(tile_addr + row as u16, &mut *rom.mapper);
                        let hi = ppu.ppu_read_debug(tile_addr + row as u16 + 8, &mut *rom.mapper);

                        for col in 0..8 {
                            let bit0 = (lo >> (7 - col)) & 1;
                            let bit1 = (hi >> (7 - col)) & 1;
                            let color_idx = (bit1 << 1) | bit0;

                            let x = tile_x * 8 + col;
                            let y = tile_y * 8 + row;
                            pixels[y * 128 + x] = color_idx;
                        }
                    }
                }
            }
        }
        pixels
    }

    pub fn get_apu_channels_state(&self) -> alloc::vec::Vec<f32> {
        let mut states = alloc::vec![0.0f32; 6]; // P1, P2, Tri, Noise, DMC, Master
        if let Some(ref apu) = self.bus.apu {
            states[0] = apu.pulse1.output() as f32 / 15.0;
            states[1] = apu.pulse2.output() as f32 / 15.0;
            states[2] = apu.triangle.output() as f32 / 15.0;
            states[3] = apu.noise.output() as f32 / 15.0;
            states[4] = apu.dmc.output_level as f32 / 127.0;
            // Approximate master level from last output
            states[5] = self.last_audio_sample.abs();
        }
        states
    }

    pub fn get_cpu_registers(&self) -> (u16, u8, u8, u8, u8, u8) {
        (
            self.cpu.pc,
            self.cpu.a,
            self.cpu.x,
            self.cpu.y,
            self.cpu.sp,
            self.cpu.p,
        )
    }

    pub fn step_instruction(&mut self) {
        let cycles_before = self.total_cycles;
        let mut cycles_to_add = self.cpu.step(&mut self.bus);
        let dma_stall = self.bus.take_cpu_stall_cycles();
        cycles_to_add = cycles_to_add.saturating_add(dma_stall);

        // Catch up other components
        self.master_cpu_cycles = self.master_cpu_cycles.saturating_add(cycles_to_add);
        self.total_cycles = cycles_before.saturating_add(cycles_to_add);
        // Step PPU/APU as needed (simplified for-loop)
        if let (Some(ppu), Some(rom)) = (self.bus.ppu.as_mut(), self.bus.rom.as_mut()) {
            ppu.step_many((cycles_to_add * 3) as usize, &mut *rom.mapper);
        }
        // ... apu stepping would go here but it's complex because of the phase accumulator
        // For debugging, we just care about CPU/PPU mostly.
    }
}
