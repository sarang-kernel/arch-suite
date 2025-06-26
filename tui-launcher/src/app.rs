// ===================================================================
// Application State Module
// ===================================================================
// This module defines the core data structures that hold the entire
// state of the application. It also re-exports types from sub-modules
// to provide a clean public API for the rest of the application.

// Re-export types from sub-modules to make them accessible from here.
pub use crate::actions::{Action, AppAction};
pub use crate::components::stateful_list::StatefulList;

use crate::actions;
use tui_input::Input;

// --- Enums for State Management ---
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AppView { MainMenu, HelpManual, Replicator, Cloner, Utilities, ManualInstaller }
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Popup { None, Help, Action, Confirm, Input, Select }

// --- Core Application Structs ---
pub struct MenuItem<'a> {
    pub icon: &'a str,
    pub text: &'a str,
    pub help: &'a str,
    pub action: Action,
}

pub struct App<'a> {
    // Core State
    pub current_view: AppView,
    pub active_popup: Popup,
    pub should_quit: bool,
    
    // Menus
    pub main_menu: StatefulList<MenuItem<'a>>,
    pub replicator_menu: StatefulList<MenuItem<'a>>,
    pub cloner_menu: StatefulList<MenuItem<'a>>,
    pub utilities_menu: StatefulList<MenuItem<'a>>,
    pub manual_install_menu: StatefulList<MenuItem<'a>>,
    
    // Popup Data
    pub popup_title: String,
    pub popup_text: String,
    pub popup_list: StatefulList<String>,
    pub popup_input: Input,
    pub popup_action: Option<Action>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            current_view: AppView::MainMenu,
            active_popup: Popup::None,
            should_quit: false,
            main_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[R]", text: "Replicator (Recommended)", help: "Captures the 'recipe' of your system to perform a clean, fresh installation on new hardware.", action: Action::SetView(AppView::Replicator) },
                MenuItem { icon: "[C]", text: "Cloner (Advanced)", help: "Creates a direct, 1:1 bootable ISO image of your current system. Best for backups or identical hardware.", action: Action::SetView(AppView::Cloner) },
                MenuItem { icon: "[U]", text: "Utilities & Manual Tools", help: "Essential tools for system maintenance, including a hardware inspector, USB flasher, and manual installation steps.", action: Action::SetView(AppView::Utilities) },
                MenuItem { icon: "[H]", text: "Main Help", help: "Displays the main, scrollable help manual for the entire application.", action: Action.SetView(AppView::HelpManual) },
                MenuItem { icon: "[Q]", text: "Quit", help: "Exits the Arch System Suite application.", action: Action::Quit },
            ]),
            replicator_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[S]", text: "Create System Snapshot", help: "Gathers package lists, /etc configs, and dotfiles into a single snapshot file.", action: Action::Execute(actions::create_snapshot) },
                MenuItem { icon: "[D]", text: "Deploy from Snapshot", help: "Performs a fresh Arch install and applies a snapshot file to replicate a system.", action: Action::Execute(actions::deploy_snapshot) },
            ]),
            cloner_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[I]", text: "Create Bootable ISO", help: "Creates a bootable .iso file from the current system state using 'archiso'.", action: Action::Execute(actions::create_iso) },
            ]),
            utilities_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[H]", text: "System Inspector & Prep", help: "Detects CPU/GPU and prepares a list of recommended drivers for installation.", action: Action::Execute(actions::inspect_system) },
                MenuItem { icon: "[F]", text: "Flash ISO to USB", help: "A safe wrapper around 'dd' to burn any .iso file to a USB drive.", action: Action::Execute(actions::flash_iso) },
                MenuItem { icon: "[M]", text: "Manual Install Tools", help: "A step-by-step interface for advanced users to partition, format, and install.", action: Action::SetView(AppView::ManualInstaller) },
            ]),
            manual_install_menu: StatefulList::with_items(vec![
                MenuItem { icon: "[1]", text: "Wipe Disk", help: "Completely erases all data and partition tables from a selected disk.", action: Action::Execute(actions::manual_wipe_disk) },
                MenuItem { icon: "[2]", text: "Partition Disk", help: "Creates a simple EFI + Root partition layout on a selected disk.", action: Action::Execute(actions::manual_partition_disk) },
                MenuItem { icon: "[3]", text: "Format Partitions", help: "Formats the partitions created in the previous step (fat32 for EFI, ext4 for Root).", action: Action::Execute(actions::manual_format_partitions) },
                MenuItem { icon: "[4]", text: "Mount Partitions", help: "Mounts the root and EFI partitions to /mnt and /mnt/boot/efi.", action: Action::Execute(actions::manual_mount_partitions) },
                MenuItem { icon: "[5]", text: "Install Base System", help: "Runs 'pacstrap' to install the base Arch Linux system to /mnt.", action: Action::Execute(actions::manual_pacstrap) },
                MenuItem { icon: "[6]", text: "Setup Bootloader", help: "Runs 'arch-chroot' to install and configure the GRUB bootloader.", action: Action::Execute(actions::manual_chroot_grub) },
            ]),
            popup_title: String::new(),
            popup_text: String::new(),
            popup_list: StatefulList::with_items(vec![]),
            popup_input: Input::default(),
            popup_action: None,
        }
    }
}
