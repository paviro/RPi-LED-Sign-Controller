#!/bin/bash
set -e

# Colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}RPi LED Sign Controller - Uninstallation Script${NC}"
echo -e "==============================================="
echo -e "GitHub Repository: ${GREEN}https://github.com/paviro/rpi-led-sign-controller${NC}"

# Check if script is run with root privileges
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Please run as root (use sudo)${NC}"
  exit 1
fi

# Determine the actual user who ran the script
ACTUAL_USER=${SUDO_USER:-$USER}
ACTUAL_HOME=$(eval echo ~$ACTUAL_USER)

# Add explanation and confirmation
echo -e "\n${YELLOW}This uninstallation script will:${NC}"
echo -e "  • Stop and remove the RPi LED Sign Controller systemd service"
echo -e "  • Remove the application binary from /usr/local/bin"
echo -e "  • Offer to remove the source code from /usr/local/src/rpi-led-sign-controller"
echo -e "  • Check for and offer to remove data files in /var/lib/led-matrix-controller"
echo -e "  • Ask if you want to uninstall Rust and Git"
echo -e "  • Offer to clean up unused packages with apt autoremove"
echo -e "\n${YELLOW}You'll be asked to confirm each step of the process.${NC}"

# Confirm proceeding with uninstallation
read -p "Do you want to proceed with uninstallation? [y/N]: " confirm_uninstall
if [[ "$confirm_uninstall" != "y" && "$confirm_uninstall" != "Y" ]]; then
    echo -e "${GREEN}Uninstallation cancelled.${NC}"
    exit 0
fi

echo -e "${YELLOW}Starting uninstallation...${NC}"

# Function to get yes/no input
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

# Stop and disable the systemd service
if systemctl is-active --quiet rpi-led-sign.service; then
    echo -e "${YELLOW}Stopping service...${NC}"
    systemctl stop rpi-led-sign.service
fi

if systemctl is-enabled --quiet rpi-led-sign.service 2>/dev/null; then
    echo -e "${YELLOW}Disabling service...${NC}"
    systemctl disable rpi-led-sign.service
fi

# Remove the systemd service file
if [ -f /etc/systemd/system/rpi-led-sign.service ]; then
    echo -e "${YELLOW}Removing systemd service...${NC}"
    rm /etc/systemd/system/rpi-led-sign.service
    systemctl daemon-reload
    echo -e "${GREEN}Systemd service removed.${NC}"
fi

# Remove the binary
if [ -f /usr/local/bin/rpi_led_sign_controller ]; then
    echo -e "${YELLOW}Removing binary...${NC}"
    rm /usr/local/bin/rpi_led_sign_controller
    echo -e "${GREEN}Binary removed.${NC}"
fi

# Remove source code - improved to check current directory
REPO_DIR="/usr/local/src/rpi-led-sign-controller"
CURRENT_DIR=$(pwd)

# Determine if we're running from within a repository
IS_REPO_DIR=false
if [ -f "Cargo.toml" ] && grep -q "rpi_led_sign_controller" "Cargo.toml" 2>/dev/null; then
    IS_REPO_DIR=true
    echo -e "${YELLOW}Running from within a repository directory.${NC}"
fi

# Check if we're in a scripts subdirectory of a repository
if [ -f "../Cargo.toml" ] && grep -q "rpi_led_sign_controller" "../Cargo.toml" 2>/dev/null; then
    IS_REPO_DIR=true
    CURRENT_DIR=$(cd .. && pwd)
    echo -e "${YELLOW}Running from scripts directory of a repository.${NC}"
fi

# Only offer to remove the standard repo location if:
# 1. It exists AND
# 2. We're not currently in it
if [ -d "$REPO_DIR" ] && [ "$CURRENT_DIR" != "$REPO_DIR" ]; then
    echo -e "${YELLOW}Found source code at $REPO_DIR${NC}"
    REMOVE_SOURCE=$(get_yes_no "Do you want to remove the source code?" "y")
    
    if [ "$REMOVE_SOURCE" -eq 1 ]; then
        echo -e "${YELLOW}Removing source code...${NC}"
        rm -rf $REPO_DIR
        echo -e "${GREEN}Source code removed.${NC}"
    else
        echo -e "${BLUE}Source code kept at $REPO_DIR${NC}"
    fi
elif [ -d "$REPO_DIR" ] && [ "$CURRENT_DIR" = "$REPO_DIR" ]; then
    echo -e "${YELLOW}Currently in the source code directory at $REPO_DIR${NC}"
    echo -e "${BLUE}Cannot remove the directory you're currently in.${NC}"
    echo -e "${BLUE}The source code will remain at $REPO_DIR${NC}"
fi

# If running from a non-standard repo location, inform the user
if [ "$IS_REPO_DIR" = true ] && [ "$CURRENT_DIR" != "$REPO_DIR" ]; then
    echo -e "${YELLOW}You seem to be running this script from a non-standard repository location:${NC}"
    echo -e "${BLUE}$CURRENT_DIR${NC}"
    echo -e "${YELLOW}This directory will not be automatically removed.${NC}"
fi

# Check for data directory
DATA_DIR="/var/lib/led-matrix-controller"
if [ -d "$DATA_DIR" ]; then
    echo -e "${YELLOW}Found data directory at $DATA_DIR${NC}"
    REMOVE_DATA=$(get_yes_no "Do you want to remove the data directory? This will delete all playlists and custom content." "n")
    
    if [ "$REMOVE_DATA" -eq 1 ]; then
        echo -e "${YELLOW}Removing data directory...${NC}"
        rm -rf $DATA_DIR
        echo -e "${GREEN}Data directory removed.${NC}"
    else
        echo -e "${BLUE}Data directory kept at $DATA_DIR${NC}"
    fi
fi

# Ask about uninstalling Rust
echo -e "\n${BLUE}Rust Uninstallation${NC}"
if sudo -u $ACTUAL_USER bash -c "source $ACTUAL_HOME/.cargo/env 2>/dev/null && command -v rustc &> /dev/null && command -v cargo &> /dev/null"; then
    REMOVE_RUST=$(get_yes_no "Do you want to uninstall Rust?" "n")
    
    if [ "$REMOVE_RUST" -eq 1 ]; then
        echo -e "${YELLOW}Uninstalling Rust for user $ACTUAL_USER...${NC}"
        if [ -f "$ACTUAL_HOME/.cargo/bin/rustup" ]; then
            sudo -u $ACTUAL_USER bash -c "$ACTUAL_HOME/.cargo/bin/rustup self uninstall -y"
            echo -e "${GREEN}Rust uninstalled successfully.${NC}"
        else
            echo -e "${RED}Rustup not found. Please uninstall Rust manually.${NC}"
        fi
    else
        echo -e "${BLUE}Keeping Rust installation.${NC}"
    fi
else
    echo -e "${GREEN}Rust is not installed for user $ACTUAL_USER.${NC}"
fi

# Ask about uninstalling Git
echo -e "\n${BLUE}Git Uninstallation${NC}"
if command -v git &> /dev/null; then
    REMOVE_GIT=$(get_yes_no "Do you want to uninstall Git?" "n")
    
    if [ "$REMOVE_GIT" -eq 1 ]; then
        echo -e "${YELLOW}Uninstalling Git...${NC}"
        apt-get remove -y git
        echo -e "${GREEN}Git uninstalled successfully.${NC}"
    else
        echo -e "${BLUE}Keeping Git installation.${NC}"
    fi
else
    echo -e "${GREEN}Git is not installed.${NC}"
fi

# Ask about running autoremove to clean up unused dependencies
echo -e "\n${BLUE}System Cleanup${NC}"
RUN_AUTOREMOVE=$(get_yes_no "Do you want to run apt autoremove to clean up unused packages?" "n") 

if [ "$RUN_AUTOREMOVE" -eq 1 ]; then
    echo -e "${YELLOW}Running apt autoremove...${NC}"
    apt-get autoremove -y
    echo -e "${GREEN}System cleaned up successfully.${NC}"
else
    echo -e "${BLUE}Skipping system cleanup.${NC}"
fi

echo -e "\n${GREEN}Uninstallation complete!${NC}"
echo -e "The RPi LED Sign Controller has been removed from your system."
echo -e "For more information, visit: ${BLUE}https://github.com/paviro/rpi-led-sign-controller${NC}"
exit 0 