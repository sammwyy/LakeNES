use clap::Parser;
use lakenes_core::NES;
use minifb::{Key, Window, WindowOptions};
use std::fs;
use std::path::PathBuf;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Observer, Producer, Split};

#[derive(Parser)]
struct Args {
    #[arg(help = "Path to NES ROM file")]
    rom: Option<PathBuf>,
    #[arg(short, long)]
    verbose: bool,
}

mod logger;

fn main() {
    let args = Args::parse();
    logger::init_logger(args.verbose);

    let mut nes = if let Some(rom_path) = args.rom {
        let rom_data = fs::read(rom_path).expect("Failed to read ROM file");
        Some(NES::new(&rom_data))
    } else {
        None
    };

    let mut window = Window::new(
        "LakeNES - Desktop",
        256,
        240,
        WindowOptions {
            scale: minifb::Scale::X2,
            ..WindowOptions::default()
        },
    )
    .expect("No se pudo crear la ventana");

    log::info!("Starting emulator window... (Use Ctrl+O to open a ROM)");

    // Audio Setup
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No audio device available");
    let config = device.default_output_config().expect("No audio config");
    let sample_rate = config.sample_rate() as f64;
    let channels = config.channels() as usize;

    let ring = HeapRb::<f32>::new(8192); // Ring buffer for audio samples
    let (mut producer, mut consumer) = ring.split();

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let sample = consumer.try_pop().unwrap_or(0.0);
                    for s in frame.iter_mut() {
                        *s = sample;
                    }
                }
            },
            move |err| eprintln!("Audio stream error: {}", err),
            None,
        )
        .expect("Failed to create audio stream");

    stream.play().expect("Failed to start audio stream");

    // Pre-calculate samples per cycle
    let samples_per_cycle: f64 = sample_rate / 1789773.0;
    let mut sample_accumulator = 0.0;
    let empty_buffer = vec![0u32; 256 * 240];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle ROM loading via hotkey Ctrl+O
        let is_ctrl = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
        if is_ctrl && window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            // Open ROM dialog
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("NES Rom", &["nes", "NES"])
                .pick_file()
            {
                match fs::read(&path) {
                    Ok(data) => {
                        nes = Some(NES::new(&data));
                        log::info!("Loaded ROM: {:?}", path);
                    }
                    Err(e) => log::error!("Failed to read ROM: {}", e),
                }
            }
        }

        if let Some(ref mut nes_instance) = nes {
            let mut joypad_bits = 0u8;

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

            nes_instance.update_joypad(1, joypad_bits);

            let mut cycles_this_frame = 0;
            while cycles_this_frame < 29780 {
                let cpu_cycles = nes_instance.cpu.step(&mut nes_instance.bus);
                cycles_this_frame += cpu_cycles;

                for _ in 0..(cpu_cycles * 3) {
                    if let Some(ref mut ppu) = nes_instance.bus.ppu {
                        if let Some(ref mut rom) = nes_instance.bus.rom {
                            ppu.step(&mut *rom.mapper);
                        }
                    }
                }

                for _ in 0..cpu_cycles {
                    if let Some(ref mut apu) = nes_instance.bus.apu {
                        apu.step();
                        sample_accumulator += samples_per_cycle;
                        if sample_accumulator >= 1.0 {
                            sample_accumulator -= 1.0;
                            let sample = apu.output_sample() * 0.4;
                            while producer.vacant_len() == 0 {
                                std::thread::sleep(std::time::Duration::from_micros(10));
                            }
                            let _ = producer.try_push(sample);
                        }
                    }
                }

                nes_instance.bus.check_ppu_nmi();
                nes_instance.bus.check_mapper_irq();
            }

            window
                .update_with_buffer(nes_instance.get_frame_buffer(), 256, 240)
                .unwrap();
        } else {
            // No NES loaded, just update window with black buffer
            window.update_with_buffer(&empty_buffer, 256, 240).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}
