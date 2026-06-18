# Rust-Boy

A high-accuracy Game Boy emulator written in Rust, designed to faithfully reproduce the behavior of the original hardware with cycle-accurate precision.

## Overview

Rust-Boy is a cycle-accurate Game Boy emulator that prioritizes hardware accuracy and correctness. By faithfully implementing the Z80-based CPU architecture, PPU graphics processor, and APU audio unit, the emulator strives to provide an authentic experience that closely mirrors the original Game Boy hardware.

## Features

- **Cycle-accurate Z80 CPU emulation** - Precise instruction timing and register behavior
- **PPU graphics emulation** - Faithful tile-based rendering engine matching original hardware
- **APU audio emulation** - Support for all four audio channels with waveform synthesis
- **Boot ROM support** - Optional authentic Game Boy boot sequence
- **Extensible opcode architecture** - Clean, maintainable instruction set implementation

## Development Status

**Pre-Alpha** - Currently in active development with focus on core CPU and opcode implementation. PPU and APU integration planned for upcoming releases.

## Supported Platforms

- **Windows** (Primary)
- **Linux** (via Rust compilation)
- **macOS** (via Rust compilation)

## Building

```bash
cargo build --release
```

## Roadmap

- ✅ CPU core architecture
- 🔄 Complete opcode set implementation
- ⏳ PPU graphics engine
- ⏳ APU audio system
- ⏳ Full Game Boy compatibility
- ⏳ UI and debugging tools
- ⏳ Save state support

## License

Licensed under Apache License 2.0 - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to open issues and pull requests.