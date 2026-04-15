use lakenes_core::NES;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MAX_FRAMES: usize = 600;

pub fn roms_root() -> Option<PathBuf> {
    std::env::var("LAKENES_TEST_ROMS_DIR")
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.exists())
}

pub fn load_nes_from_relative_rom(relative_path: &str) -> Option<NES> {
    let root = roms_root()?;
    let rom_path = root.join(relative_path);
    let rom_data = fs::read(rom_path).ok()?;
    Some(NES::new(&rom_data))
}

pub fn run_frames(nes: &mut NES, frames: usize) {
    for _ in 0..frames {
        nes.step_frame();
    }
}

pub fn run_until_ram_value(
    nes: &mut NES,
    addr: u16,
    expected: u8,
    max_frames: usize,
) -> Result<(), String> {
    for frame in 0..max_frames {
        nes.step_frame();
        let value = nes.bus.read(addr);
        if value == expected {
            return Ok(());
        }
        if frame % 60 == 0 {
            nes.bus.check_ppu_nmi();
            nes.bus.check_mapper_irq();
        }
    }
    let got = nes.bus.read(addr);
    Err(format!(
        "Timeout waiting RAM[0x{addr:04X}] == 0x{expected:02X}, got 0x{got:02X}"
    ))
}

pub fn run_until_prg_ram_signature(
    nes: &mut NES,
    start_addr: u16,
    expected_ascii: &str,
    max_frames: usize,
) -> Result<(), String> {
    let bytes = expected_ascii.as_bytes();
    if bytes.is_empty() {
        return Err("Expected signature must not be empty".to_string());
    }

    for _ in 0..max_frames {
        nes.step_frame();
        let mut matched = true;
        for (i, expected) in bytes.iter().enumerate() {
            let addr = start_addr + i as u16;
            if nes.bus.read(addr) != *expected {
                matched = false;
                break;
            }
        }
        if matched {
            return Ok(());
        }
    }
    Err(format!(
        "Timeout waiting signature '{}' at 0x{start_addr:04X}",
        expected_ascii
    ))
}

pub fn require_rom(relative_path: &str) -> Option<String> {
    let Some(root) = roms_root() else {
        return Some("LAKENES_TEST_ROMS_DIR is not set or path does not exist".to_string());
    };
    let full = root.join(relative_path);
    if full.exists() {
        None
    } else {
        Some(format!(
            "ROM not found: {} (set LAKENES_TEST_ROMS_DIR)",
            full.display()
        ))
    }
}

pub fn rom_exists<P: AsRef<Path>>(relative_path: P) -> bool {
    if let Some(root) = roms_root() {
        root.join(relative_path).exists()
    } else {
        false
    }
}
