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
use std::{path::Path, process::Command};

/// Construit un package DEB pour un réseau donné.
/// Cette fonction reproduit l'étape de CI build_deb qui :
/// 1. Installe cargo-deb
/// 2. Construit le binaire avec les features appropriées
/// 3. Génère le package DEB
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
pub fn build_deb(network: String) -> Result<()> {
    println!("📦 Construction du package DEB pour le réseau: {network}");

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

    println!("📦 Runtime: {runtime}");

    // Étape 0: S'assurer que le fichier raw spec existe (téléchargement depuis release si besoin)
    super::ensure_raw_spec::ensure_raw_spec(&network)?;

    // Étape 1: Installer cargo-deb
    println!("📥 Installation de cargo-deb...");
    exec_should_success(Command::new("cargo").args(["install", "cargo-deb"]))?;

    // Étape 2: Construire le binaire avec les features appropriées
    println!("🔨 Construction du binaire...");
    let features = format!("--features {runtime},embed,distance-oracle --no-default-features");
    let mut build_cmd = Command::new("cargo");
    apply_vendor_config_if_present(&mut build_cmd)
        .args(["build", "--release", "-p", "duniter"])
        .args(features.split_whitespace());
    exec_should_success(&mut build_cmd)?;

    // Étape 3: Générer le package DEB
    println!("📦 Génération du package DEB...");
    exec_should_success(Command::new("cargo").args(["deb", "--no-build", "-p", "duniter"]))?;

    // Vérifier que le fichier DEB a été généré
    let deb_files = find_deb_files()?;
    if deb_files.is_empty() {
        return Err(anyhow!("Aucun fichier DEB généré dans target/debian/"));
    }

    println!("✅ Package DEB généré avec succès!");
    println!("📋 Résumé:");
    println!("   - Réseau: {network}");
    println!("   - Runtime: {runtime}");
    println!("   - Fichiers DEB générés:");
    for deb_file in &deb_files {
        println!("     - {deb_file}");
    }

    Ok(())
}

fn find_deb_files() -> Result<Vec<String>> {
    use std::fs;

    let deb_dir = "target/debian";
    if !std::path::Path::new(deb_dir).exists() {
        return Ok(vec![]);
    }

    let mut deb_files = Vec::new();
    let entries = fs::read_dir(deb_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && let Some(extension) = path.extension()
            && extension == "deb"
            && let Some(file_name) = path.file_name()
        {
            deb_files.push(file_name.to_string_lossy().to_string());
        }
    }

    Ok(deb_files)
}

fn exec_should_success(command: &mut Command) -> Result<()> {
    if !command.status()?.success() {
        std::process::exit(1);
    } else {
        Ok(())
    }
}

fn apply_vendor_config_if_present(command: &mut Command) -> &mut Command {
    if Path::new("vendor-config.toml").exists() {
        command.args(["--config", "vendor-config.toml", "--frozen", "--offline"]);
    }
    command
}
