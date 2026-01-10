use clap::Parser;
use std::path::PathBuf;

mod bus;
mod cpu;
mod logger;
mod memory;
mod ppu;
mod rom;

use bus::Bus;
use cpu::CPU;
use memory::RAM;
use rom::ROM;

use crate::ppu::PPU;

#[derive(Parser)]
#[command(name = "nes-emulator")]
#[command(about = "NES Emulator Core", long_about = None)]
struct Args {
    #[arg(help = "Path to NES ROM file")]
    rom: PathBuf,

    #[arg(short, long, help = "Enable verbose debug output")]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    logger::init_logger(args.verbose);

    let rom = ROM::load(&args.rom).expect("Failed to load ROM");
    log::info!("Loaded ROM: {}", args.rom.display());
    log::debug!("Mapper: {}", rom.mapper());

    let mut bus = Bus::new();
    let ram = RAM::new();
    let ppu = PPU::new(rom.chr_rom.to_vec());
    bus.attach_ram(ram);
    bus.attach_rom(rom);
    bus.attach_ppu(ppu);

    let mut cpu = CPU::new();
    cpu.reset(&mut bus);

    log::info!("Starting emulation...");

    loop {
        cpu.step(&mut bus);
    }
}
