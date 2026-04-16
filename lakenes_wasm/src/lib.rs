use lakenes_core::NES;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmNes {
    nes: NES,
}

#[wasm_bindgen]
impl WasmNes {
    pub fn new(rom_data: &[u8]) -> Self {
        console_error_panic_hook::set_once();
        Self {
            nes: NES::new(rom_data),
        }
    }

    pub fn step_frame(&mut self) {
        self.nes.step_frame();
    }

    pub fn step_cycle(&mut self) -> u64 {
        self.nes.step_cycle()
    }

    pub fn get_frame_buffer_ptr(&self) -> *const u32 {
        self.nes.get_frame_buffer().as_ptr()
    }

    pub fn update_joypad(&mut self, player: u8, buttons: u8) {
        self.nes.update_joypad(player, buttons);
    }

    pub fn set_audio_sample_rate(&mut self, sample_rate: f64) {
        self.nes.set_audio_sample_rate(sample_rate);
    }

    pub fn get_audio_sample(&mut self) -> f32 {
        self.nes.get_audio_sample()
    }

    pub fn audio_buffer_len(&self) -> usize {
        self.nes.audio_buffer_len()
    }

    pub fn set_speed(&mut self, percent: u32) {
        self.nes.set_speed(percent);
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
        self.nes
            .set_apu_volumes(master, pulse1, pulse2, triangle, noise, dmc);
    }

    pub fn get_ppu_mask(&self) -> u8 {
        self.nes.get_ppu_mask()
    }

    pub fn write_ppu_mask(&mut self, mask: u8) {
        self.nes.write_ppu_mask(mask);
    }

    pub fn set_ppu_mask_override(&mut self, mask: Option<u8>) {
        self.nes.set_ppu_mask_override(mask);
    }

    pub fn get_rom_mapper_id(&self) -> u16 {
        self.nes.get_rom_mapper_id()
    }

    pub fn get_rom_prg_size(&self) -> usize {
        self.nes.get_rom_prg_size()
    }

    pub fn get_rom_chr_size(&self) -> usize {
        self.nes.get_rom_chr_size()
    }

    pub fn get_total_frames(&self) -> u64 {
        self.nes.total_frames()
    }

    pub fn get_total_cycles(&self) -> u64 {
        self.nes.total_cycles()
    }

    pub fn get_ppu_palette(&self) -> Vec<u32> {
        self.nes.get_ppu_palette().to_vec()
    }

    pub fn get_ppu_oam(&self) -> Vec<u8> {
        self.nes.get_ppu_oam().to_vec()
    }

    pub fn get_pattern_table(&mut self, table_idx: u8) -> Vec<u8> {
        self.nes.get_pattern_table(table_idx)
    }

    pub fn get_apu_channels_state(&self) -> Vec<f32> {
        self.nes.get_apu_channels_state()
    }

    pub fn get_cpu_registers(&self) -> Vec<u32> {
        let (pc, a, x, y, sp, p) = self.nes.get_cpu_registers();
        vec![pc as u32, a as u32, x as u32, y as u32, sp as u32, p as u32]
    }

    pub fn step_instruction(&mut self) {
        self.nes.step_instruction();
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.nes.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.nes.paused
    }

    pub fn disassemble(&mut self, addr: u16) -> String {
        let rom = self.nes.get_prg_rom();
        // Since get_prg_rom returns data starting at 0x8000, we need to adjust addr
        if addr >= 0x8000 {
            let offset = addr - 0x8000;
            let (text, next) = lakenes_core::cpu::disasm::disassemble(&rom, offset);
            format!("{:04X}|{}", (next + 0x8000), text)
        } else {
            format!("{:04X}|RAM ${:04X}", (addr + 1), addr)
        }
    }
}
