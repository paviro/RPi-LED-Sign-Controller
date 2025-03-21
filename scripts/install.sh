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
echo -e "  • Check for and install required dependencies (Git, Rust, Node.js)"
echo -e "  • Clone the repository if needed"
echo -e "  • Build the application from source"
echo -e "  • Help you configure your LED panel"
echo -e "  • Install the application as a systemd service"
echo -e "  • Start the service automatically on boot"

echo -e "\n${YELLOW}This script will make changes to your system.${NC}"
echo -e "${YELLOW}Do you want to proceed with the installation/update?${NC}"

read -p "Continue? [y/N]: " continue_install
if [[ "$continue_install" != "y" && "$continue_install" != "Y" ]]; then
    echo -e "${RED}Installation aborted.${NC}"
    exit 1
fi
echo -e "${GREEN}Proceeding with installation...${NC}"

# First, add a helper function for standardized reading near the top of the script
read_input() {
    local prompt="$1"
    local var_name="$2"
    local result
    
    if [ -t 0 ]; then
        # Terminal is interactive, read normally
        read -p "$prompt" result
    else
        # Running from pipe or non-interactive, use /dev/tty
        read -p "$prompt" result </dev/tty
    fi
    
    # Use eval to set the variable by name in the parent scope
    eval "$var_name=\"\$result\""
}

# Then update the Raspberry Pi detection override
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

# Add the reconfigure function here, before it's used
ask_reconfigure() {
    local reason=$1  # Why we're asking (update/no update)
    local default="N"
    
    if [ "$reason" == "update" ]; then
        echo -e "\n${GREEN}✓ Update successful!${NC}"
        echo -e "${YELLOW}Would you like to modify your LED panel configuration?${NC}"
    else
        echo -e "\n${GREEN}✓ The RPi LED Sign Controller is already installed and up to date.${NC}"
        echo -e "${YELLOW}Would you like to modify your LED panel configuration?${NC}"
    fi
    
    if [ -t 0 ]; then
        # Terminal is interactive, read normally
        read -p "Reconfigure LED panel settings? [y/N]: " reconfigure
    else
        # Running from pipe or non-interactive, use /dev/tty
        read -p "Reconfigure LED panel settings? [y/N]: " reconfigure </dev/tty
    fi
    
    if [[ "$reconfigure" != "y" && "$reconfigure" != "Y" ]]; then
        if [ "$reason" == "update" ]; then
            echo -e "${GREEN}Keeping existing configuration.${NC}"
            echo -e "${YELLOW}Restarting service with updated binary...${NC}"
            systemctl restart rpi-led-sign.service
            echo -e "${GREEN}Service restarted successfully.${NC}"
        else
            echo -e "${GREEN}No changes needed. Your installation will continue to use the existing settings.${NC}"
        fi
        
        # Display common completion information
        echo -e "\n${GREEN}===== RPi LED Sign Controller Information =====${NC}"
        echo -e "Web interface available at: http://$(hostname -I | awk '{print $1}'):$(systemctl show rpi-led-sign.service -p Environment | grep LED_PORT | sed 's/.*LED_PORT=\([0-9]*\).*/\1/' || echo "3000")"
        echo -e "Source code is located at: ${BLUE}/usr/local/src/rpi-led-sign-controller${NC}"
        echo -e "You can manage the service with: sudo systemctl [start|stop|restart|status] rpi-led-sign.service"
        echo -e ""
        echo -e "${BLUE}===== Update & Maintenance =====${NC}"
        echo -e "To update in the future, you can either:"
        echo -e "  • Run this script again: ${BLUE}curl -sSL https://raw.githubusercontent.com/paviro/rpi-led-sign-controller/main/scripts/install.sh | sudo bash${NC}"
        echo -e "  • Or from the source directory: ${BLUE}cd /usr/local/src/rpi-led-sign-controller && sudo bash scripts/install.sh${NC}"
        echo -e ""
        echo -e "To uninstall, run: ${BLUE}sudo bash /usr/local/src/rpi-led-sign-controller/scripts/uninstall.sh${NC}"
        echo -e ""
        echo -e "For more information, visit: ${BLUE}https://github.com/paviro/rpi-led-sign-controller${NC}"
        return 1  # Don't reconfigure
    fi
    
    echo -e "${YELLOW}Proceeding with reconfiguration...${NC}"
    
    # Stop the service before reconfiguration if it's running
    if systemctl is-active --quiet rpi-led-sign.service; then
        echo -e "${YELLOW}Stopping service before reconfiguration...${NC}"
        systemctl stop rpi-led-sign.service
    fi
    return 0  # Reconfigure
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

# Check for and install Node.js if necessary
if ! command -v node &> /dev/null || ! command -v npm &> /dev/null; then
    echo -e "${YELLOW}Node.js not found. Installing Node.js...${NC}"
    apt-get update
    apt-get install -y nodejs npm
    echo -e "${GREEN}Node.js installed successfully.${NC}"
else
    echo -e "${GREEN}Node.js is already installed.${NC}"
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

# Check if the app is already installed
APP_INSTALLED=0
if [ -f "/usr/local/bin/rpi_led_sign_controller" ]; then
    APP_INSTALLED=1
    echo -e "${GREEN}RPi LED Sign Controller is already installed.${NC}"
else
    echo -e "${YELLOW}RPi LED Sign Controller is not yet installed.${NC}"
fi

# Check if we're inside the repository directory
INSIDE_REPO=0
# Either we're directly in the repo dir
if [ -f "Cargo.toml" ] && grep -q "rpi_led_sign_controller" "Cargo.toml" 2>/dev/null; then
    INSIDE_REPO=1
    REPO_DIR=$(pwd)
    echo -e "${BLUE}Already in project directory.${NC}"
# Or we're in the scripts subdirectory
elif [ -f "../Cargo.toml" ] && grep -q "rpi_led_sign_controller" "../Cargo.toml" 2>/dev/null; then
    INSIDE_REPO=1
    REPO_DIR=$(cd .. && pwd)
    echo -e "${BLUE}Already in project directory (scripts subfolder).${NC}"
fi

# Set standard repository location if not already inside repo
if [ $INSIDE_REPO -eq 0 ]; then
    REPO_DIR="/usr/local/src/rpi-led-sign-controller"
fi

# Check if repo dir exists and fix ownership if needed
if [ -d "$REPO_DIR" ]; then
    # Check if any files in the repo have incorrect ownership
    if [ "$(find "$REPO_DIR" -not -user $ACTUAL_USER | wc -l)" -gt 0 ]; then
        echo -e "${YELLOW}Fixing repository permissions...${NC}"
        echo -e "${BLUE}This ensures your user can pull updates from GitHub${NC}"
        chown -R $ACTUAL_USER:$ACTUAL_USER "$REPO_DIR"
        echo -e "${GREEN}Repository permissions fixed.${NC}"
    fi
fi

# Determine if we need to clone or navigate to the repository
if [ $INSIDE_REPO -eq 0 ]; then
    # We're not in the repo directory, check if it exists at the standard location
    if [ -d "$REPO_DIR" ]; then
        echo -e "${BLUE}Found existing repository at $REPO_DIR${NC}"
        cd "$REPO_DIR"
    else
        echo -e "${YELLOW}Creating repository directory...${NC}"
        mkdir -p "$REPO_DIR"
        chown $ACTUAL_USER:$ACTUAL_USER "$REPO_DIR"
        
        echo -e "${YELLOW}Cloning repository as user $ACTUAL_USER...${NC}"
        # Clone the repository as the regular user
        sudo -u $ACTUAL_USER git clone https://github.com/paviro/rpi-led-sign-controller.git "$REPO_DIR"
        cd "$REPO_DIR"
    fi
fi

# If app is installed, always check for updates
if [ $APP_INSTALLED -eq 1 ]; then
    echo -e "${YELLOW}Fetching the latest changes from GitHub...${NC}"
    sudo -u $ACTUAL_USER git fetch

    # Now check if we're behind the remote repository
    UPDATES_AVAILABLE=0
    git_status=$(sudo -u $ACTUAL_USER git status -uno)
    if echo "$git_status" | grep -q "Your branch is behind"; then
        UPDATES_AVAILABLE=1
        echo -e "${YELLOW}Updates are available.${NC}"
        
        # Pull changes as the regular user
        sudo -u $ACTUAL_USER git pull
        echo -e "${GREEN}Repository updated successfully.${NC}"
    else
        echo -e "${GREEN}Repository is already up to date.${NC}"
    fi
    
    # Create update marker file with proper ownership
    if [ $UPDATES_AVAILABLE -eq 1 ]; then
        echo "updated=$(date +%s)" > "$REPO_DIR/.update_status"
        chown $ACTUAL_USER:$ACTUAL_USER "$REPO_DIR/.update_status"
    fi
    
    # Only return to original directory if we don't need to build
    # This is critical - we need to stay in the repo dir for building
    if [ "$UPDATES_AVAILABLE" -eq 0 ] && [ ! -f "$REPO_DIR/.update_status" ]; then
        if [ "$CURRENT_DIR" != "$REPO_DIR" ]; then
            cd "$CURRENT_DIR"
        fi
    fi
fi

# Add code to ensure UPDATE_MARKER variable is defined
UPDATE_MARKER="$REPO_DIR/.update_status"

# Record the project directory
PROJECT_DIR=$(pwd)

# Set frontend repository location
FRONTEND_REPO_DIR="/usr/local/src/rpi-led-sign-controller-frontend"

# Check if we're inside the backend repository
INSIDE_BACKEND_REPO=0
# Either we're directly in the repo dir
if [ -f "Cargo.toml" ] && grep -q "rpi_led_sign_controller" "Cargo.toml" 2>/dev/null; then
    INSIDE_BACKEND_REPO=1
    REPO_DIR=$(pwd)
    echo -e "${BLUE}Already in backend project directory.${NC}"
# Or we're in the scripts subdirectory
elif [ -f "../Cargo.toml" ] && grep -q "rpi_led_sign_controller" "../Cargo.toml" 2>/dev/null; then
    INSIDE_BACKEND_REPO=1
    REPO_DIR=$(cd .. && pwd)
    echo -e "${BLUE}Already in backend project directory (scripts subfolder).${NC}"
fi

# Set standard repository location if not already inside repo
if [ $INSIDE_BACKEND_REPO -eq 0 ]; then
    REPO_DIR="/usr/local/src/rpi-led-sign-controller"
fi

# Now handle the frontend repository
echo -e "${YELLOW}Checking for frontend repository...${NC}"
FRONTEND_REPO_EXISTS=0
if [ -d "$FRONTEND_REPO_DIR" ]; then
    FRONTEND_REPO_EXISTS=1
    echo -e "${GREEN}Frontend repository already exists at $FRONTEND_REPO_DIR${NC}"
    
    # Check if any files in the frontend repo have incorrect ownership
    if [ "$(find "$FRONTEND_REPO_DIR" -not -user $ACTUAL_USER | wc -l)" -gt 0 ]; then
        echo -e "${YELLOW}Fixing frontend repository permissions...${NC}"
        chown -R $ACTUAL_USER:$ACTUAL_USER "$FRONTEND_REPO_DIR"
        echo -e "${GREEN}Frontend repository permissions fixed.${NC}"
    fi
else
    echo -e "${YELLOW}Frontend repository not found. Will clone it.${NC}"
fi

# Clone frontend repository if it doesn't exist
if [ $FRONTEND_REPO_EXISTS -eq 0 ]; then
    echo -e "${YELLOW}Creating frontend repository directory...${NC}"
    mkdir -p "$FRONTEND_REPO_DIR"
    chown $ACTUAL_USER:$ACTUAL_USER "$FRONTEND_REPO_DIR"
    
    echo -e "${YELLOW}Cloning frontend repository as user $ACTUAL_USER...${NC}"
    # Clone the repository as the regular user
    sudo -u $ACTUAL_USER git clone https://github.com/paviro/RPi-LED-Sign-Controller-Frontend.git "$FRONTEND_REPO_DIR"
    echo -e "${GREEN}Frontend repository cloned successfully.${NC}"
fi

# Initialize update flags with default values
BACKEND_UPDATES_AVAILABLE=0
FRONTEND_UPDATES_AVAILABLE=0
FRONTEND_REBUILD_NEEDED=0

# Check for backend updates
if [ $APP_INSTALLED -eq 1 ]; then
    echo -e "${YELLOW}Fetching the latest changes from GitHub for backend...${NC}"
    cd "$REPO_DIR"
    sudo -u $ACTUAL_USER git fetch

    # Now check if we're behind the remote repository
    git_status=$(sudo -u $ACTUAL_USER git status -uno)
    if echo "$git_status" | grep -q "Your branch is behind"; then
        BACKEND_UPDATES_AVAILABLE=1
        echo -e "${YELLOW}Backend updates are available.${NC}"
        
        # Pull changes as the regular user
        sudo -u $ACTUAL_USER git pull
        echo -e "${GREEN}Backend repository updated successfully.${NC}"
    else
        echo -e "${GREEN}Backend repository is already up to date.${NC}"
    fi
    
    # Create update marker file for backend with proper ownership
    if [ $BACKEND_UPDATES_AVAILABLE -eq 1 ]; then
        echo "updated=$(date +%s)" > "$REPO_DIR/.update_status"
        chown $ACTUAL_USER:$ACTUAL_USER "$REPO_DIR/.update_status"
    fi
fi

# Check for frontend updates
echo -e "${YELLOW}Checking for frontend updates...${NC}"

# Only if frontend repo exists, check for updates
if [ $FRONTEND_REPO_EXISTS -eq 1 ]; then
    cd "$FRONTEND_REPO_DIR"
    sudo -u $ACTUAL_USER git fetch
    
    git_status=$(sudo -u $ACTUAL_USER git status -uno)
    if echo "$git_status" | grep -q "Your branch is behind"; then
        FRONTEND_UPDATES_AVAILABLE=1
        echo -e "${YELLOW}Frontend updates are available.${NC}"
        
        # Pull changes as the regular user
        sudo -u $ACTUAL_USER git pull
        echo -e "${GREEN}Frontend repository updated successfully.${NC}"
        FRONTEND_REBUILD_NEEDED=1
    else
        echo -e "${GREEN}Frontend repository is already up to date.${NC}"
    fi
else
    # If frontend was newly cloned, we need to build it
    FRONTEND_REBUILD_NEEDED=1
fi

# Check if frontend has already been compiled and copied - with improved detection for deleted files
FRONTEND_FILES_EXIST=0
if [ -d "$REPO_DIR/static" ] && [ -d "$REPO_DIR/static/_next" ] && [ "$(ls -A "$REPO_DIR/static" 2>/dev/null)" ]; then
    # Check if the static directory has actual content and wasn't emptied by an update
    echo -e "${GREEN}Frontend files already exist in backend static directory.${NC}"
    FRONTEND_FILES_EXIST=1
else
    # Static directory doesn't exist, is empty, or doesn't have the Next.js build files
    echo -e "${YELLOW}Frontend files missing or incomplete in backend static directory.${NC}"
    # Force rebuild of frontend
    FRONTEND_REBUILD_NEEDED=1
fi

# Build the frontend if needed or if backend was updated or if frontend files don't exist
if [ $FRONTEND_REBUILD_NEEDED -eq 1 ] || [ $BACKEND_UPDATES_AVAILABLE -eq 1 ] || [ $FRONTEND_FILES_EXIST -eq 0 ]; then
    echo -e "${YELLOW}Building frontend...${NC}"
    cd "$FRONTEND_REPO_DIR"
    
    # Install dependencies and build
    echo -e "${YELLOW}Installing frontend dependencies...${NC}"
    sudo -u $ACTUAL_USER npm install
    
    echo -e "${YELLOW}Building frontend...${NC}"
    sudo -u $ACTUAL_USER npm run build
    
    echo -e "${GREEN}Frontend built successfully.${NC}"
    
    # Make sure static directory exists
    mkdir -p "$REPO_DIR/static"
    
    # Copy the built files to the backend's static folder
    echo -e "${YELLOW}Copying frontend files to backend...${NC}"
    cp -r "$FRONTEND_REPO_DIR/out/"* "$REPO_DIR/static/"
    echo -e "${GREEN}Frontend files copied successfully.${NC}"
else
    echo -e "${GREEN}Skipping frontend build as files already exist and no updates were found.${NC}"
fi

# Build the application if new installation, update pulled, or rebuild requested
if [ "$BACKEND_UPDATES_AVAILABLE" -eq 1 ] || [ ! -f "/usr/local/bin/rpi_led_sign_controller" ] || [ -f "$REPO_DIR/.update_status" ]; then
    # Make sure we're in the repository directory
    if [ "$(pwd)" != "$REPO_DIR" ]; then
        echo -e "${YELLOW}Changing to repository directory for build...${NC}"
        cd "$REPO_DIR"
    fi

    echo -e "${YELLOW}Building backend application...${NC}"
    # Use the user's cargo environment
    sudo -u $ACTUAL_USER bash -c "source $ACTUAL_HOME/.cargo/env && cargo build --release"
    echo -e "${GREEN}Backend build completed.${NC}"

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
    if [ -f "$REPO_DIR/.update_status" ]; then
        rm "$REPO_DIR/.update_status"
    fi
    
    # After binary update section
    if [ -f "/etc/systemd/system/rpi-led-sign.service" ] && [ "$BACKEND_UPDATES_AVAILABLE" -eq 1 -o "$FRONTEND_UPDATES_AVAILABLE" -eq 1 ]; then
        if ! ask_reconfigure "update"; then
            exit 0
        fi
        # Continue with configuration
    fi
fi

# Check if we need to ask for reconfiguration when there were no updates or it's a fresh install
if [ $APP_INSTALLED -eq 1 ] && [ $BACKEND_UPDATES_AVAILABLE -eq 0 ] && [ $FRONTEND_UPDATES_AVAILABLE -eq 0 ]; then
    if ! ask_reconfigure "no_update"; then
        exit 0
    fi
    # Continue with configuration
fi

# If it's a fresh installation, always ask to configure
if [ $APP_INSTALLED -eq 0 ]; then
    echo -e "${GREEN}Fresh installation completed. Now let's configure your LED panel.${NC}"
    # Continue with configuration - no exit option here as configuration is required for first install
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

# Update the get_input function
get_input() {
    local prompt=$1
    local default=$2
    local value
    
    read -p "${prompt} [${default}]: " value
    echo ${value:-$default}
}

# Update the get_yes_no function
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
    
    # For the configuration test
    if [ -t 0 ]; then
        read -p "Did the LED panel display correctly? (y/n): " is_working
    else
        read -p "Did the LED panel display correctly? (y/n): " is_working </dev/tty
    fi
    
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
echo -e "  • Or from the source directory: ${BLUE}cd /usr/local/src/rpi-led-sign-controller && sudo bash scripts/install.sh${NC}"
echo -e ""
echo -e "To uninstall, run: ${BLUE}sudo bash /usr/local/src/rpi-led-sign-controller/scripts/uninstall.sh${NC}"
echo -e ""
echo -e "For more information, visit: ${BLUE}https://github.com/paviro/rpi-led-sign-controller${NC}"
exit 0