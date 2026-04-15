mod common;

use common::{DEFAULT_MAX_FRAMES, load_nes_from_relative_rom, require_rom, run_until_ram_value};

#[test]
fn ppu_vbl_nmi_timing_stabilizes() {
    let relative = "blargg/ppu/vbl_nmi_timing/rom_singles/01-vbl_basics.nes";
    if let Some(msg) = require_rom(relative) {
        eprintln!("{msg}");
        return;
    }

    let mut nes = load_nes_from_relative_rom(relative).expect("Failed to load ROM");
    run_until_ram_value(&mut nes, 0x6000, 0x00, DEFAULT_MAX_FRAMES * 3)
        .expect("PPU VBL/NMI timing test did not converge to success code");
}

#[test]
fn ppu_sprite_hit_behavior_stabilizes() {
    let relative = "blargg/ppu/sprite_hit_tests_2005.10.05/01.basics.nes";
    if let Some(msg) = require_rom(relative) {
        eprintln!("{msg}");
        return;
    }

    let mut nes = load_nes_from_relative_rom(relative).expect("Failed to load ROM");
    run_until_ram_value(&mut nes, 0x6000, 0x00, DEFAULT_MAX_FRAMES * 3)
        .expect("Sprite hit test did not report success");
}
