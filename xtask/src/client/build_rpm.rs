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
use std::process::Command;

/// Construit un package RPM pour un réseau donné.
/// Cette fonction reproduit l'étape de CI build_rpm qui :
/// 1. Installe cargo-generate-rpm
/// 2. Construit le binaire avec les features appropriées
/// 3. Génère le package RPM
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
pub fn build_rpm(network: String) -> Result<()> {
    println!("📦 Construction du package RPM pour le réseau: {}", network);

    let runtime = if network.starts_with("g1") {
        "g1"
    } else if network.starts_with("gdev") {
        "gdev"
    } else if network.starts_with("gtest") {
        "gtest"
    } else {
        return Err(anyhow!(
            "Réseau inconnu: {}. Les réseaux supportés sont g1-*, gdev-* et gtest-*.",
            network
        ));
    };

    println!("📦 Runtime: {}", runtime);

    // Étape 1: Installer cargo-generate-rpm
    println!("📥 Installation de cargo-generate-rpm...");
    exec_should_success(Command::new("cargo").args([
        "install",
        "cargo-generate-rpm",
        "--version",
        "0.16.1",
    ]))?;

    // Étape 2: Construire le binaire avec les features appropriées
    println!("🔨 Construction du binaire...");
    let features = format!("--features {} --no-default-features", runtime);
    exec_should_success(
        Command::new("cargo")
            .args(["build", "-Zgit=shallow-deps", "--release"])
            .args(features.split_whitespace()),
    )?;

    // Étape 3: Générer le package RPM
    println!("📦 Génération du package RPM...");
    exec_should_success(Command::new("cargo").args(["generate-rpm", "-p", "node"]))?;

    // Vérifier que le fichier RPM a été généré
    let rpm_files = find_rpm_files()?;
    if rpm_files.is_empty() {
        return Err(anyhow!(
            "Aucun fichier RPM généré dans target/generate-rpm/"
        ));
    }

    println!("✅ Package RPM généré avec succès!");
    println!("📋 Résumé:");
    println!("   - Réseau: {}", network);
    println!("   - Runtime: {}", runtime);
    println!("   - Fichiers RPM générés:");
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
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "rpm" {
                    if let Some(file_name) = path.file_name() {
                        rpm_files.push(file_name.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Ok(rpm_files)
}

fn exec_should_success(command: &mut Command) -> Result<()> {
    if !command.status()?.success() {
        std::process::exit(1);
    } else {
        Ok(())
    }
}
