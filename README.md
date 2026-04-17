# LakeNES

LakeNES is a NES emulator written in Rust.

> To-do: Refactor the codebase to use a per-cycle accurate CPU and PPU instead of a per-instruction step. This will allow for more accurate emulation of the NES hardware.

## Features

- CPU emulation
- PPU emulation
- APU emulation
- Mapper emulation
- Controller emulation
- Save state support
- Debugging tools

## Getting Started

### Prerequisites

- Rust 1.70.0 or higher
- Cargo 1.70.0 or higher

### Usage

```bash
./lakenes <rom_path>
```

## License

This project is licensed under the MIT license.