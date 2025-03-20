#!/bin/bash
set -e

# Colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}RPi LED Sign Controller - Installation & Update Script${NC}"
echo -e "==============================================="
echo -e "GitHub Repository: ${GREEN}https://github.com/paviro/rpi-led-sign-controller${NC}"

# Introduction
echo -e "\n${BLUE}About this software:${NC}"
echo -e "This script will install or update the RPi LED Sign Controller, which allows you to"
echo -e "drive HUB75-compatible RGB LED matrix panels from your Raspberry Pi."
echo -e "The software provides a web interface for creating and managing text displays,"
echo -e "animations, and playlists on your LED panels."

echo -e "\n${YELLOW}This script will:${NC}"
echo -e "  • Check if the app is already installed and offer to update it"
echo -e "  • Check for and install required dependencies (Git, Rust)"
echo -e "  • Clone the repository if needed"
echo -e "  • Build the application from source"
echo -e "  • Help you configure your LED panel"
echo -e "  • Install the application as a systemd service"
echo -e "  • Start the service automatically on boot"

# Check if we're running on a Raspberry Pi
if ! grep -q "Raspberry Pi" /proc/cpuinfo && ! grep -q "BCM" /proc/cpuinfo; then
    echo -e "\n${RED}Error: This script must be run on a Raspberry Pi.${NC}"
    echo -e "${YELLOW}If you are running on a Raspberry Pi and seeing this error,${NC}"
    echo -e "${YELLOW}please continue by typing 'y' or abort with any other key.${NC}"
    read -p "Continue anyway? [y/N]: " force_continue
    if [[ "$force_continue" != "y" && "$force_continue" != "Y" ]]; then
        echo -e "${RED}Installation aborted.${NC}"
        exit 1
    fi
    echo -e "${YELLOW}Continuing installation despite platform check...${NC}"
else
    echo -e "\n${GREEN}Raspberry Pi detected.${NC}"
fi

# Function to check if running on a Debian-based system
check_debian_based() {
    if ! command -v apt &> /dev/null && ! command -v apt-get &> /dev/null; then
        echo -e "${RED}Error: This script requires a Debian-based system (Raspberry Pi OS Lite recommended)${NC}"
        echo -e "${RED}The 'apt' package manager was not found on your system.${NC}"
        echo -e "${YELLOW}If you're using a non-Debian system but still want to install, please refer to:${NC}"
        echo -e "${BLUE}https://github.com/paviro/rpi-led-sign-controller${NC}"
        exit 1
    fi
    echo -e "${GREEN}Debian-based system detected.${NC}"
}

# Call this function early in the script, right after the Raspberry Pi check
check_debian_based

# Check if script is run with root privileges
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Please run as root (use sudo)${NC}"
  exit 1
fi

# Determine the actual user who ran the script
ACTUAL_USER=${SUDO_USER:-$USER}
ACTUAL_HOME=$(eval echo ~$ACTUAL_USER)

# Check for and install git if necessary
if ! command -v git &> /dev/null; then
    echo -e "${YELLOW}Git not found. Installing git...${NC}"
    apt-get update
    apt-get install -y git
    echo -e "${GREEN}Git installed successfully.${NC}"
else
    echo -e "${GREEN}Git is already installed.${NC}"
fi

# Check for and install rust if necessary
if ! sudo -u $ACTUAL_USER bash -c "source $ACTUAL_HOME/.cargo/env 2>/dev/null && command -v rustc &> /dev/null && command -v cargo &> /dev/null"; then
    echo -e "${YELLOW}Rust not found. Installing rust for user $ACTUAL_USER...${NC}"
    apt-get update
    apt-get install -y curl build-essential
    
    # Install Rust for the actual user
    sudo -u $ACTUAL_USER bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
    echo -e "${GREEN}Rust installed successfully for user $ACTUAL_USER.${NC}"
else
    echo -e "${GREEN}Rust is already installed for user $ACTUAL_USER.${NC}"
fi

# Store current directory
CURRENT_DIR=$(pwd)

# Check if we're in the project directory or scripts subdirectory
REPO_DIR="/usr/local/src/rpi-led-sign-controller"
UPDATE_MARKER="$REPO_DIR/.update_status"
UPDATES_AVAILABLE=0

if [ -f "../Cargo.toml" ] && grep -q "rpi_led_sign_controller" "../Cargo.toml" 2>/dev/null; then
    echo -e "${YELLOW}Running from scripts directory, moving up one level...${NC}"
    cd ..
    echo -e "${GREEN}Now in project directory.${NC}"
elif [ -f "Cargo.toml" ] && grep -q "rpi_led_sign_controller" "Cargo.toml" 2>/dev/null; then
    echo -e "${GREEN}Already in project directory.${NC}"
else
    # Check if the repository already exists in /usr/local/src
    if [ -d "$REPO_DIR" ]; then
        echo -e "${YELLOW}Found existing repository at $REPO_DIR${NC}"
        
        # Check if binary and service exist to determine if installation is complete
        if [ -f "/usr/local/bin/rpi_led_sign_controller" ] && [ -f "/etc/systemd/system/rpi-led-sign.service" ]; then
            echo -e "${GREEN}Found complete previous installation.${NC}"
            
            # Check for updates
            cd "$REPO_DIR"
            echo -e "${YELLOW}Checking for updates...${NC}"
            git fetch
            
            # Check if there are any updates
            if git status -uno | grep -q "Your branch is up to date"; then
                echo -e "${GREEN}Repository is already up to date. No new updates available.${NC}"
                
                # Skip rebuild prompt, just ask about reconfiguration
                read -p "Do you want to reconfigure your LED panel settings? [y/N]: " reconfigure
                if [[ "$reconfigure" == "y" || "$reconfigure" == "Y" ]]; then
                    # Stop service before reconfiguration
                    echo -e "${YELLOW}Stopping service for reconfiguration...${NC}"
                    systemctl stop rpi-led-sign.service
                    # Continue with configuration (we'll continue the script flow)
                else
                    echo -e "${GREEN}No changes needed. Exiting.${NC}"
                    exit 0
                fi
            else
                # Updates are available
                echo -e "${GREEN}Updates are available!${NC}"
                UPDATES_AVAILABLE=1
                read -p "Do you want to update the installation? [Y/n]: " do_update
                if [[ "$do_update" == "n" || "$do_update" == "N" ]]; then
                    read -p "Do you want to reconfigure your LED panel settings? [y/N]: " reconfigure
                    if [[ "$reconfigure" == "y" || "$reconfigure" == "Y" ]]; then
                        # Stop service before reconfiguration
                        echo -e "${YELLOW}Stopping service for reconfiguration...${NC}"
                        systemctl stop rpi-led-sign.service
                        # Continue with configuration (we'll continue the script flow)
                    else
                        echo -e "${GREEN}No changes made. Exiting.${NC}"
                        exit 0
                    fi
                else
                    # Save the current script hash before pulling
                    SCRIPT_PATH="$REPO_DIR/scripts/install.sh"
                    SCRIPT_OLD_HASH=""
                    if [ -f "$SCRIPT_PATH" ]; then
                        SCRIPT_OLD_HASH=$(md5sum "$SCRIPT_PATH" | awk '{print $1}')
                    fi

                    # Pull changes
                    git pull
                    echo -e "${GREEN}Repository updated successfully.${NC}"

                    # Check if the script itself was updated
                    if [ -f "$SCRIPT_PATH" ]; then
                        SCRIPT_NEW_HASH=$(md5sum "$SCRIPT_PATH" | awk '{print $1}')
                        if [ "$SCRIPT_OLD_HASH" != "$SCRIPT_NEW_HASH" ] && [ ! -z "$SCRIPT_OLD_HASH" ]; then
                            echo -e "${YELLOW}The installation script itself has been updated.${NC}"
                            echo -e "${YELLOW}Relaunching the updated script...${NC}"
                            # Create an update marker
                            echo "updated=$(date +%s)" > "$UPDATE_MARKER"
                            # Return to original directory to match original state
                            cd "$CURRENT_DIR"
                            # Re-execute the script
                            exec sudo bash "$SCRIPT_PATH"
                            # The exec command replaces the current process, so the script will not continue past this point
                        fi
                    fi

                    # Create an update marker if we didn't relaunch
                    echo "updated=$(date +%s)" > "$UPDATE_MARKER"
                fi
            fi
        else
            echo -e "${YELLOW}Found incomplete installation. Repository exists but installation wasn't completed.${NC}"
            
            # Confirm continuing installation
            echo -e "\n${YELLOW}Ready to complete the installation of RPi LED Sign Controller.${NC}"
            echo -e "You will need an LED matrix panel connected to your Raspberry Pi's GPIO pins."
            read -p "Do you want to continue with the installation? [y/N]: " confirm_install
            if [[ "$confirm_install" != "y" && "$confirm_install" != "Y" ]]; then
                echo -e "${RED}Installation aborted.${NC}"
                exit 1
            fi
            
            echo -e "${YELLOW}Continuing with installation using existing repository...${NC}"
            cd "$REPO_DIR"
            
            # Make sure the repository is up to date
            echo -e "${YELLOW}Ensuring repository is up to date...${NC}"
            git pull
            # Create an update marker
            echo "updated=$(date +%s)" > "$UPDATE_MARKER"
        fi
    else
        # New installation - confirm first
        echo -e "\n${YELLOW}Ready to install RPi LED Sign Controller.${NC}"
        echo -e "You will need an LED matrix panel connected to your Raspberry Pi's GPIO pins."
        echo -e "Installation requires approximately 500MB of disk space and may take 10-15 minutes"
        echo -e "depending on your Raspberry Pi model."

        read -p "Do you want to continue with the installation? [y/N]: " confirm_install
        if [[ "$confirm_install" != "y" && "$confirm_install" != "Y" ]]; then
            echo -e "${RED}Installation aborted.${NC}"
            exit 1
        fi
        
        echo -e "${GREEN}Starting installation...${NC}"
        
        echo -e "${YELLOW}Cloning repository to $REPO_DIR...${NC}"
        mkdir -p "$REPO_DIR"
        git clone https://github.com/paviro/rpi-led-sign-controller.git "$REPO_DIR"
        cd "$REPO_DIR"
        # Make sure the directory is owned by the actual user
        chown -R $ACTUAL_USER:$ACTUAL_USER "$REPO_DIR"
        echo -e "${GREEN}Repository cloned successfully.${NC}"
    fi
fi

# Record the project directory
PROJECT_DIR=$(pwd)

# Build the application if new installation, update pulled, or rebuild requested
if [ "$UPDATES_AVAILABLE" -eq 1 ] || [ ! -f "/usr/local/bin/rpi_led_sign_controller" ] || [ -f "$UPDATE_MARKER" ]; then
    echo -e "${YELLOW}Building application...${NC}"
    # Use the user's cargo environment
    sudo -u $ACTUAL_USER bash -c "source $ACTUAL_HOME/.cargo/env && cargo build --release"
    echo -e "${GREEN}Build completed.${NC}"

    # Stop the service before replacing the binary if it's running
    if [ -f "/etc/systemd/system/rpi-led-sign.service" ] && systemctl is-active --quiet rpi-led-sign.service; then
        echo -e "${YELLOW}Stopping service before updating binary...${NC}"
        systemctl stop rpi-led-sign.service
    fi

    # Install the binary (this requires root)
    echo -e "${YELLOW}Installing binary to /usr/local/bin...${NC}"
    cp target/release/rpi_led_sign_controller /usr/local/bin/
    chmod +x /usr/local/bin/rpi_led_sign_controller
    echo -e "${GREEN}Binary installed.${NC}"
    
    # Remove update marker if it exists
    if [ -f "$UPDATE_MARKER" ]; then
        rm "$UPDATE_MARKER"
    fi

    # Binary has been updated, now ask about reconfiguration
    if [ -f "/etc/systemd/system/rpi-led-sign.service" ]; then
        echo -e "\n${YELLOW}Configuration Options${NC}"
        read -p "Do you want to reconfigure your LED panel settings? [y/N]: " reconfigure_after_update
        if [[ "$reconfigure_after_update" != "y" && "$reconfigure_after_update" != "Y" ]]; then
            echo -e "${GREEN}Keeping existing configuration.${NC}"
            echo -e "${YELLOW}Restarting service with updated binary...${NC}"
            systemctl restart rpi-led-sign.service
            echo -e "${GREEN}Service restarted.${NC}"
            echo -e "${GREEN}Update complete!${NC}"
            
            # Display final information
            echo -e "Web interface available at: http://$(hostname -I | awk '{print $1}'):$(systemctl show rpi-led-sign.service -p Environment | grep LED_PORT | sed 's/.*LED_PORT=\([0-9]*\).*/\1/' || echo "3000")"
            echo -e "Source code is located at: ${BLUE}/usr/local/src/rpi-led-sign-controller${NC}"
            echo -e "You can manage the service with: sudo systemctl [start|stop|restart|status] rpi-led-sign.service"
            echo -e ""
            echo -e "To update in the future, you can either:"
            echo -e "  • Run this script again: ${BLUE}curl -sSL https://raw.githubusercontent.com/paviro/rpi-led-sign-controller/main/scripts/install.sh | sudo bash${NC}"
            echo -e "  • Or from the source directory: ${BLUE}cd /usr/local/src/rpi-led-sign-controller && git pull && sudo bash scripts/install.sh${NC}"
            echo -e ""
            echo -e "To uninstall, run: ${BLUE}sudo bash /usr/local/src/rpi-led-sign-controller/scripts/uninstall.sh${NC}"
            echo -e ""
            echo -e "For more information, visit: ${BLUE}https://github.com/paviro/rpi-led-sign-controller${NC}"
            exit 0
        fi
        
        echo -e "${YELLOW}Proceeding with reconfiguration...${NC}"
        # Continue with configuration
    fi
fi

###########################################
# Interactive LED panel configuration
###########################################

echo -e "${BLUE}LED Panel Configuration${NC}"
echo -e "-----------------------------------------------"
echo -e "Let's configure your LED panel. You will be able to test the configuration before finalizing."
echo -e "For each option, press Enter to use the default value or enter a custom value."

# Default values - These should match the table exactly
DEFAULT_ROWS=32
DEFAULT_COLS=64
DEFAULT_CHAIN_LENGTH=1
DEFAULT_PARALLEL=1
DEFAULT_HARDWARE_MAPPING="regular"
DEFAULT_GPIO_SLOWDOWN=""
DEFAULT_PWM_BITS=11
DEFAULT_PWM_LSB_NANOSECONDS=130
DEFAULT_LED_SEQUENCE="RGB"
DEFAULT_DITHER_BITS=0
DEFAULT_PANEL_TYPE=""
DEFAULT_MULTIPLEXING=""
DEFAULT_PIXEL_MAPPER=""
DEFAULT_ROW_SETTER="direct"
DEFAULT_LIMIT_REFRESH_RATE=0
DEFAULT_MAX_BRIGHTNESS=100
DEFAULT_INTERLACED=0
DEFAULT_NO_HARDWARE_PULSE=0
DEFAULT_SHOW_REFRESH=0
DEFAULT_INVERSE_COLORS=0
DEFAULT_PI_CHIP=""
DEFAULT_WEB_PORT=3000
DEFAULT_WEB_INTERFACE="0.0.0.0"

# Actual values (will be set if not using defaults)
DRIVER=""  # Required - no default
ROWS=$DEFAULT_ROWS
COLS=$DEFAULT_COLS
CHAIN_LENGTH=$DEFAULT_CHAIN_LENGTH
PARALLEL=$DEFAULT_PARALLEL
HARDWARE_MAPPING=$DEFAULT_HARDWARE_MAPPING
GPIO_SLOWDOWN=$DEFAULT_GPIO_SLOWDOWN
PWM_BITS=$DEFAULT_PWM_BITS
PWM_LSB_NANOSECONDS=$DEFAULT_PWM_LSB_NANOSECONDS
LED_SEQUENCE=$DEFAULT_LED_SEQUENCE
DITHER_BITS=$DEFAULT_DITHER_BITS
PANEL_TYPE=$DEFAULT_PANEL_TYPE
MULTIPLEXING=$DEFAULT_MULTIPLEXING
PIXEL_MAPPER=$DEFAULT_PIXEL_MAPPER
ROW_SETTER=$DEFAULT_ROW_SETTER
LIMIT_REFRESH_RATE=$DEFAULT_LIMIT_REFRESH_RATE
MAX_BRIGHTNESS=$DEFAULT_MAX_BRIGHTNESS
INTERLACED=$DEFAULT_INTERLACED
NO_HARDWARE_PULSE=$DEFAULT_NO_HARDWARE_PULSE
SHOW_REFRESH=$DEFAULT_SHOW_REFRESH
INVERSE_COLORS=$DEFAULT_INVERSE_COLORS
PI_CHIP=$DEFAULT_PI_CHIP
WEB_PORT=$DEFAULT_WEB_PORT
WEB_INTERFACE=$DEFAULT_WEB_INTERFACE

# Function to get user input with a default value
get_input() {
    local prompt=$1
    local default=$2
    local value
    
    read -p "${prompt} [${default}]: " value
    echo ${value:-$default}
}

# Function to get yes/no input with clearer defaults
get_yes_no() {
    local prompt=$1
    local default=$2
    local value
    
    # Set the display and default value
    if [[ "$default" == "y" || "$default" == "Y" ]]; then
        default_display="Y/n"
        default_value=1
    else
        default_display="y/N"
        default_value=0
    fi
    
    # Format prompt consistently with other inputs
    read -p "${prompt} (default: $([ $default_value -eq 1 ] && echo "yes" || echo "no")) [${default_display}]: " value
    value=$(echo "$value" | tr '[:upper:]' '[:lower:]')
    
    if [[ -z "$value" ]]; then
        echo $default_value
    elif [[ "$value" == "y" ]]; then
        echo 1
    else
        echo 0
    fi
}

# Function to test the current configuration
test_configuration() {
    echo -e "${YELLOW}Testing LED panel with current configuration...${NC}"
    echo -e "${RED}The program will run for 10 seconds. Press Ctrl+C to stop earlier if needed.${NC}"
    
    # Build the command with required settings
    CMD="/usr/local/bin/rpi_led_sign_controller"
    CMD+=" --driver $DRIVER"
    
    # Add non-default parameters
    if [ "$ROWS" != "$DEFAULT_ROWS" ]; then
        CMD+=" --rows $ROWS"
    fi
    
    if [ "$COLS" != "$DEFAULT_COLS" ]; then
        CMD+=" --cols $COLS"
    fi
    
    if [ "$CHAIN_LENGTH" != "$DEFAULT_CHAIN_LENGTH" ]; then
        CMD+=" --chain-length $CHAIN_LENGTH"
    fi
    
    if [ "$PARALLEL" != "$DEFAULT_PARALLEL" ]; then
        CMD+=" --parallel $PARALLEL"
    fi
    
    if [ "$HARDWARE_MAPPING" != "$DEFAULT_HARDWARE_MAPPING" ]; then
        CMD+=" --hardware-mapping $HARDWARE_MAPPING"
    fi
    
    if [ ! -z "$GPIO_SLOWDOWN" ]; then
        CMD+=" --gpio-slowdown $GPIO_SLOWDOWN"
    fi
    
    if [ "$PWM_BITS" != "$DEFAULT_PWM_BITS" ]; then
        CMD+=" --pwm-bits $PWM_BITS"
    fi
    
    if [ "$PWM_LSB_NANOSECONDS" != "$DEFAULT_PWM_LSB_NANOSECONDS" ]; then
        CMD+=" --pwm-lsb-nanoseconds $PWM_LSB_NANOSECONDS"
    fi
    
    if [ "$DITHER_BITS" != "$DEFAULT_DITHER_BITS" ]; then
        CMD+=" --dither-bits $DITHER_BITS"
    fi
    
    if [ "$ROW_SETTER" != "$DEFAULT_ROW_SETTER" ]; then
        CMD+=" --row-setter $ROW_SETTER"
    fi
    
    if [ "$LED_SEQUENCE" != "$DEFAULT_LED_SEQUENCE" ]; then
        CMD+=" --led-sequence $LED_SEQUENCE"
    fi
    
    if [ "$LIMIT_REFRESH_RATE" != "$DEFAULT_LIMIT_REFRESH_RATE" ]; then
        CMD+=" --limit-refresh-rate $LIMIT_REFRESH_RATE"
    fi
    
    if [ "$MAX_BRIGHTNESS" != "$DEFAULT_MAX_BRIGHTNESS" ]; then
        CMD+=" --limit-max-brightness $MAX_BRIGHTNESS"
    fi
    
    if [ "$WEB_PORT" != "$DEFAULT_WEB_PORT" ]; then
        CMD+=" --port $WEB_PORT"
    fi
    
    if [ "$WEB_INTERFACE" != "$DEFAULT_WEB_INTERFACE" ]; then
        CMD+=" --interface $WEB_INTERFACE"
    fi
    
    # Add optional parameters if set
    if [ ! -z "$PANEL_TYPE" ]; then
        CMD+=" --panel-type $PANEL_TYPE"
    fi
    
    if [ ! -z "$MULTIPLEXING" ]; then
        CMD+=" --multiplexing $MULTIPLEXING"
    fi
    
    if [ ! -z "$PIXEL_MAPPER" ]; then
        CMD+=" --pixel-mapper $PIXEL_MAPPER"
    fi
    
    if [ ! -z "$PI_CHIP" ]; then
        CMD+=" --pi-chip $PI_CHIP"
    fi
    
    # Add switches if enabled
    if [ "$INTERLACED" -eq 1 ]; then
        CMD+=" --interlaced"
    fi
    
    if [ "$NO_HARDWARE_PULSE" -eq 1 ]; then
        CMD+=" --no-hardware-pulse"
    fi
    
    if [ "$SHOW_REFRESH" -eq 1 ]; then
        CMD+=" --show-refresh"
    fi
    
    if [ "$INVERSE_COLORS" -eq 1 ]; then
        CMD+=" --inverse-colors"
    fi
    
    echo -e "${YELLOW}Running: $CMD${NC}"
    timeout 10s $CMD || true  # Allow timeout without failing the script
    
    # Ask if it worked
    local is_working
    read -p "Did the LED panel display correctly? (y/n): " is_working
    if [[ $is_working == "y" || $is_working == "Y" ]]; then
        return 0  # Success
    else
        return 1  # Failure
    fi
}

configure_panel() {
    echo -e "${YELLOW}Please provide the following LED panel information:${NC}"
    
    echo -e "\n${BLUE}Driver Selection (REQUIRED)${NC}"
    echo "1. binding (C++ binding - recommended for most users)"
    echo "2. native (Pure Rust library - experimental)"
    read -p "Select driver type [1]: " driver_choice
    if [[ $driver_choice == "2" ]]; then
        DRIVER="native"
    else
        DRIVER="binding"
    fi
    
    echo -e "\n${BLUE}Panel Dimensions${NC}"
    echo "Default: $DEFAULT_ROWS rows, $DEFAULT_COLS columns"
    ROWS=$(get_input "Number of rows (default: $DEFAULT_ROWS)" $DEFAULT_ROWS)
    COLS=$(get_input "Number of columns (default: $DEFAULT_COLS)" $DEFAULT_COLS)
    CHAIN_LENGTH=$(get_input "Number of panels daisy-chained together (default: $DEFAULT_CHAIN_LENGTH)" $DEFAULT_CHAIN_LENGTH)
    PARALLEL=$(get_input "Number of chains to run in parallel (1-3) (default: $DEFAULT_PARALLEL)" $DEFAULT_PARALLEL)
    
    echo -e "\n${BLUE}Hardware Configuration${NC}"
    echo "Common hardware mappings:"
    echo "  - regular (default) - Standard GPIO mapping"
    echo "  - adafruit-hat - Adafruit RGB Matrix Bonnet/HAT"
    echo "  - adafruit-hat-pwm - Adafruit HAT with hardware PWM"
    echo "  - regular-pi1 - Standard GPIO mapping for Raspberry Pi 1"
    echo "  - classic - Early version of matrix wiring"
    echo "  - classic-pi1 - Early version for Pi 1 Rev A"
    
    HARDWARE_MAPPING=$(get_input "Hardware mapping (default: $DEFAULT_HARDWARE_MAPPING)" $DEFAULT_HARDWARE_MAPPING)
    
    echo -e "\n${BLUE}GPIO Settings${NC}"
    echo "GPIO slowdown is needed for newer Raspberry Pi models:"
    echo "  - Pi 0-3: usually value 1 or 2"
    echo "  - Pi 4: usually value 3 or 4"
    echo "  - (leave empty for automatic selection)"
    
    GPIO_SLOWDOWN=$(get_input "GPIO slowdown factor (leave empty for auto)" "$DEFAULT_GPIO_SLOWDOWN")
    
    echo -e "\n${BLUE}Panel Performance Settings${NC}"
    PWM_BITS=$(get_input "PWM bits (1-11) (default: $DEFAULT_PWM_BITS)" $DEFAULT_PWM_BITS)
    PWM_LSB_NANOSECONDS=$(get_input "PWM LSB nanoseconds (base time-unit) (default: $DEFAULT_PWM_LSB_NANOSECONDS)" $DEFAULT_PWM_LSB_NANOSECONDS)
    DITHER_BITS=$(get_input "Dither bits (0 for no dithering) (default: $DEFAULT_DITHER_BITS)" $DEFAULT_DITHER_BITS)
    
    echo -e "\n${BLUE}Row Address Setup${NC}"
    echo "Row setter options:"
    echo "  - direct (default) - Direct row selection"
    echo "  - shiftregister - AB addressed panels"
    echo "  - directabcdline - Direct ABCD line selection"
    echo "  - abcshiftregister - ABC shift register selection"
    echo "  - sm5266 - SM5266 with ABC shifter + DE direct"
    
    ROW_SETTER=$(get_input "Row setter (default: $DEFAULT_ROW_SETTER)" $DEFAULT_ROW_SETTER)
    
    echo -e "\n${BLUE}Color Settings${NC}"
    echo "Common LED sequences:"
    echo "  - RGB (most panels)"
    echo "  - RBG"
    echo "  - GRB"
    echo "  - GBR"
    echo "  - BRG"
    echo "  - BGR"
    
    LED_SEQUENCE=$(get_input "LED color sequence (default: $DEFAULT_LED_SEQUENCE)" $DEFAULT_LED_SEQUENCE)
    
    # Panel type
    echo -e "\n${BLUE}Advanced Panel Settings${NC}"
    echo "Some panels need special initialization, e.g., FM6126A"
    
    PANEL_TYPE=$(get_input "Panel type (leave empty if not needed)" "$DEFAULT_PANEL_TYPE")
    
    # Multiplexing 
    echo "Multiplexing options:"
    echo "  1. None (default) - No multiplexing"
    echo "  2. Stripe - Traditional line-by-line"
    echo "  3. Checkered/Checker - Alternate pixels on different scan lines"
    echo "  4. Spiral - Panel using spiral of matrix segments"
    echo "  5. ZStripe/ZStripe08 - Z-stripe with 8 pixel intervals"
    echo "  6. ZStripe44 - Z-stripe with 4x4 pixel intervals"
    echo "  7. ZStripe80 - Z-stripe with 8x0 pixel intervals"
    echo "  8. Coreman - Multiplexing used in some Colorlight controllers"
    echo "  9. Kaler2Scan - Scan pattern used in some Kaler panels"
    echo "  10. P10Z - P10 outdoor panels with Z layout"
    echo "  11. QiangLiQ8 - QiangLi Q8 panels"
    echo "  12. InversedZStripe - Inverted Z-stripe pattern"
    echo "  13. P10Outdoor1R1G1B1 - P10 outdoor panel variant 1"
    echo "  14. P10Outdoor1R1G1B2 - P10 outdoor panel variant 2"
    echo "  15. P10Outdoor1R1G1B3 - P10 outdoor panel variant 3"
    echo "  16. P10Coreman - P10 panels with Coreman multiplexing"
    echo "  17. P8Outdoor1R1G1B - P8 outdoor panels"
    echo "  18. FlippedStripe - Stripe pattern with flipped orientation"
    echo "  19. P10Outdoor32x16HalfScan - P10 32x16 outdoor panels with half-scan"

    read -p "Select multiplexing type [1]: " multiplex_choice
    case $multiplex_choice in
        2) MULTIPLEXING="Stripe";;
        3) MULTIPLEXING="Checkered";;
        4) MULTIPLEXING="Spiral";;
        5) MULTIPLEXING="ZStripe";;
        6) MULTIPLEXING="ZStripe44";;
        7) MULTIPLEXING="ZStripe80";;
        8) MULTIPLEXING="Coreman";;
        9) MULTIPLEXING="Kaler2Scan";;
        10) MULTIPLEXING="P10Z";;
        11) MULTIPLEXING="QiangLiQ8";;
        12) MULTIPLEXING="InversedZStripe";;
        13) MULTIPLEXING="P10Outdoor1R1G1B1";;
        14) MULTIPLEXING="P10Outdoor1R1G1B2";;
        15) MULTIPLEXING="P10Outdoor1R1G1B3";;
        16) MULTIPLEXING="P10Coreman";;
        17) MULTIPLEXING="P8Outdoor1R1G1B";;
        18) MULTIPLEXING="FlippedStripe";;
        19) MULTIPLEXING="P10Outdoor32x16HalfScan";;
        *) MULTIPLEXING="";;  # Default to no multiplexing
    esac
    
    # Pixel mapper
    echo "Pixel mapper (semicolon-separated list, e.g., 'U-mapper;Rotate:90')"
    echo "(Leave empty if not needed)"
    
    PIXEL_MAPPER=$(get_input "Pixel mapper (leave empty for none)" "$DEFAULT_PIXEL_MAPPER")
    
    # Advanced switch options
    echo -e "\n${BLUE}Additional Options${NC}"
    INTERLACED=$(get_yes_no "Enable interlaced scan mode?" "n")

    if [[ "$DRIVER" == "binding" ]]; then
        NO_HARDWARE_PULSE=$(get_yes_no "Disable hardware pin-pulse generation?" "n")
        SHOW_REFRESH=$(get_yes_no "Show refresh rate statistics on terminal?" "n")
        INVERSE_COLORS=$(get_yes_no "Invert display colors?" "n")
    fi
    
    if [[ "$DRIVER" == "native" ]]; then
        echo "Raspberry Pi chip model (e.g., BCM2711, leave empty for auto)"
        PI_CHIP=$(get_input "Pi chip model (leave empty for auto)" "$DEFAULT_PI_CHIP")
    fi
    
    LIMIT_REFRESH_RATE=$(get_input "Limit refresh rate (Hz, 0 for unlimited) (default: $DEFAULT_LIMIT_REFRESH_RATE)" $DEFAULT_LIMIT_REFRESH_RATE)
    MAX_BRIGHTNESS=$(get_input "Maximum brightness limit (0-100) (default: $DEFAULT_MAX_BRIGHTNESS)" $DEFAULT_MAX_BRIGHTNESS)
    
    echo -e "\n${BLUE}Web Interface${NC}"
    WEB_PORT=$(get_input "Web server port (default: $DEFAULT_WEB_PORT)" $DEFAULT_WEB_PORT)
    WEB_INTERFACE=$(get_input "Network interface to bind to (default: $DEFAULT_WEB_INTERFACE)" $DEFAULT_WEB_INTERFACE)
}

# Main configuration flow
configure_panel

while ! test_configuration; do
    echo -e "${YELLOW}Configuration test failed. Let's adjust the settings.${NC}"
    configure_panel
done

echo -e "${GREEN}Great! Configuration test successful.${NC}"

# Create environment variables string for systemd service
ENV_VARS=""
# Driver is required, always add it
ENV_VARS+="Environment=\"LED_DRIVER=$DRIVER\"\n"

# Only add non-default values
if [ "$ROWS" != "$DEFAULT_ROWS" ]; then
    ENV_VARS+="Environment=\"LED_ROWS=$ROWS\"\n"
fi

if [ "$COLS" != "$DEFAULT_COLS" ]; then
    ENV_VARS+="Environment=\"LED_COLS=$COLS\"\n"
fi

if [ "$CHAIN_LENGTH" != "$DEFAULT_CHAIN_LENGTH" ]; then
    ENV_VARS+="Environment=\"LED_CHAIN_LENGTH=$CHAIN_LENGTH\"\n"
fi

if [ "$PARALLEL" != "$DEFAULT_PARALLEL" ]; then
    ENV_VARS+="Environment=\"LED_PARALLEL=$PARALLEL\"\n"
fi

if [ "$HARDWARE_MAPPING" != "$DEFAULT_HARDWARE_MAPPING" ]; then
    ENV_VARS+="Environment=\"LED_HARDWARE_MAPPING=$HARDWARE_MAPPING\"\n"
fi

if [ "$PWM_BITS" != "$DEFAULT_PWM_BITS" ]; then
    ENV_VARS+="Environment=\"LED_PWM_BITS=$PWM_BITS\"\n"
fi

if [ "$PWM_LSB_NANOSECONDS" != "$DEFAULT_PWM_LSB_NANOSECONDS" ]; then
    ENV_VARS+="Environment=\"LED_PWM_LSB_NANOSECONDS=$PWM_LSB_NANOSECONDS\"\n"
fi

if [ "$DITHER_BITS" != "$DEFAULT_DITHER_BITS" ]; then
    ENV_VARS+="Environment=\"LED_DITHER_BITS=$DITHER_BITS\"\n"
fi

if [ "$ROW_SETTER" != "$DEFAULT_ROW_SETTER" ]; then
    ENV_VARS+="Environment=\"LED_ROW_SETTER=$ROW_SETTER\"\n"
fi

if [ "$LED_SEQUENCE" != "$DEFAULT_LED_SEQUENCE" ]; then
    ENV_VARS+="Environment=\"LED_SEQUENCE=$LED_SEQUENCE\"\n"
fi

if [ "$LIMIT_REFRESH_RATE" != "$DEFAULT_LIMIT_REFRESH_RATE" ]; then
    ENV_VARS+="Environment=\"LED_LIMIT_REFRESH_RATE=$LIMIT_REFRESH_RATE\"\n"
fi

if [ "$MAX_BRIGHTNESS" != "$DEFAULT_MAX_BRIGHTNESS" ]; then
    ENV_VARS+="Environment=\"LED_LIMIT_MAX_BRIGHTNESS=$MAX_BRIGHTNESS\"\n"
fi

if [ "$WEB_PORT" != "$DEFAULT_WEB_PORT" ]; then
    ENV_VARS+="Environment=\"LED_PORT=$WEB_PORT\"\n"
fi

if [ "$WEB_INTERFACE" != "$DEFAULT_WEB_INTERFACE" ]; then
    ENV_VARS+="Environment=\"LED_INTERFACE=$WEB_INTERFACE\"\n"
fi

# Add optional parameters if set
if [ ! -z "$GPIO_SLOWDOWN" ]; then
    ENV_VARS+="Environment=\"LED_GPIO_SLOWDOWN=$GPIO_SLOWDOWN\"\n"
fi

if [ ! -z "$PANEL_TYPE" ]; then
    ENV_VARS+="Environment=\"LED_PANEL_TYPE=$PANEL_TYPE\"\n"
fi

if [ ! -z "$MULTIPLEXING" ]; then
    ENV_VARS+="Environment=\"LED_MULTIPLEXING=$MULTIPLEXING\"\n"
fi

if [ ! -z "$PIXEL_MAPPER" ]; then
    ENV_VARS+="Environment=\"LED_PIXEL_MAPPER=$PIXEL_MAPPER\"\n"
fi

if [ ! -z "$PI_CHIP" ]; then
    ENV_VARS+="Environment=\"LED_PI_CHIP=$PI_CHIP\"\n"
fi

# Boolean options (inverse logic for hardware pulsing)
if [ "$INTERLACED" -eq 1 ]; then
    ENV_VARS+="Environment=\"LED_INTERLACED=1\"\n"
fi

if [ "$NO_HARDWARE_PULSE" -eq 1 ]; then
    ENV_VARS+="Environment=\"LED_HARDWARE_PULSING=0\"\n"
elif [ "$NO_HARDWARE_PULSE" -ne "$DEFAULT_NO_HARDWARE_PULSE" ]; then
    ENV_VARS+="Environment=\"LED_HARDWARE_PULSING=1\"\n"
fi

if [ "$SHOW_REFRESH" -eq 1 ]; then
    ENV_VARS+="Environment=\"LED_SHOW_REFRESH=1\"\n"
fi

if [ "$INVERSE_COLORS" -eq 1 ]; then
    ENV_VARS+="Environment=\"LED_INVERSE_COLORS=1\"\n"
fi

# Create systemd service with the configuration
echo -e "${YELLOW}Creating systemd service with your configuration...${NC}"
cat > /etc/systemd/system/rpi-led-sign.service <<EOF
[Unit]
Description=RPi LED Sign Controller
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/rpi_led_sign_controller
$(echo -e $ENV_VARS)
Restart=on-failure
User=root

# Priority settings
Nice=-10
IOSchedulingClass=realtime
IOSchedulingPriority=0
CPUSchedulingPolicy=fifo
CPUSchedulingPriority=99
OOMScoreAdjust=-900

[Install]
WantedBy=multi-user.target
EOF

# Enable and start the service (this requires root)
systemctl daemon-reload
systemctl enable rpi-led-sign.service
systemctl start rpi-led-sign.service
echo -e "${GREEN}Systemd service installed and started.${NC}"

# Return to the original directory
cd $CURRENT_DIR

echo -e "${GREEN}Installation complete!${NC}"
echo -e "Web interface available at: http://$(hostname -I | awk '{print $1}'):$WEB_PORT"
echo -e "Source code is located at: ${BLUE}/usr/local/src/rpi-led-sign-controller${NC}"
echo -e "You can manage the service with: sudo systemctl [start|stop|restart|status] rpi-led-sign.service"
echo -e ""
echo -e "To update in the future, you can either:"
echo -e "  • Run this script again: ${BLUE}curl -sSL https://raw.githubusercontent.com/paviro/rpi-led-sign-controller/main/scripts/install.sh | sudo bash${NC}"
echo -e "  • Or from the source directory: ${BLUE}cd /usr/local/src/rpi-led-sign-controller && git pull && sudo bash scripts/install.sh${NC}"
echo -e ""
echo -e "To uninstall, run: ${BLUE}sudo bash /usr/local/src/rpi-led-sign-controller/scripts/uninstall.sh${NC}"
echo -e ""
echo -e "For more information, visit: ${BLUE}https://github.com/paviro/rpi-led-sign-controller${NC}"
exit 0