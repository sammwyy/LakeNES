use clap::Parser;
use minifb::{Key, Window, WindowOptions};
use std::path::PathBuf;

mod bus;
mod cpu;
mod joypad;
mod logger;
mod memory;
mod ppu;
mod rom;

use bus::Bus;
use cpu::CPU;
use memory::RAM;
use ppu::PPU;
use rom::ROM;

use crate::joypad::Joypad;

#[derive(Parser)]
struct Args {
    #[arg(help = "Path to NES ROM file")]
    rom: PathBuf,
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    logger::init_logger(args.verbose);

    let rom = ROM::load(&args.rom).expect("Failed to load ROM");
    let mut bus = Bus::new();
    let joypad = Joypad::new();

    let chr_rom_copy = rom.chr_rom.to_vec();

    bus.attach_ram(RAM::new());
    bus.attach_rom(rom);
    bus.attach_ppu(PPU::new(chr_rom_copy));
    bus.attach_joypad(joypad, 1);

    let mut cpu = CPU::new();
    cpu.reset(&mut bus);

    let mut window = Window::new(
        "Sammwy - PPU Debug",
        256,
        240,
        WindowOptions {
            scale: minifb::Scale::X2,
            ..WindowOptions::default()
        },
    )
    .expect("No se pudo crear la ventana");

    // Limit FPS
    window.limit_update_rate(Some(std::time::Duration::from_micros(16666)));

    log::info!("Starting emulation...");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut joypad_bits = 0u8;
        if window.is_key_down(Key::P) {
            if let Some(ref mut ppu) = bus.ppu {
                ppu.scroll_x = ppu.scroll_x.wrapping_add(1); // Test manual
            }
        }

        if window.is_key_down(Key::Z) {
            joypad_bits |= 0b00000001;
        } // A
        if window.is_key_down(Key::X) {
            joypad_bits |= 0b00000010;
        } // B
        if window.is_key_down(Key::Space) {
            joypad_bits |= 0b00000100;
        } // Select
        if window.is_key_down(Key::Enter) {
            joypad_bits |= 0b00001000;
        } // Start
        if window.is_key_down(Key::Up) {
            joypad_bits |= 0b00010000;
        } // Up
        if window.is_key_down(Key::Down) {
            joypad_bits |= 0b00100000;
        } // Down
        if window.is_key_down(Key::Left) {
            joypad_bits |= 0b01000000;
        } // Left
        if window.is_key_down(Key::Right) {
            joypad_bits |= 0b10000000;
        } // Right

        if let Some(ref mut joypad1) = bus.joypad1 {
            joypad1.update(joypad_bits);
        }

        // ~1.79MHz, / 60fps = ~29780 (NTSC)
        let mut cycles_this_frame = 0;
        while cycles_this_frame < 29780 {
            let cpu_cycles = cpu.step(&mut bus);
            cycles_this_frame += cpu_cycles;

            for _ in 0..(cpu_cycles * 3) {
                if let Some(ref mut ppu) = bus.ppu {
                    ppu.step();
                }
            }

            bus.check_ppu_nmi();
        }

        if let Some(ref mut ppu) = bus.ppu {
            // ppu.render_sprites();

            let mut buffer_u32 = vec![0u32; 256 * 240];
            for i in 0..(256 * 240) {
                let r = ppu.frame_buffer[i * 3] as u32;
                let g = ppu.frame_buffer[i * 3 + 1] as u32;
                let b = ppu.frame_buffer[i * 3 + 2] as u32;
                buffer_u32[i] = (r << 16) | (g << 8) | b;
            }

            window.update_with_buffer(&buffer_u32, 256, 240).unwrap();
        } else {
            window.update();
        }
    }
}
