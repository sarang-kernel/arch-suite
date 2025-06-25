#!/bin/bash
set -euo pipefail

# =============================
# 🚀 Arch System Suite - ENGINE v1.5.0
# This script is called by the Rust TUI to perform system-level tasks.
# =============================

# --- SELF-CONTAINED DEPENDENCY INSTALLER (BOOTSTRAP) ---
ensure_dependencies() {
    # This function runs if the script is executed directly, not via a package manager.
    # It ensures the environment has the necessary tools.
    if ! command -v gum &> /dev/null; then
        echo "The 'gum' TUI utility is not installed. It is required to continue."
        read -p "Would you like to attempt to install it now? (y/N) " choice
        case "$choice" in
            y|Y ) sudo pacman -Syu --noconfirm gum;;
            * ) echo "Aborting."; exit 1;;
        esac
    fi

    local deps=(bash gum arch-install-scripts pacman-contrib gptfdisk dosfstools e2fsprogs archiso rsync pciutils)
    local missing=()
    for dep in "${deps[@]}"; do
        # Check if the package is installed
        if ! pacman -Q "$dep" &> /dev/null; then
            missing+=("$dep")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        gum style --border double --foreground 212 "Missing Dependencies" \
            "The following required packages are missing: ${missing[*]}"
        
        if gum confirm "Would you like to install them now?"; then
            sudo pacman -Syu --noconfirm --needed "${missing[@]}"
        else
            gum style --foreground 196 "Cannot proceed without dependencies. Aborting."
            exit 1
        fi
    fi
}

# --- CONFIGURATION & DEFAULTS ---
CURRENT_USER_NAME="${SUDO_USER:-${USER}}"
USER_HOME_DIR=$(getent passwd "$CURRENT_USER_NAME" | cut -d: -f6)
WORKDIR="${USER_HOME_DIR}/arch-suite-work"
MOUNT_POINT="/mnt"
HARDWARE_DRIVERS_FILE="$WORKDIR/detected-drivers.txt"

# --- TRAP FOR SAFE EXIT ---
trap 'echo -e "\n[!] Interrupted. Cleaning up..."; umount -R "$MOUNT_POINT" &>/dev/null; exit 1' SIGINT SIGTERM

# --- HELPER FUNCTIONS ---
select_disk() { gum style --foreground "#00ffff" "Select the target disk:"; local DRIVE_LIST; DRIVE_LIST=$(lsblk -dpno NAME,SIZE,MODEL,TRAN | grep -vE "(loop|sr[0-9])"); INSTALL_TARGET=$(echo "$DRIVE_LIST" | gum choose --header="Choose a disk:" --no-limit); [[ -z "$INSTALL_TARGET" ]] && return 1; return 0; }
wipe_disk() { gum confirm "⚠️ Wipe ALL data on $INSTALL_TARGET? This is irreversible!" || return 1; umount -R "${INSTALL_TARGET}"* &>/dev/null || true; wipefs -a "$INSTALL_TARGET"; sgdisk --zap-all "$INSTALL_TARGET"; }
pause() { read -rp "Press Enter to continue..."; }

# =================================================
# REPLICATOR / CLONER / UTILITIES / HELP LOGIC
# (These functions are unchanged from the previous complete version)
# =================================================
replicator_menu() {
    clear
    CHOICE=$(gum choose "📸 Create System Snapshot" "🛰️ Deploy System from Snapshot" "⬅️ Back")
    case "$CHOICE" in
        "📸 Create System Snapshot") create_system_snapshot ;;
        "🛰️ Deploy System from Snapshot") deploy_system_snapshot ;;
    esac
}

create_system_snapshot() {
    mkdir -p "$WORKDIR/snapshot_tmp"
    local SNAPSHOT_DIR="$WORKDIR/snapshot_tmp"
    local SNAPSHOT_FILE="$WORKDIR/snapshot-$(date +%Y%m%d).tar.gz"
    gum style --foreground=6 "Gathering system information..."
    pacman -Qqe > "$SNAPSHOT_DIR/packages.x86_64.txt"
    pacman -Qqm > "$SNAPSHOT_DIR/packages.foreign.txt"
    sudo tar -czf "$SNAPSHOT_DIR/etc.tar.gz" /etc
    tar -czf "$SNAPSHOT_DIR/home.tar.gz" -C "$USER_HOME_DIR" --exclude='.cache' --exclude='.Trash' .
    systemctl list-unit-files --state=enabled --no-pager | awk '{print $1}' > "$SNAPSHOT_DIR/services.enabled.txt"
    gum spin --spinner dot --title="Creating final snapshot file..." -- tar -czf "$SNAPSHOT_FILE" -C "$SNAPSHOT_DIR" .
    sudo rm -rf "$SNAPSHOT_DIR"
    sudo chown "$CURRENT_USER_NAME:$CURRENT_USER_NAME" "$SNAPSHOT_FILE"
    gum style --border double --margin 1 --foreground=7 "✅ Snapshot created: $SNAPSHOT_FILE"; pause
}

deploy_system_snapshot() {
    clear; gum style --border double --margin 1 "Deploy Pre-flight Check"
    if ! ping -c 1 archlinux.org &>/dev/null; then gum style --foreground=196 "❌ Internet connection required."; sleep 3; return; fi
    if [[ $EUID -ne 0 ]]; then gum style --foreground=196 "❌ Deployment must be run as root."; sleep 3; return; fi
    echo "✅ Checks passed."
    gum style --foreground=6 "Please locate your snapshot file..."
    local SNAPSHOT_FILE; SNAPSHOT_FILE=$(gum file .); [[ -z "$SNAPSHOT_FILE" ]] && return 1
    if ! file "$SNAPSHOT_FILE" | grep -q "gzip compressed data"; then gum style --foreground=196 "❌ Invalid snapshot file."; sleep 3; return; fi
    select_disk || return; wipe_disk || return
    sgdisk -n 1:0:+512M -t 1:ef00 -c 1:EFI "$INSTALL_TARGET"
    sgdisk -n 2:0:0 -t 2:8300 -c 2:Root "$INSTALL_TARGET"
    local EFI_PART="${INSTALL_TARGET}p1"; local ROOT_PART="${INSTALL_TARGET}p2"
    mkfs.fat -F32 "$EFI_PART"; mkfs.ext4 -F "$ROOT_PART"
    mount "$ROOT_PART" "$MOUNT_POINT"; mkdir -p "$MOUNT_POINT/boot/efi"; mount "$EFI_PART" "$MOUNT_POINT/boot/efi"
    local PACKAGES_TO_INSTALL="base linux linux-firmware networkmanager grub"
    if [[ -f "$HARDWARE_DRIVERS_FILE" ]]; then PACKAGES_TO_INSTALL+=" $(cat "$HARDWARE_DRIVERS_FILE")"; gum style --foreground=7 "Found hardware profile. Adding drivers: $(cat "$HARDWARE_DRIVERS_FILE")"; fi
    gum spin --spinner dot --title="Installing base system & drivers..." -- pacstrap "$MOUNT_POINT" $PACKAGES_TO_INSTALL
    genfstab -U "$MOUNT_POINT" >> "$MOUNT_POINT/etc/fstab"
    mkdir -p "$MOUNT_POINT/tmp/snapshot"; tar -xzf "$SNAPSHOT_FILE" -C "$MOUNT_POINT/tmp/snapshot"
    local SNAPSHOT_DIR="/tmp/snapshot"
    gum spin --spinner dot --title="Restoring /etc..." -- arch-chroot "$MOUNT_POINT" tar -xzf "${SNAPSHOT_DIR}/etc.tar.gz" -C /
    gum spin --spinner dot --title="Installing Pacman packages..." -- arch-chroot "$MOUNT_POINT" pacman -S --noconfirm --needed - < "${MOUNT_POINT}${SNAPSHOT_DIR}/packages.x86_64.txt"
    local NEW_USER; NEW_USER=$(gum input --placeholder "Confirm username to create (e.g., $CURRENT_USER_NAME)")
    local NEW_PASS; NEW_PASS=$(gum input --password --placeholder "Enter password for $NEW_USER")
    arch-chroot "$MOUNT_POINT" /bin/bash -c "useradd -m -G wheel '$NEW_USER'; echo -e '$NEW_PASS\n$NEW_PASS' | passwd '$NEW_USER'; sed -i 's/# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /etc/sudoers; sudo -u '$NEW_USER' tar -xzf '${SNAPSHOT_DIR}/home.tar.gz' -C '/home/$NEW_USER'; pacman -S --noconfirm --needed git base-devel; cd /tmp && sudo -u '$NEW_USER' git clone https://aur.archlinux.org/yay.git && cd yay && sudo -u '$NEW_USER' makepkg -si --noconfirm; sudo -u '$NEW_USER' yay -S --noconfirm --needed - < ${SNAPSHOT_DIR}/packages.foreign.txt; while read -r service; do [[ -n \"\$service\" ]] && systemctl enable \"\$service\"; done < ${SNAPSHOT_DIR}/services.enabled.txt; grub-install --target=x86_64-efi --efi-directory=/boot/efi --bootloader-id=GRUB; grub-mkconfig -o /boot/grub/grub.cfg"
    rm -rf "$MOUNT_POINT/tmp/snapshot"; umount -R "$MOUNT_POINT"
    gum style --border double --margin 1 --foreground=7 "✅ Deployment Complete!"; pause
}

cloner_menu() {
    clear
    CHOICE=$(gum choose "📀 Create Bootable ISO from Current System" "⬅️ Back")
    case "$CHOICE" in
        "📀 Create Bootable ISO from Current System") create_bootable_iso ;;
    esac
}

create_bootable_iso() {
    local PROFILE_NAME="arch-clone-profile"; local ISO_OUTPUT_DIR="$WORKDIR/out"
    mkdir -p "$WORKDIR" && cd "$WORKDIR"; cp -r /usr/share/archiso/configs/releng/ "$PROFILE_NAME" && cd "$PROFILE_NAME"
    rm -rf airootfs; mkdir -p airootfs
    gum spin --spinner dot --title="Syncing system files..." -- sudo rsync -aAXH --exclude={"/dev/*","/proc/*","/sys/*","/tmp/*","/home/*/.cache"} / ./airootfs
    pacman -Qqe > packages.x86_64
    gum spin --spinner dot --title="Building ISO..." -- sudo mkarchiso -v -w "$WORKDIR/work" -o "$ISO_OUTPUT_DIR" .
    gum style --border double --margin 1 --foreground=7 "✅ ISO created in $ISO_OUTPUT_DIR"; pause
}

utilities_menu() {
    clear
    CHOICE=$(gum choose "🔬 System Inspector & Prep" "💾 Flash ISO to USB" "🔧 Manual Installation Tools" "🗃️ Quick Backups" "🧹 Clean Workspace" "⬅️ Back")
    case "$CHOICE" in
        "🔬 System Inspector & Prep") system_inspector ;;
        "💾 Flash ISO to USB") flash_iso ;;
        "🔧 Manual Installation Tools") manual_install_menu ;;
        "🗃️ Quick Backups") quick_backup_menu ;;
        "🧹 Clean Workspace") clean_workspace ;;
    esac
}

flash_iso() {
    clear; gum style --border double "ISO to USB Flasher"
    if [[ $EUID -ne 0 ]]; then gum style --foreground=196 "❌ This action requires root privileges. Please run with 'sudo'."; sleep 3; return; fi
    gum style --foreground=6 "Please select the ISO file to flash..."
    local ISO_PATH; ISO_PATH=$(gum file --file-type=.iso .)
    if [[ ! -f "$ISO_PATH" ]]; then gum style --foreground=208 "No ISO file selected. Aborting."; sleep 2; return; fi
    select_disk || return
    local ISO_FILENAME; ISO_FILENAME=$(basename "$ISO_PATH")
    gum style --border normal --padding 1 --margin 1 "You are about to perform a destructive action:" "Source:      $ISO_FILENAME" "Destination: $INSTALL_TARGET"
    if gum confirm "⚠️ This will ERASE ALL DATA on $INSTALL_TARGET. Are you sure?"; then
        gum spin --spinner dot --title "Flashing... (This may take several minutes)" -- sudo dd if="$ISO_PATH" of="$INSTALL_TARGET" bs=4M status=progress oflag=sync
        gum style --foreground=7 "✅ Flash complete."
    else
        gum style --foreground=208 "Flash operation cancelled."
    fi
    pause
}

system_inspector() {
    clear; gum style --border double "System Inspector & Hardware Prep"
    local CPU_VENDOR; CPU_VENDOR=$(grep -m 1 "vendor_id" /proc/cpuinfo | awk '{print $3}'); local CPU_DRIVERS=""
    if [[ "$CPU_VENDOR" == "AuthenticAMD" ]]; then CPU_DRIVERS="amd-ucode"; fi
    if [[ "$CPU_VENDOR" == "GenuineIntel" ]]; then CPU_DRIVERS="intel-ucode"; fi
    local GPU_VENDOR; GPU_VENDOR=$(lspci | grep -E "VGA|3D" | awk '{print $5}'); local GPU_DRIVERS=""
    if [[ "$GPU_VENDOR" == "NVIDIA" ]]; then GPU_DRIVERS="nvidia-dkms"; fi
    if [[ "$GPU_VENDOR" == "Advanced" ]]; then GPU_DRIVERS="mesa lib32-mesa vulkan-radeon lib32-vulkan-radeon"; fi
    if [[ "$GPU_VENDOR" == "Intel" ]]; then GPU_DRIVERS="mesa lib32-mesa vulkan-intel lib32-vulkan-intel"; fi
    { echo -e "COMPONENT\tDETECTED\tRECOMMENDED DRIVERS"; echo -e "CPU\t$CPU_VENDOR\t$CPU_DRIVERS"; echo -e "GPU\t$GPU_VENDOR\t$GPU_DRIVERS"; } | gum table
    if gum confirm "Save these drivers for the next deployment?"; then mkdir -p "$WORKDIR"; echo "$CPU_DRIVERS $GPU_DRIVERS" > "$HARDWARE_DRIVERS_FILE"; gum style --foreground=7 "✅ Driver profile saved."; fi
    pause
}

manual_install_menu() {
    clear; gum style --border double "Manual Installation Tools (Advanced)"
    select_disk || return
    while true; do
        CHOICE=$(gum choose "Wipe Disk" "Partition (Simple EFI + Root)" "Format Partitions" "Mount Partitions" "Install Base System" "Setup Bootloader" "⬅️ Back")
        case "$CHOICE" in
            "Wipe Disk") wipe_disk ;;
            "Partition (Simple EFI + Root)") sgdisk -n 1:0:+512M -t 1:ef00 -c 1:EFI "$INSTALL_TARGET"; sgdisk -n 2:0:0 -t 2:8300 -c 2:Root "$INSTALL_TARGET"; gum style "✅ Partitioned."; sleep 1 ;;
            "Format Partitions") mkfs.fat -F32 "${INSTALL_TARGET}p1"; mkfs.ext4 -F "${INSTALL_TARGET}p2"; gum style "✅ Formatted."; sleep 1 ;;
            "Mount Partitions") mount "${INSTALL_TARGET}p2" "$MOUNT_POINT"; mkdir -p "$MOUNT_POINT/boot/efi"; mount "${INSTALL_TARGET}p1" "$MOUNT_POINT/boot/efi"; gum style "✅ Mounted."; sleep 1 ;;
            "Install Base System") pacstrap "$MOUNT_POINT" base linux linux-firmware; genfstab -U "$MOUNT_POINT" >> "$MOUNT_POINT/etc/fstab"; gum style "✅ Base system installed."; sleep 1 ;;
            "Setup Bootloader") arch-chroot "$MOUNT_POINT" grub-install --target=x86_64-efi --efi-directory=/boot/efi; arch-chroot "$MOUNT_POINT" grub-mkconfig -o /boot/grub/grub.cfg; gum style "✅ GRUB installed."; sleep 1 ;;
            "⬅️ Back") break ;;
        esac
    done
}

quick_backup_menu() {
    clear; CHOICE=$(gum choose "📦 Export Package List" "🗃️ Backup Dotfiles" "⬅️ Back")
    case "$CHOICE" in
        "📦 Export Package List") pacman -Qqe > "$WORKDIR/packages-$(date +%F).txt"; gum style --foreground=7 "✅ Package list saved."; sleep 2 ;;
        "🗃️ Backup Dotfiles") tar -czf "$WORKDIR/dotfiles-$(date +%F).tar.gz" -C "$USER_HOME_DIR" .; gum style --foreground=7 "✅ Dotfiles backed up."; sleep 2 ;;
    esac
}

clean_workspace() { rm -rf "$WORKDIR"; gum style --foreground=7 "✅ Workspace cleaned."; sleep 2; }

show_help() {
    clear; gum style --border double --padding 1 --margin 1 <<-'EOF'
    ❓ Help - Arch System Suite

    🚀 Replicator: The best way to replicate your setup on new hardware.
       1. `Create Snapshot`: Run on your main PC to create a `snapshot.tar.gz`.
       2. `Deploy Snapshot`: Run on a new PC from the Arch Live ISO to install
          and configure the system from the snapshot file.

    💿 Cloner: Creates a direct, 1:1 bootable copy of your system.
       - Best for moving to identical hardware or for forensic backups.

    🧰 Utilities:
       - `System Inspector`: Detects hardware and prepares a driver list.
       - `Flash ISO to USB`: A safe `dd` wrapper to burn an ISO to a drive.
       - `Manual Tools`: Step-by-step disk partitioning and installation.
       - `Quick Backups`: Export package lists or dotfiles.
EOF
    pause
}

# =================================================
# 🎬 SCRIPT ENTRY POINT (Argument Parser)
# =================================================
clear
# Run the dependency check at the very beginning of execution.
ensure_dependencies

case "${1:-}" in
    --replicator) replicator_menu ;;
    --cloner) cloner_menu ;;
    --utilities) utilities_menu ;;
    --help) show_help ;;
    *) gum style --foreground=196 "This script is intended to be run by the 'arch-suite' executable."; exit 1 ;;
esac
