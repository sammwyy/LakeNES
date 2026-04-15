mod common;

use common::{load_nes_from_relative_rom, require_rom, run_frames};

#[test]
fn mapper3_cnrom_smoke_boots() {
    let relative = "mappers/mapper3_cnrom_smoke.nes";
    if let Some(msg) = require_rom(relative) {
        eprintln!("{msg}");
        return;
    }

    let mut nes = load_nes_from_relative_rom(relative).expect("Failed to load mapper 3 ROM");
    run_frames(&mut nes, 120);
}

#[test]
fn mapper7_axrom_smoke_boots() {
    let relative = "mappers/mapper7_axrom_smoke.nes";
    if let Some(msg) = require_rom(relative) {
        eprintln!("{msg}");
        return;
    }

    let mut nes = load_nes_from_relative_rom(relative).expect("Failed to load mapper 7 ROM");
    run_frames(&mut nes, 120);
}
