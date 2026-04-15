LakeNES test ROMs are intentionally not committed in this repository.

Set the environment variable `LAKENES_TEST_ROMS_DIR` to a local folder that
contains the ROM layout expected by the test harness.

Expected layout:

- `blargg/cpu/instr_test-v5/all_instrs.nes`
- `blargg/ppu/vbl_nmi_timing/rom_singles/01-vbl_basics.nes`
- `blargg/ppu/sprite_hit_tests_2005.10.05/01.basics.nes`
- `mappers/mapper3_cnrom_smoke.nes`
- `mappers/mapper7_axrom_smoke.nes`

When files are missing, tests log a skip message and return early.
