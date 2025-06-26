// ===================================================================
// Core Actions Module
// ===================================================================
use crate::app::{AppAction};
use anyhow::{anyhow, Context, Result};
use std::io::{self, Write};
use std::process::Stdio;
use tokio::process::Command;

/// An enum to represent the different types of actions the app can perform.
/// This is more robust than using magic strings in the error channel.
#[derive(Clone)]
pub enum Action {
    Quit,
    SetView(crate::app::AppView),
    Execute(fn() -> AppAction),
}

// --- Replicator Actions ---
pub fn create_snapshot() -> AppAction {
    Box::pin(async {
        let user_output = Command::new("whoami").output().await?;
        let user_name = String::from_utf8(user_output.stdout)?.trim().to_string();
        let home_dir = format!("/home/{}", user_name);
        let work_dir = format!("{}/arch-suite-work", home_dir);
        let snapshot_dir = format!("{}/snapshot_tmp", work_dir);
        let snapshot_file = format!("{}/snapshot-{}.tar.gz", work_dir, chrono::Local::now().format("%Y%m%d"));
        std::fs::create_dir_all(&snapshot_dir)?;
        let command_script = format!(
            "pacman -Qqe > {0}/packages.x86_64.txt && \
             pacman -Qqm > {0}/packages.foreign.txt && \
             sudo tar -czf {0}/etc.tar.gz /etc && \
             sudo tar -czf {0}/home.tar.gz -C {1} --exclude='.cache' . && \
             sudo tar -czf {2} -C {0} . && \
             sudo chown {3}:{3} '{2}' && \
             sudo rm -rf {0}",
            snapshot_dir, home_dir, snapshot_file, user_name
        );
        let output = Command::new("sudo").arg("sh").arg("-c").arg(command_script).output().await?;
        if output.status.success() {
            Ok(format!("✅ Snapshot created successfully:\n{}", snapshot_file))
        } else {
            Err(anyhow!("Failed to create snapshot:\n{}", String::from_utf8_lossy(&output.stderr)))
        }
    })
}

pub fn deploy_snapshot() -> AppAction { Box::pin(async { Ok("Deploy Snapshot not yet implemented.".to_string()) }) }

// --- Cloner Actions ---
pub fn create_iso() -> AppAction { Box::pin(async { Ok("Create ISO not yet implemented.".to_string()) }) }

// --- Utilities Actions ---
pub fn inspect_system() -> AppAction { Box::pin(async { Ok("System Inspector not yet implemented.".to_string()) }) }
pub fn flash_iso() -> AppAction { Box::pin(async { Ok("Flash ISO not yet implemented.".to_string()) }) }

// --- Manual Installer Actions ---
pub fn manual_wipe_disk() -> AppAction { Box::pin(async { Ok("Wipe Disk not yet implemented.".to_string()) }) }
pub fn manual_partition_disk() -> AppAction { Box::pin(async { Ok("Partition Disk not yet implemented.".to_string()) }) }
pub fn manual_format_partitions() -> AppAction { Box::pin(async { Ok("Format Partitions not yet implemented.".to_string()) }) }
pub fn manual_mount_partitions() -> AppAction { Box::pin(async { Ok("Mount Partitions not yet implemented.".to_string()) }) }
pub fn manual_pacstrap() -> AppAction { Box::pin(async { Ok("Pacstrap not yet implemented.".to_string()) }) }
pub fn manual_chroot_grub() -> AppAction { Box::pin(async { Ok("Chroot & GRUB not yet implemented.".to_string()) }) }


// --- Dependency Management ---
pub async fn check_and_install_dependencies() -> Result<bool> {
    let deps = ["gum", "arch-install-scripts", "pacman-contrib", "gptfdisk", "dosfstools", "e2fsprogs", "archiso", "rsync", "pciutils"];
    let mut missing_deps = Vec::new();
    println!("Checking dependencies...");
    for dep in &deps {
        let status = Command::new("pacman").arg("-Q").arg(dep).stdout(Stdio::null()).stderr(Stdio::null()).status().await?;
        if !status.success() { missing_deps.push(*dep); }
    }
    if missing_deps.is_empty() {
        println!("✅ All dependencies are satisfied.");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Ok(true);
    }
    println!("\n⚠️ The following required packages are missing: {}", missing_deps.join(", "));
    print!("Would you like to install them now with sudo? (y/N) ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().eq_ignore_ascii_case("y") {
        println!("Attempting to install missing packages...");
        let mut args = vec!["-Syu", "--noconfirm", "--needed"];
        args.extend_from_slice(&missing_deps);
        let mut child = Command::new("sudo").args(&args).spawn().context("Failed to run sudo pacman. Do you have sudo privileges?")?;
        let status = child.wait().await?;
        if status.success() { println!("✅ Dependencies installed successfully."); Ok(true) } 
        else { println!("❌ Failed to install dependencies. Please try installing them manually."); Ok(false) }
    } else { Ok(false) }
}
