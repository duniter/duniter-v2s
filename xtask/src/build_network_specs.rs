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

use anyhow::Result;
use std::process::Command;

pub fn build_network_specs(runtime: String) -> Result<()> {
    println!(
        "🚀 Construction des spécifications réseau pour le runtime: {}",
        runtime
    );

    // Vérifier que le fichier genesis.json existe
    let genesis_file = std::path::Path::new("output/genesis.json");
    if !genesis_file.exists() {
        return Err(anyhow::anyhow!(
            "Le fichier output/genesis.json n'existe pas. Exécutez d'abord 'cargo xtask g1-data' pour générer les données G1."
        ));
    }

    // Définir les variables d'environnement comme dans la CI
    unsafe {
        std::env::set_var(
            "DUNITER_GENESIS_DATA",
            genesis_file.to_string_lossy().to_string(),
        );
    }

    // Construire les features comme dans la CI
    let features = format!("--features {} --no-default-features", runtime);
    println!("🔧 Features: {}", features);

    // Créer le répertoire release s'il n'existe pas
    std::fs::create_dir_all("release")?;

    // Construire le binaire avec les features appropriées
    println!("🔨 Construction du binaire...");
    exec_should_success(
        Command::new("cargo")
            .args(["build", "--release"])
            .args(features.split_whitespace()),
    )?;

    // Générer le fichier de spécification
    let spec_file = format!("release/{}.json", runtime);
    println!("📄 Génération du fichier de spécification: {}", spec_file);

    let chain_arg = format!("{}_live", runtime);
    exec_should_success(
        Command::new("cargo")
            .args(["run", "--release"])
            .args(features.split_whitespace())
            .args(["build-spec", "--chain", &chain_arg])
            .env(
                "DUNITER_GENESIS_DATA",
                genesis_file.to_string_lossy().to_string(),
            )
            .stdout(std::fs::File::create(&spec_file)?),
    )?;

    // Copier le fichier de configuration YAML comme dans la CI
    let config_src = format!("resources/{}.yaml", runtime);
    let config_dst = format!("release/{}.yaml", runtime);

    if std::path::Path::new(&config_src).exists() {
        println!(
            "📋 Copie du fichier de configuration: {} -> {}",
            config_src, config_dst
        );
        std::fs::copy(&config_src, &config_dst)?;
    } else {
        println!("⚠️  Fichier de configuration non trouvé: {}", config_src);
    }

    println!("✅ Spécifications réseau générées avec succès!");
    println!("📁 Fichiers disponibles dans le répertoire 'release/':");
    println!("   - {}", spec_file);
    if std::path::Path::new(&config_dst).exists() {
        println!("   - {}", config_dst);
    }

    Ok(())
}

fn exec_should_success(command: &mut Command) -> Result<()> {
    if !command.status()?.success() {
        std::process::exit(1);
    } else {
        Ok(())
    }
}
