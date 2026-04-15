mod common;

use common::{
    DEFAULT_MAX_FRAMES, load_nes_from_relative_rom, require_rom, run_until_prg_ram_signature,
};

#[test]
fn blargg_instr_test_v5_reports_pass() {
    let relative = "blargg/cpu/instr_test-v5/all_instrs.nes";
    if let Some(msg) = require_rom(relative) {
        eprintln!("{msg}");
        return;
    }

    let mut nes = load_nes_from_relative_rom(relative).expect("Failed to load ROM");
    run_until_prg_ram_signature(&mut nes, 0x6004, "PASSED", DEFAULT_MAX_FRAMES * 4)
        .expect("blargg CPU test did not report PASSED");
}
