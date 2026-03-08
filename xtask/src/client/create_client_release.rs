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
use std::path::Path;

/// Crée une release client sur GitLab avec les assets nécessaires.
/// Cette fonction reproduit l'étape de CI create_client_release qui :
/// 1. Crée la release GitLab avec le milestone
/// 2. Upload les fichiers client-specs.yaml et raw.json
/// 3. Optionnellement upload les packages .deb et .rpm locaux
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
/// * `branch` - La branche Git à utiliser
/// * `upload_packages` - Si true, upload aussi les packages .deb/.rpm trouvés localement
pub async fn create_client_release(
    network: String,
    branch: String,
    upload_packages: bool,
) -> Result<()> {
    println!("🚀 Création de la release client pour le réseau: {network}");

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

    // Calculer les versions et noms comme dans la CI
    let client_version = super::package_validation::get_client_version()?;
    let runtime_version = extract_runtime_version_from_network(&network)?;
    let client_milestone = format!("client-{client_version}");
    let client_release_name = format!("{runtime}-{runtime_version}-{client_version}");

    println!("📦 Version client: {client_version}");
    println!("📦 Version runtime: {runtime_version}");
    println!("🏷️ Milestone: {client_milestone}");
    println!("📋 Nom de release: {client_release_name}");

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
        println!("✅ Fichier trouvé: {file}");
    }

    // Rechercher les fichiers .deb et .rpm dans target (seulement si demandé)
    let package_files = if upload_packages {
        let packages = super::package_validation::find_local_package_files(&client_version)?;
        if !packages.is_empty() {
            println!("📦 Packages trouvés (seront uploadés):");
            for (asset_name, file_path) in &packages {
                println!("   - {asset_name} ({file_path})");
            }
        } else {
            println!(
                "⚠️  Option --upload-packages activée mais aucun package .deb ou .rpm trouvé dans target/"
            );
        }
        packages
    } else {
        println!(
            "ℹ️  Les packages locaux ne seront pas uploadés (utilisez --upload-packages pour les inclure)"
        );
        Vec::new()
    };

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

    let mut asset_files = vec![
        (
            format!("{runtime}_client-specs.yaml"),
            format!("release/client/{runtime}_client-specs.yaml"),
        ),
        (
            format!("{runtime}-raw.json"),
            format!("release/client/{runtime}-raw.json"),
        ),
    ];

    // Ajouter les packages .deb et .rpm
    asset_files.extend(package_files);

    for (asset_name, file_path) in &asset_files {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(anyhow!("Le fichier d'asset n'existe pas: {}", file_path));
        }

        println!("📤 Upload de {asset_name}...");
        let asset_url =
            crate::gitlab::upload_file(project_id.clone(), path, asset_name.clone()).await?;

        println!("📎 Création du lien d'asset: {asset_name} -> {asset_url}");
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
    println!("   - Réseau: {network}");
    println!("   - Runtime: {runtime}");
    println!("   - Branche: {branch}");
    println!("   - Release: {client_release_name}");
    println!("   - Milestone: {client_milestone}");
    println!("   - Assets: {} fichiers", asset_files.len());

    Ok(())
}

/// Extract runtime version from network name (e.g., "gtest-1100" -> "1100")
fn extract_runtime_version_from_network(network: &str) -> Result<String> {
    // The network name format is expected to be: {runtime}-{version}
    // e.g., gtest-1100, g1-1000, gdev-1000
    let parts: Vec<&str> = network.split('-').collect();

    if parts.len() < 2 {
        return Err(anyhow!(
            "Le nom du réseau '{}' doit être au format {{runtime}}-{{version}} (ex: gtest-1100)",
            network
        ));
    }

    let version = parts[1];

    // Validate that it's a number
    if version.parse::<u32>().is_err() {
        return Err(anyhow!(
            "La version extraite '{}' du réseau '{}' n'est pas un nombre valide",
            version,
            network
        ));
    }

    Ok(version.to_string())
}
