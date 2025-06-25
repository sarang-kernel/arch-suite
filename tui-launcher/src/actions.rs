// ===================================================================
// Core Actions Module
// ===================================================================
// This module contains the functions that perform the actual system
// work, like creating snapshots or installing packages. They are designed
// to be called from the event loop and return a result to be displayed.

use anyhow::{Context, Result};
use std::io::{self, Write};
use std::process::Stdio;
use tokio::process::Command;

/// A function pointer type for actions that can be executed from a menu.
pub type AppAction = std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>>>>;

/// Creates a system snapshot.
/// This function constructs a shell command and executes it, returning the result.
pub fn action_create_snapshot() -> AppAction {
    Box::pin(async {
        let home_dir = std::env::var("HOME").context("Failed to get HOME directory")?;
        let work_dir = format!("{}/arch-suite-work", home_dir);
        let snapshot_dir = format!("{}/snapshot_tmp", work_dir);
        let snapshot_file = format!("{}/snapshot-{}.tar.gz", work_dir, chrono::Local::now().format("%Y%m%d"));

        std::fs::create_dir_all(&snapshot_dir)?;

        // Using a single shell command is often simpler for a sequence of system tasks.
        // Each command is chained with `&&` to ensure it only runs if the previous one succeeded.
        let command_script = format!(
            "echo 'Gathering package lists...'; \
             pacman -Qqe > {0}/packages.x86_64.txt && \
             pacman -Qqm > {0}/packages.foreign.txt && \
             echo 'Archiving /etc...'; \
             sudo tar -czf {0}/etc.tar.gz /etc && \
             echo 'Archiving dotfiles...'; \
             tar -czf {0}/home.tar.gz -C {1} --exclude='.cache' . && \
             echo 'Creating final snapshot...'; \
             tar -czf {2} -C {0} . && \
             sudo rm -rf {0}",
            snapshot_dir, home_dir, snapshot_file
        );

        // Execute the command using Tokio for async support.
        let output = Command::new("sh").arg("-c").arg(command_script).output().await?;

        if output.status.success() {
            Ok(format!("✅ Snapshot created successfully:\n{}", snapshot_file))
        } else {
            // If the command fails, return an error with the stderr content.
            Err(anyhow::anyhow!("Failed to create snapshot:\n{}", String::from_utf8_lossy(&output.stderr)))
        }
    })
}

/// Checks for required external command-line tools and prompts to install them if missing.
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
