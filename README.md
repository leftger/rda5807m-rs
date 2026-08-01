# rda5807m-rs

[![crates.io](https://img.shields.io/crates/v/rda5807m-rs.svg)](https://crates.io/crates/rda5807m-rs)
[![docs.rs](https://img.shields.io/docsrs/rda5807m-rs)](https://docs.rs/rda5807m-rs)
[![CI](https://github.com/leftger/rda5807-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/leftger/rda5807-rs/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

A `#![no_std]` Rust device driver implementation for the RDA5807M single-chip broadcast FM radio tuner, powered by [`device-driver`](https://crates.io/crates/device-driver).

## Architecture

```text
+-------------------+       I2C Register Interface        +-------------------+
|  Microcontroller  | <=================================> | RDA5807M FM Tuner |
| (Cortex-M / Host) |     Read/Write (Addr 0x00..0x0F)    |   Radio Receiver  |
+-------------------+                                     +-------------------+
                                                                    |
                                                              [ FM Antenna ]
```

## Features

- **`no_std` Support**: Designed for bare-metal embedded targets (Cortex-M, RISC-V, ESP32, AVR, etc.) as well as host testing.
- **Declarative Register Mapping**: Type-safe register abstractions generated from a structured YAML manifest via `device-driver`.
- **Frequency Tuning**: Precise direct tuning in kHz for all supported global broadcast FM bands and channel spacings.
- **Multi-Band & Spacing**:
  - **USA / Europe**: 87.0 MHz – 108.0 MHz
  - **Japan**: 76.0 MHz – 91.0 MHz
  - **WorldWide**: 76.0 MHz – 108.0 MHz
  - **East Europe / Extended**: 65.0 MHz – 76.0 MHz / 50.0 MHz – 115.0 MHz
  - Channel spacings: 25 kHz, 50 kHz, 100 kHz, 200 kHz.
- **Audio Controls**: Volume (0–15 with clamping), Mute & Soft Mute, Bass Boost, Mono / Stereo selection.
- **De-Emphasis Filtering**: Configurable 75 µs (USA/Japan) or 50 µs (Europe/Australia) filtering.
- **Signal Quality Metrics**: RSSI reading (0–127), station validation (`is_station`), and stereo status indicator (`is_stereo`).
- **Autonomous Station Seeking**: Upward/downward channel seeking with boundary limit or wrap-around options.
- **RDS / RBDS Decoding**: Enable RDS engine and extract raw RDS blocks (Block A, B, C, D).
- **Flexible Reference Clock**: Supports 32.768 kHz crystals, 12 MHz, 13 MHz, 19.2 MHz, 24 MHz, 26 MHz, and 38.4 MHz clock inputs.

## Installation

Add `rda5807m-rs` to your `Cargo.toml`:

```toml
[dependencies]
rda5807m-rs = "0.1.2"
```

## Usage

### Quick Start (Host or Embedded)

```rust
use rda5807m_rs::{Rda5807m, Band, Space, ClkMode};

// Create driver wrapping any device_driver::RegisterInterface (e.g. over I2C)
let mut tuner = Rda5807m::new(i2c_interface);

// 1. Verify device signature (expected 0x58)
let chip_id = tuner.read_chip_id()?;
assert_eq!(chip_id, 0x58);

// 2. Power up chip with audio outputs enabled
tuner.power_on()?;

// 3. Configure reference clock and radio band
tuner.set_clock_mode(ClkMode::KHz32768)?;
tuner.set_band_and_spacing(Band::UsaEurope, Space::KHz100)?;

// 4. Tune to 101.1 MHz (101,100 kHz)
tuner.set_frequency_khz(101_100)?;

// 5. Configure volume and audio settings
tuner.set_volume(12)?;
tuner.set_bass_boost(true)?;
tuner.set_deemphasis(false)?; // 75 µs for USA

// 6. Inspect signal & reception status
if tuner.is_seek_tune_complete()? {
    let rssi = tuner.get_rssi()?;
    let is_stereo = tuner.is_stereo()?;
    let is_station = tuner.is_station()?;
    let current_freq = tuner.get_frequency_khz()?;
}

// 7. Enable RDS and read block data
tuner.set_rds_enable(true)?;
let (block_a, block_b, block_c, block_d) = tuner.read_rds_blocks()?;

// 8. Power off
tuner.power_off()?;
# Ok::<(), ()>(())
```

### Implementing `RegisterInterface`

`rda5807m-rs` communicates through the `device_driver::RegisterInterface` trait:

```rust
use device_driver::RegisterInterface;

pub struct MyI2cBus<I2C> {
    i2c: I2C,
}

impl<I2C, E> RegisterInterface for MyI2cBus<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    type AddressType = u8;
    type Error = E;

    fn write_register(
        &mut self,
        address: u8,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.i2c.write(address, data)
    }

    fn read_register(
        &mut self,
        address: u8,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c.read(address, data)
    }
}
```

## Driver API Overview

| Function | Description |
|---|---|
| `power_on()` | Powers up the RDA5807M with audio outputs enabled and mute disabled |
| `power_off()` | Places chip into low-power standby mode |
| `soft_reset()` | Triggers internal software reset |
| `set_volume(volume)` | Sets volume output (0 to 15, clamped automatically) |
| `set_mute(mute)` | Toggles hardware audio mute |
| `set_bass_boost(enable)` | Enables or disables low-frequency bass boost |
| `set_mono(force_mono)` | Forces mono output or enables stereo reception |
| `set_deemphasis(50us)` | Sets de-emphasis filter time constant (`false` = 75 µs, `true` = 50 µs) |
| `set_softmute(enable)` | Enables soft mute feature on weak signals |
| `set_rds_enable(enable)` | Enables or disables RDS/RBDS receiver engine |
| `set_clock_mode(mode)` | Configures reference clock frequency (`ClkMode`) |
| `set_band_and_spacing(band, space)` | Configures frequency band and channel step size |
| `set_frequency_khz(freq_khz)` | Tunes tuner to frequency in kHz and triggers tune operation |
| `get_frequency_khz()` | Reads current tuned frequency in kHz |
| `seek(seek_up, stop_at_limit)` | Initiates autonomous station seeking |
| `is_seek_tune_complete()` | Returns `true` when tune or seek completes (`STC`) |
| `is_seek_failed()` | Returns `true` if seek operation failed to find a station (`SF`) |
| `get_rssi()` | Reads Received Signal Strength Indicator (0..127) |
| `is_station()` | Returns `true` if tuned channel is a valid station (`FM_TRUE`) |
| `is_stereo()` | Returns `true` if current channel is stereo (`ST`) |
| `read_chip_id()` | Reads chip identifier (expected `0x58`) |
| `read_rds_blocks()` | Reads raw RDS block registers `(block_a, block_b, block_c, block_d)` |

## Register Map Reference

| Address | Name | Description |
|---|---|---|
| `0x00` | `ChipId` | Chip identification register (Read ID `0x58`) |
| `0x02` | `GeneralControl` | Chip enable, soft reset, audio mute, mono, bass boost, seek & RDS |
| `0x03` | `ChannelTuning` | Frequency channel selector, tune trigger, band & spacing setup |
| `0x04` | `VolumeAndControl` | Volume control (0–15), de-emphasis, softmute, GPIO configuration |
| `0x05` | `AudioControl` | De-emphasis selection, softmute enable, interrupt setup |
| `0x06` | `OpenReservedMode` | Reserved mode control register |
| `0x07` | `SystemControl2` | Softblend enable, extended frequency band mode |
| `0x0A` | `Status1` | Seek/tune complete (`STC`), seek fail (`SF`), stereo flag (`ST`), read channel |
| `0x0B` | `Status2` | RSSI level (bits 15:9), station status (`FM_TRUE`), block error indicators |
| `0x0C` | `RdsData0` | RDS Block A data |
| `0x0D` | `RdsData1` | RDS Block B data |
| `0x0E` | `RdsData2` | RDS Block C data |
| `0x0F` | `RdsData3` | RDS Block D data |

## Development & Testing

### Running Tests

```bash
cargo test
```

### Local CI Workflow Simulation

Simulate full GitHub Actions CI checks locally:

```bash
bash .github/scripts/ci_local.sh
```

### Git Pre-Commit Hooks

Install pre-commit hooks to automatically format and check code before committing:

```bash
bash scripts/install-git-hooks.sh
```

Or using `pre-commit`:

```bash
pre-commit install
```

## License

Dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
