# RPi LED Sign Controller

A LED matrix display controller for Raspberry Pi with web-based configuration. This application provides a flexible interface for controlling LED matrix panels through a web UI and supports multiple display drivers.

## Features

- Control RGB LED matrix panels connected to Raspberry Pi GPIO
- Web-based configuration interface
- Text scrolling with customizable speed and colors
- Two driver options: native Rust or C++ binding
- Support for various LED matrix panel configurations

## Installation

### Prerequisites

- Raspberry Pi (tested on Pi 4)
- LED matrix panels compatible with the HUB75 interface
- Rust toolchain (rustc, cargo)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/paviro/rpi-led-sign-controller.git
cd rpi-led-sign-controller

# Build the project
cargo build --release

```

## Usage

The application provides a web interface accessible at `http://<raspberry-pi-ip>:3000` for configuring the display content. 

Run the application:

```bash
# Basic usage with native Rust driver
sudo ./target/release/rpi_led_sign_controller --driver native --rows 32 --cols 64 --chain-length 1

# Using the C++ binding driver
sudo ./target/release/rpi_led_sign_controller --driver binding --rows 32 --cols 64 --chain-length 1
```

## Driver Selection

The application supports two different LED matrix drivers:

1. **Native** (`--driver native`): Pure Rust implementation from [rpi_led_panel](https://github.com/EmbersArc/rpi_led_panel)
2. **Binding** (`--driver binding`): C++ binding to Henner Zeller's [rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix) library

## CLI Arguments

| Argument | Description | Default | Supported By |
|----------|-------------|---------|-------------|
| `--driver`, `-d` | Driver type: "native" or "binding" (REQUIRED) | - | Both |
| `--rows`, `-r` | Number of rows per panel | 32 | Both |
| `--cols`, `-c` | Number of columns per panel | 64 | Both |
| `--parallel`, `-p` | Number of chains to run in parallel | 1 | Both (binding: 1-3 only) |
| `--chain-length`, `-n` | Number of daisy-chained panels | 1 | Both |
| `--led-brightness`, `-b` | Brightness percent (0-100) | 100 | Both |
| `--hardware-mapping` | Display wiring configuration | "regular" | Both |
| `--refresh-rate` | Display refresh rate | 120 | Native |
| `--pi-chip` | Raspberry Pi chip model (e.g., "BCM2711") | auto | Native |
| `--pwm-bits` | PWM bits for color depth (1-11) | 11 | Both |
| `--pwm-lsb-nanoseconds` | Base time-unit for the on-time in LSB | 130 | Both |
| `--gpio-slowdown` | GPIO slowdown factor (0-4) | auto | Both |
| `--interlaced` | Enable interlaced scan mode | false | Both |
| `--dither-bits` | Bits for time dithering | 0 | Both |
| `--panel-type` | Panel initialization type (e.g., "FM6126A") | - | Both |
| `--multiplexing` | Multiplexing type | - | Both |
| `--pixel-mapper` | List of pixel mappers ("U-mapper;Rotate:90") | - | Both |
| `--row-setter` | Row address setter type | "default" | Both |
| `--led-sequence` | LED color sequence | "RGB" | Both |
| `--no-hardware-pulse` | Disable hardware pin-pulse generation | false | Binding |
| `--show-refresh` | Show refresh rate on terminal | false | Binding |
| `--inverse-colors` | Invert display colors | false | Binding |
| `--limit-refresh` | Limit refresh rate in Hz (0=unlimited) | 0 | Binding |

## Environment Variables

All CLI options can be set via environment variables with the `LED_` prefix.

| Environment Variable | Equivalent CLI Argument |
|----------------------|-------------------------|
| `LED_DRIVER` | `--driver` |
| `LED_ROWS` | `--rows` |
| `LED_COLS` | `--cols` |
| `LED_CHAIN_LENGTH` | `--chain-length` |
| `LED_PARALLEL` | `--parallel` |
| `LED_BRIGHTNESS` | `--led-brightness` |
| `LED_HARDWARE_MAPPING` | `--hardware-mapping` |
| `LED_REFRESH_RATE` | `--refresh-rate` |
| `LED_PI_CHIP` | `--pi-chip` |
| `LED_PWM_BITS` | `--pwm-bits` |
| `LED_PWM_LSB_NANOSECONDS` | `--pwm-lsb-nanoseconds` |
| `LED_GPIO_SLOWDOWN` | `--gpio-slowdown` |
| `LED_INTERLACED` | `--interlaced` |
| `LED_DITHER_BITS` | `--dither-bits` |
| `LED_PANEL_TYPE` | `--panel-type` |
| `LED_MULTIPLEXING` | `--multiplexing` |
| `LED_PIXEL_MAPPER` | `--pixel-mapper` |
| `LED_ROW_SETTER` | `--row-setter` |
| `LED_SEQUENCE` | `--led-sequence` |
| `LED_HARDWARE_PULSING` | `--no-hardware-pulse` (inverted) |
| `LED_SHOW_REFRESH` | `--show-refresh` |
| `LED_INVERSE_COLORS` | `--inverse-colors` |
| `LED_LIMIT_REFRESH` | `--limit-refresh` |

## Hardware Mapping Options

The `--hardware-mapping` parameter depends on how your LED matrix is connected to the Raspberry Pi. Common values include:

- `regular`: Standard GPIO mapping 
- `adafruit-hat`: Adafruit RGB Matrix Bonnet/HAT
- `adafruit-hat-pwm`: Adafruit RGB Matrix Bonnet/HAT with hardware PWM
- `regular-pi1`: Standard GPIO mapping for Raspberry Pi 1
- `classic`: Early version of matrix wiring
- `classic-pi1`: Early version for Pi 1

## Multiplexing Options

The following options are available for the `--multiplexing` parameter:

- `Stripe` - Traditional line-by-line multiplexing
- `Checkered` - Alternate pixels are on different scan lines
- `Spiral` - Panel using spiral of matrix segments
- `ZStripe08` - Z-stripe with 8 pixel intervals
- `ZStripe44` - Z-stripe with 4x4 pixel intervals 
- `ZStripe80` - Z-stripe with 8x0 pixel intervals
- `Coreman` - Multiplexing used in some Colorlight controllers
- `Kaler2Scan` - Scan pattern used in some Kaler panels
- `P10Z` - P10 outdoor panels with Z layout
- `QiangLiQ8` - QiangLi Q8 panels
- `InversedZStripe` - Inverted Z-stripe pattern
- `P10Outdoor1R1G1B1` - P10 outdoor panel variant 1
- `P10Outdoor1R1G1B2` - P10 outdoor panel variant 2
- `P10Outdoor1R1G1B3` - P10 outdoor panel variant 3
- `P10Coreman` - P10 panels with Coreman multiplexing
- `P8Outdoor1R1G1B` - P8 outdoor panels
- `FlippedStripe` - Stripe pattern with flipped orientation
- `P10Outdoor32x16HalfScan` - P10 32x16 outdoor panels with half-scan

## Disclaimer

This project was developed with significant assistance from AI (specifically Claude). While efforts have been made to ensure code quality, the implementation may contain inefficiencies or non-idiomatic patterns. Contributions and improvements are welcome! I am not a Rust developer 🙈

## Credits

- [rpi_led_panel](https://github.com/EmbersArc/rpi_led_panel) - Native Rust driver
- [rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix) - C++ library
- [rust-rpi-rgb-led-matrix](https://github.com/rust-rpi-led-matrix/rust-rpi-rgb-led-matrix) - Rust binding