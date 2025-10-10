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

/// Construit les spécifications raw pour un réseau donné.
/// Cette fonction reproduit l'étape de CI build_raw_specs qui :
/// 1. Produit les spécifications réseau via print-spec
/// 2. Fusionne les spécifications client (YAML -> JSON)
/// 3. Génère le fichier raw spec final
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
/// * `runtime` - Le runtime à utiliser (gdev, gtest, g1)
pub fn build_raw_specs(network: String) -> Result<()> {
    println!(
        "🚀 Construction des spécifications raw pour le réseau: {}",
        network
    );

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

    // Créer le répertoire release s'il n'existe pas
    std::fs::create_dir_all("release/client/")?;

    // Vérifier que les fichiers nécessaires existent
    let required_files = vec![format!("node/specs/{}_client-specs.yaml", runtime)];

    for file in &required_files {
        if !Path::new(file).exists() {
            return Err(anyhow!(
                "Le fichier requis n'existe pas: {}. Assurez-vous d'avoir les spécifications client.",
                file
            ));
        } else {
            // Copier le fichier dans la release
            println!("✅ Fichier trouvé: {}", file);
            std::fs::copy(
                file,
                format!(
                    "release/client/{}",
                    Path::new(file).file_name().unwrap().to_string_lossy()
                ),
            )?;
            println!("📋 Fichier copié dans release/client/: {}", file);
        }
    }

    // Étape 1: Imprimer les spécifications réseau
    println!("📄 Téléchargement des spécifications réseau...");
    let printed_spec_file = format!("release/client/{}-printed.json", runtime);
    exec_should_success(
        Command::new("cargo")
            .args(["xtask", "print-spec", &network])
            .stdout(std::fs::File::create(&printed_spec_file)?),
    )?;

    // Étape 2: Vérifier et installer les outils nécessaires
    println!("🔧 Vérification des outils nécessaires...");

    // Vérifier si yq est disponible
    if Command::new("yq").arg("--version").status().is_err() {
        println!("📥 yq non trouvé, téléchargement...");
        exec_should_success(Command::new("wget").args([
            "https://github.com/mikefarah/yq/releases/download/v4.44.6/yq_linux_amd64",
            "-O",
            "release/client/yq",
        ]))?;
        exec_should_success(Command::new("chmod").args(["+x", "release/client/yq"]))?;
    }

    // Vérifier si jq est disponible
    if Command::new("jq").arg("--version").status().is_err() {
        return Err(anyhow!(
            "jq n'est pas installé. Veuillez installer jq pour continuer.\n\
            Sur Ubuntu/Debian: sudo apt-get install jq\n\
            Sur macOS: brew install jq"
        ));
    }

    // Étape 3: Convertir YAML -> JSON pour les spécifications client
    println!("🔄 Conversion YAML -> JSON des spécifications client...");
    let client_specs_json = format!("release/client/{}_client-specs.json", runtime);

    // Utiliser yq (local ou système)
    let yq_cmd = if Path::new("release/client/yq").exists() {
        "./release/client/yq"
    } else {
        "yq"
    };
    exec_should_success(
        Command::new(yq_cmd)
            .args(["--output-format", "json"])
            .stdin(std::fs::File::open(format!(
                "node/specs/{}_client-specs.yaml",
                runtime
            ))?)
            .stdout(std::fs::File::create(&client_specs_json)?),
    )?;

    // Étape 4: Fusionner les spécifications
    println!("🔗 Fusion des spécifications...");
    let final_spec_file = format!("{}.json", runtime);
    exec_should_success(
        Command::new("jq")
            .args(["-s", ".[0] * .[1]", &client_specs_json, &printed_spec_file])
            .stdout(std::fs::File::create(&final_spec_file)?),
    )?;

    // Étape 5: Générer le fichier raw spec
    println!("🔨 Génération du fichier raw spec...");
    let features = format!("--features {} --no-default-features", runtime);
    let raw_spec_file = format!("release/client/{}-raw.json", runtime);

    exec_should_success(
        Command::new("cargo")
            .args(["run", "-Zgit=shallow-deps"])
            .args(features.split_whitespace())
            .args(["--", "build-spec", "--chain", &final_spec_file, "--raw"])
            .stdout(std::fs::File::create(&raw_spec_file)?),
    )?;

    println!("✅ Spécifications raw générées avec succès!");
    println!("📁 Fichier généré: {}", raw_spec_file);
    println!("📋 Résumé:");
    println!("   - Réseau: {}", network);
    println!("   - Runtime: {}", runtime);
    println!("   - Fichier raw spec: {}", raw_spec_file);

    // Copier le fichier dans specs/
    std::fs::create_dir_all("node/specs/")?;
    let dest_path = format!("node/specs/{}-raw.json", runtime);
    std::fs::copy(&raw_spec_file, &dest_path)?;
    println!("📋 Fichier copié dans node/specs/: {}", dest_path);

    Ok(())
}

fn exec_should_success(command: &mut Command) -> Result<()> {
    if !command.status()?.success() {
        std::process::exit(1);
    } else {
        Ok(())
    }
}
