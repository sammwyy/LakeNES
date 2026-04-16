/* tslint:disable */
/* eslint-disable */

export class WasmNes {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  set_paused(paused: boolean): void;
  step_cycle(): bigint;
  step_frame(): void;
  disassemble(addr: number): string;
  get_ppu_oam(): Uint8Array;
  get_ppu_mask(): number;
  update_joypad(player: number, buttons: number): void;
  write_ppu_mask(mask: number): void;
  get_ppu_palette(): Uint32Array;
  set_apu_volumes(master: number, pulse1: number, pulse2: number, triangle: number, noise: number, dmc: number): void;
  audio_buffer_len(): number;
  get_audio_sample(): number;
  get_rom_chr_size(): number;
  get_rom_prg_size(): number;
  get_total_cycles(): bigint;
  get_total_frames(): bigint;
  step_instruction(): void;
  get_cpu_registers(): Uint32Array;
  get_pattern_table(table_idx: number): Uint8Array;
  get_rom_mapper_id(): number;
  get_frame_buffer_ptr(): number;
  set_audio_sample_rate(sample_rate: number): void;
  set_ppu_mask_override(mask?: number | null): void;
  get_apu_channels_state(): Float32Array;
  static new(rom_data: Uint8Array): WasmNes;
  is_paused(): boolean;
  set_speed(percent: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmnes_free: (a: number, b: number) => void;
  readonly wasmnes_audio_buffer_len: (a: number) => number;
  readonly wasmnes_disassemble: (a: number, b: number) => [number, number];
  readonly wasmnes_get_apu_channels_state: (a: number) => [number, number];
  readonly wasmnes_get_audio_sample: (a: number) => number;
  readonly wasmnes_get_cpu_registers: (a: number) => [number, number];
  readonly wasmnes_get_frame_buffer_ptr: (a: number) => number;
  readonly wasmnes_get_pattern_table: (a: number, b: number) => [number, number];
  readonly wasmnes_get_ppu_mask: (a: number) => number;
  readonly wasmnes_get_ppu_oam: (a: number) => [number, number];
  readonly wasmnes_get_ppu_palette: (a: number) => [number, number];
  readonly wasmnes_get_rom_chr_size: (a: number) => number;
  readonly wasmnes_get_rom_mapper_id: (a: number) => number;
  readonly wasmnes_get_rom_prg_size: (a: number) => number;
  readonly wasmnes_get_total_cycles: (a: number) => bigint;
  readonly wasmnes_get_total_frames: (a: number) => bigint;
  readonly wasmnes_is_paused: (a: number) => number;
  readonly wasmnes_new: (a: number, b: number) => number;
  readonly wasmnes_set_apu_volumes: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wasmnes_set_audio_sample_rate: (a: number, b: number) => void;
  readonly wasmnes_set_paused: (a: number, b: number) => void;
  readonly wasmnes_set_ppu_mask_override: (a: number, b: number) => void;
  readonly wasmnes_set_speed: (a: number, b: number) => void;
  readonly wasmnes_step_cycle: (a: number) => bigint;
  readonly wasmnes_step_frame: (a: number) => void;
  readonly wasmnes_step_instruction: (a: number) => void;
  readonly wasmnes_update_joypad: (a: number, b: number, c: number) => void;
  readonly wasmnes_write_ppu_mask: (a: number, b: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
