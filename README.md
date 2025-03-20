# RPi LED Sign Controller

A LED matrix display controller for Raspberry Pi with web-based configuration. This application provides a flexible interface for controlling LED matrix panels through a web UI and supports multiple display drivers.

<div style="display: flex; gap: 10px; justify-content: center; align-items: center;">

  <img src="https://github.com/user-attachments/assets/565ccce9-e814-40f8-b08e-8e85194e1f89" alt="Playlist Manager" width="45%">
  
  <img src="https://github.com/user-attachments/assets/4979dadd-8e92-4b18-b548-a109d098f879" alt="Text Editor" width="45%">

</div>


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
- Rust toolchain (rustc, cargo) (if not using quick install)

### Quick Install

To quickly install or update the LED Sign Controller on your Raspberry Pi, you can use this one-liner:

```bash
# Download the installation script
curl -sSL https://raw.githubusercontent.com/paviro/rpi-led-sign-controller/main/scripts/install.sh -o install.sh
# Run it with sudo
sudo bash install.sh
```

This will download and run the installation script, which will check for an existing installation, install/update dependencies, build the application, and help you configure your LED panel.

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

| Argument | Type | Description | Default | Supported By |
|----------|------|-------------|---------|-------------|
| `--driver`, `-d` | Option | Driver type: "native" or "binding" (REQUIRED) | - | Both |
| `--rows`, `-r` | Option | Number of rows per panel | 32 | Both |
| `--cols`, `-c` | Option | Number of columns per panel | 64 | Both |
| `--parallel`, `-p` | Option | Number of chains to run in parallel | 1 | Both |
| `--chain-length`, `-n` | Option | Number of daisy-chained panels | 1 | Both |
| `--limit-max-brightness` | Option | Maximum brightness limit (0-100). The UI's 100% setting will equal this value | 100 | Both |
| `--hardware-mapping` | Option | Display wiring configuration | "regular" | Both |
| `--limit-refresh-rate` | Option | Limit refresh rate in Hz (0 = unlimited) | 0 | Both |
| `--pi-chip` | Option | Raspberry Pi chip model (e.g., "BCM2711") | auto | Native |
| `--pwm-bits` | Option | PWM bits for color depth (1-11) | 11 | Both |
| `--pwm-lsb-nanoseconds` | Option | Base time-unit for the on-time in LSB | 130 | Both |
| `--gpio-slowdown` | Option | GPIO slowdown factor (0-4) | auto | Both |
| `--dither-bits` | Option | Bits for time dithering | 0 | Both |
| `--panel-type` | Option | Panel initialization type (e.g., "FM6126A") | - | Both |
| `--multiplexing` | Option | Multiplexing type | - | Both |
| `--pixel-mapper` | Option | List of pixel mappers ("U-mapper;Rotate:90") | - | Both |
| `--row-setter` | Option | Row address setter type | "direct" | Both |
| `--led-sequence` | Option | LED color sequence | "RGB" | Both |
| `--interlaced` | Switch | Enable interlaced scan mode | Disabled | Both |
| `--no-hardware-pulse` | Switch | Disable hardware pin-pulse generation | Disabled | Binding |
| `--show-refresh` | Switch | Show refresh rate on terminal | Disabled | Binding |
| `--inverse-colors` | Switch | Invert display colors | Disabled | Binding |


## Environment Variables

All CLI options can be set via environment variables with the `LED_` prefix.

| Environment Variable | Equivalent CLI Argument |
|----------------------|-------------------------|
| `LED_DRIVER` | `--driver` |
| `LED_ROWS` | `--rows` |
| `LED_COLS` | `--cols` |
| `LED_CHAIN_LENGTH` | `--chain-length` |
| `LED_PARALLEL` | `--parallel` |
| `LED_LIMIT_MAX_BRIGHTNESS` | `--limit-max-brightness` |
| `LED_HARDWARE_MAPPING` | `--hardware-mapping` |
| `LED_LIMIT_REFRESH_RATE` | `--limit-refresh-rate` |
| `LED_PI_CHIP` | `--pi-chip` |
| `LED_PWM_BITS` | `--pwm-bits` |
| `LED_PWM_LSB_NANOSECONDS` | `--pwm-lsb-nanoseconds` |
| `LED_GPIO_SLOWDOWN` | `--gpio-slowdown` |
| `LED_DITHER_BITS` | `--dither-bits` |
| `LED_PANEL_TYPE` | `--panel-type` |
| `LED_MULTIPLEXING` | `--multiplexing` |
| `LED_PIXEL_MAPPER` | `--pixel-mapper` |
| `LED_ROW_SETTER` | `--row-setter` |
| `LED_SEQUENCE` | `--led-sequence` |
| `LED_HARDWARE_PULSING` | `--no-hardware-pulse` (inverted) |
| `LED_SHOW_REFRESH` | `--show-refresh` |
| `LED_INVERSE_COLORS` | `--inverse-colors` |

## Hardware Mapping Options

The `--hardware-mapping` parameter depends on how your LED matrix is connected to the Raspberry Pi.

| Mapping Value | Alternate Names | Description | Driver Support |
|---------------|----------------|-------------|----------------|
| `regular` | | Standard GPIO mapping (default) | Both |
| `adafruit-hat` | `AdafruitHat` | Adafruit RGB Matrix Bonnet/HAT | Both |
| `adafruit-hat-pwm` | `AdafruitHatPwm` | Adafruit RGB Matrix Bonnet/HAT with hardware PWM | Both |
| `regular-pi1` | `RegularPi1` | Standard GPIO mapping for Raspberry Pi 1 | Both |
| `classic` | | Early version of matrix wiring (not recommended for new setups) | Both |
| `classic-pi1` | `ClassicPi1` | Early version for Pi 1 Rev A | Both |

Both kebab-case (`adafruit-hat`) and PascalCase (`AdafruitHat`) naming styles are supported for backward compatibility with both drivers.

## Row Setter Options

The `--row-setter` parameter controls how the row address is set on the LED matrix. The following options are supported:

| Option Value | Alternate Name | Description |
|--------------|----------------|-------------|
| `direct` | `default` | Direct row selection (default) |
| `shiftregister` | `ab-addressed` | Shift register selection (AB addressed panels) |
| `directabcdline` | `direct-row-select` | Direct ABCD line selection |
| `abcshiftregister` | `abc-addressed` | ABC shift register selection |
| `sm5266` | `abc-shift-de` | SM5266 with ABC shifter + DE direct |

The row setter determines how the GPIO pins are configured to address different rows on the LED panel. The correct value depends on your specific LED panel type and wiring configuration.

## Multiplexing Options

The `--multiplexing` parameter determines how the display is electrically multiplexed.

| Multiplexing Value | Description |
|--------------------|-------------|
| `Stripe` | Traditional line-by-line multiplexing (default for binding driver) |
| `Checkered`, `Checker` | Alternate pixels are on different scan lines |
| `Spiral` | Panel using spiral of matrix segments |
| `ZStripe`, `ZStripe08` | Z-stripe with 8 pixel intervals |
| `ZStripe44` | Z-stripe with 4x4 pixel intervals |
| `ZStripe80` | Z-stripe with 8x0 pixel intervals |
| `Coreman` | Multiplexing used in some Colorlight controllers |
| `Kaler2Scan` | Scan pattern used in some Kaler panels |
| `P10Z` | P10 outdoor panels with Z layout |
| `QiangLiQ8` | QiangLi Q8 panels |
| `InversedZStripe` | Inverted Z-stripe pattern |
| `P10Outdoor1R1G1B1` | P10 outdoor panel variant 1 |
| `P10Outdoor1R1G1B2` | P10 outdoor panel variant 2 |
| `P10Outdoor1R1G1B3` | P10 outdoor panel variant 3 |
| `P10Coreman` | P10 panels with Coreman multiplexing |
| `P8Outdoor1R1G1B` | P8 outdoor panels |
| `FlippedStripe` | Stripe pattern with flipped orientation |
| `P10Outdoor32x16HalfScan` | P10 32x16 outdoor panels with half-scan |

The correct multiplexing option depends on your specific panel type. Most common panels use either `Stripe` or `Checkered`.

## Web Server Configuration

The application includes a web server for configuration and control of the LED matrix. You can customize where this web server binds using the following options:

### Web Server Options

| Option | Description | Default |
|--------|-------------|---------|
| `--port` | Web server port | 3000 |
| `--interface` | Network interface to bind to | `0.0.0.0` (all interfaces) |

### Environment Variables

These settings can also be configured using environment variables:

- `LED_PORT` - Set the web server port
- `LED_INTERFACE` - Set the binding interface

## CLI Usage Notes

### Options vs. Switches

This application uses two types of command-line parameters with different behaviors:

#### 1. CLI Arguments

- **Options** require a value: `--rows 32`, `--cols 64`
- **Switches** are flags with no values:
  - To enable: include the switch (e.g., `--interlaced`)
  - To disable: omit the switch entirely

#### 2. Environment Variables

All parameters (including switches) accept values when set as environment variables:

- For normal options: `LED_ROWS=32`, `LED_COLS=64`
- For switches: 
  - To enable: `LED_INTERLACED=true` or `LED_INTERLACED=1`
  - To disable: `LED_INTERLACED=false` or `LED_INTERLACED=0`

This difference in behavior between CLI switches and environment variables is due to how environment variables fundamentally work - they must always have a value, whereas CLI flags can be present or absent.

### Special Case: Hardware Pulsing

Note that the environment variable `LED_HARDWARE_PULSING` is inverted from its CLI counterpart `--no-hardware-pulse`:

- CLI: `--no-hardware-pulse` (disables hardware pulsing)
- ENV: `LED_HARDWARE_PULSING=false` (also disables hardware pulsing)

This reversal exists because the CLI flag is a "negative" switch.

## Disclaimer

This project was developed with significant assistance from AI (specifically Claude). While efforts have been made to ensure code quality, the implementation may contain inefficiencies or non-idiomatic patterns. Contributions and improvements are welcome! I am not a Rust developer 🙈

## Credits

- [rpi_led_panel](https://github.com/EmbersArc/rpi_led_panel) - Native Rust driver
- [rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix) - C++ library
- [rust-rpi-rgb-led-matrix](https://github.com/rust-rpi-led-matrix/rust-rpi-rgb-led-matrix) - Rust binding
