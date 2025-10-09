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

/// Crée une release client sur GitLab avec les assets nécessaires.
/// Cette fonction reproduit l'étape de CI create_client_release qui :
/// 1. Crée la release GitLab avec le milestone
/// 2. Upload les fichiers client-specs.yaml et raw.json
/// 3. Crée les liens d'assets pour la release
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
/// * `branch` - La branche Git à utiliser
pub async fn create_client_release(network: String, branch: String) -> Result<()> {
    println!(
        "🚀 Création de la release client pour le réseau: {}",
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

    // Calculer les versions et noms comme dans la CI
    let client_version = get_client_version()?;
    let runtime_version = get_runtime_version(runtime)?;
    let client_milestone = format!("client-{}", client_version);
    let client_release_name = format!("{}-{}-{}", runtime, runtime_version, client_version);

    println!("📦 Version client: {}", client_version);
    println!("📦 Version runtime: {}", runtime_version);
    println!("🏷️ Milestone: {}", client_milestone);
    println!("📋 Nom de release: {}", client_release_name);

    // Vérifier que les fichiers nécessaires existent
    let required_files = vec![
        format!("release/client/{}-raw.json", runtime),
        format!("release/client/{}_client-specs.yaml", runtime),
    ];

    for file in &required_files {
        if !Path::new(file).exists() {
            return Err(anyhow!(
                "Le fichier requis n'existe pas: {}. Assurez-vous d'avoir exécuté build-raw-specs.",
                file
            ));
        }
        println!("✅ Fichier trouvé: {}", file);
    }

    // Étape 1: Créer la release client via GitLab
    println!("🌐 Création de la release client GitLab...");
    crate::gitlab::release_client(
        client_release_name.clone(),
        branch.clone(),
        client_milestone.clone(),
    )
    .await?;

    // Étape 2: Uploader les fichiers et créer les liens d'assets
    println!("📤 Upload des fichiers client vers GitLab...");

    // ID du projet GitLab (nodes/rust/duniter-v2s)
    let project_id = "nodes%2Frust%2Fduniter-v2s".to_string();

    let asset_files = vec![
        (
            format!("{}_client-specs.yaml", runtime),
            format!("release/client/{}_client-specs.yaml", runtime),
        ),
        (
            format!("{}-raw.json", runtime),
            format!("release/client/{}-raw.json", runtime),
        ),
    ];

    for (asset_name, file_path) in &asset_files {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(anyhow!("Le fichier d'asset n'existe pas: {}", file_path));
        }

        println!("📤 Upload de {}...", asset_name);
        let asset_url =
            crate::gitlab::upload_file(project_id.clone(), path, asset_name.clone()).await?;

        println!(
            "📎 Création du lien d'asset: {} -> {}",
            asset_name, asset_url
        );
        // Créer le lien d'asset via GitLab
        crate::gitlab::create_asset_link(
            client_release_name.clone(),
            asset_name.clone(),
            asset_url,
        )
        .await?;
    }

    println!("✅ Release client créée avec succès!");
    println!("📋 Résumé:");
    println!("   - Réseau: {}", network);
    println!("   - Runtime: {}", runtime);
    println!("   - Branche: {}", branch);
    println!("   - Release: {}", client_release_name);
    println!("   - Milestone: {}", client_milestone);
    println!("   - Assets: {} fichiers", asset_files.len());

    Ok(())
}

fn get_client_version() -> Result<String> {
    let output = Command::new("grep")
        .args(["version", "node/Cargo.toml"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Impossible de lire la version du client depuis node/Cargo.toml"
        ));
    }

    let version_line = String::from_utf8(output.stdout)?;
    let version = version_line
        .split("version = \"")
        .nth(1)
        .ok_or_else(|| anyhow!("Format de version invalide dans node/Cargo.toml"))?
        .split('"')
        .next()
        .ok_or_else(|| anyhow!("Format de version invalide dans node/Cargo.toml"))?;

    Ok(version.to_string())
}

fn get_runtime_version(runtime: &str) -> Result<String> {
    let runtime_file = format!("runtime/{}/src/lib.rs", runtime);
    let output = Command::new("grep")
        .args(["spec_version:", &runtime_file])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Impossible de lire la version du runtime depuis {}",
            runtime_file
        ));
    }

    let version_line = String::from_utf8(output.stdout)?;
    let version = version_line
        .split("spec_version: ")
        .nth(1)
        .ok_or_else(|| anyhow!("Format de version invalide dans {}", runtime_file))?
        .split(',')
        .next()
        .ok_or_else(|| anyhow!("Format de version invalide dans {}", runtime_file))?
        .trim();

    Ok(version.to_string())
}
