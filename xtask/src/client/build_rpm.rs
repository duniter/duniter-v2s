// Copyright 2021 Axiom-Team
//
// This file is part of Duniter-v2S.
//
// Duniter-v2S is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// Duniter-v2S is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.

use anyhow::{Result, anyhow};
use std::process::{Command, Stdio};

/// Builds an RPM package for a given network.
/// This function reproduces the CI build_rpm step which:
/// 1. Installs cargo-generate-rpm
/// 2. Builds the binary with appropriate features
/// 3. Generates the RPM package
/// # Arguments
/// * `network` - The network name (e.g., gtest-1000, g1-1000, gdev-1000)
pub fn build_rpm(network: String) -> Result<()> {
    println!("📦 Building RPM package for network: {}", network);

    // Check if rpm command is available
    if Command::new("which").arg("rpm").output().is_err()
        || !Command::new("which").arg("rpm").output()?.status.success()
    {
        return Err(anyhow!(
            "❌ The 'rpm' command is not available on this system.\n\
             RPM package generation requires the rpm command-line tool.\n\
             On macOS, you can install it with: brew install rpm\n\
             On Linux, it should be pre-installed or available via your package manager."
        ));
    }

    let runtime = if network.starts_with("g1") {
        "g1"
    } else if network.starts_with("gdev") {
        "gdev"
    } else if network.starts_with("gtest") {
        "gtest"
    } else {
        return Err(anyhow!(
            "Unknown network: {}. Supported networks are g1-*, gdev-*, and gtest-*.",
            network
        ));
    };

    println!("📦 Runtime: {}", runtime);

    // Step 1: Install cargo-generate-rpm
    println!("📥 Installing cargo-generate-rpm...");
    exec_should_success(Command::new("cargo").args([
        "install",
        "cargo-generate-rpm",
        "--version",
        "0.19.0",
    ]))?;

    // Step 2: Build the binary with appropriate features
    println!("🔨 Building binary...");
    let features = format!("--features {} --no-default-features", runtime);
    exec_should_success(
        Command::new("cargo")
            .args(["build", "-Zgit=shallow-deps", "--release"])
            .args(features.split_whitespace()),
    )?;

    // Step 3: Generate the RPM package
    // Note: We disable automatic dependency resolution (--auto-req disabled) because
    // on macOS, the dependency detection tools (like ldd equivalent) may produce
    // non-UTF-8 output, causing the "stream did not contain valid UTF-8" error.
    // For RPM packages, dependencies will need to be manually specified in Cargo.toml
    // if required, or handled by the package manager on the target system.
    println!("📦 Generating RPM package...");
    exec_should_success(Command::new("cargo").args([
        "generate-rpm",
        "-p",
        "node",
        "--auto-req",
        "disabled",
    ]))?;

    // Verify that the RPM file was generated
    let rpm_files = find_rpm_files()?;
    if rpm_files.is_empty() {
        return Err(anyhow!("No RPM file generated in target/generate-rpm/"));
    }

    println!("✅ RPM package generated successfully!");
    println!("📋 Summary:");
    println!("   - Network: {}", network);
    println!("   - Runtime: {}", runtime);
    println!("   - Generated RPM files:");
    for rpm_file in &rpm_files {
        println!("     - {}", rpm_file);
    }

    Ok(())
}

fn find_rpm_files() -> Result<Vec<String>> {
    use std::fs;

    let rpm_dir = "target/generate-rpm";
    if !std::path::Path::new(rpm_dir).exists() {
        return Ok(vec![]);
    }

    let mut rpm_files = Vec::new();
    let entries = fs::read_dir(rpm_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && let Some(extension) = path.extension()
            && extension == "rpm"
            && let Some(file_name) = path.file_name()
        {
            rpm_files.push(file_name.to_string_lossy().to_string());
        }
    }

    Ok(rpm_files)
}

fn exec_should_success(command: &mut Command) -> Result<()> {
    // Explicitly set stdout/stderr to inherit to avoid UTF-8 validation issues
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    let status = command
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn command: {}", e))?
        .wait()
        .map_err(|e| anyhow!("Failed to wait for command: {}", e))?;

    if !status.success() {
        return Err(anyhow!("Command failed with status: {}", status));
    }

    Ok(())
}
