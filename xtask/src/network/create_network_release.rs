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

/// Crée une release réseau sur GitLab avec les assets nécessaires.
/// Une release réseau = produire et publier le genesis.json et le runtime wasm.
/// Ne livre pas le client, c'est une autre étape à réaliser ensuite.
/// Les livraisons de tous les futurs clients feront référence à cette release réseau
/// pour construire les spécifications du client (raw specs).
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
/// * `branch` - La branche GitLab à partir de laquelle créer la release (ex: network/gtest-1000).
///   Doit exister sur GitLab. Peut-être n'importe quelle branche, mais préférer créer
///   une branche dédiée par réseau comme `network/gtest-1000`.
pub async fn create_network_release(network: String, branch: String) -> Result<()> {
    println!("🚀 Création de la release réseau pour: {network}");

    // Déterminer le runtime basé sur le réseau
    let runtime = if network.starts_with("gdev-") {
        "gdev"
    } else if network.contains("gtest-") {
        "gtest"
    } else if network.contains("g1-") {
        "g1"
    } else {
        return Err(anyhow!(
            "Impossible de déterminer le runtime pour le réseau: {}. Préfixez le nom de release par gdev-, gtest- ou g1-. Ex. : gtest-1000",
            network
        ));
    };

    println!("📦 Runtime détecté: {runtime}");

    // Vérifier que les fichiers nécessaires existent
    let required_files = vec![
        format!("release/network/genesis.json"),
        format!("release/network/{}.yaml", runtime),
        format!("release/network/{}.json", runtime),
        format!(
            "release/network/{}_runtime.compact.compressed.wasm",
            runtime
        ),
        format!("release/network/{}_runtime.compact.wasm", runtime),
    ];

    for file in &required_files {
        if !Path::new(file).exists() {
            return Err(anyhow!(
                "Le fichier requis n'existe pas: {}. Assurez-vous d'avoir exécuté les étapes de build précédentes.",
                file
            ));
        }
        println!("✅ Fichier trouvé: {file}");
    }

    // Créer la release réseau via GitLab
    println!("🌐 Création de la release GitLab...");
    crate::gitlab::release_network(network.clone(), branch.clone()).await?;

    // Uploader les fichiers et créer les liens d'assets
    println!("📤 Upload des fichiers vers GitLab...");

    // ID du projet GitLab (nodes/rust/duniter-v2s)
    let project_id = "nodes%2Frust%2Fduniter-v2s".to_string();

    let asset_files = vec![
        (
            "g1-data.json".to_string(),
            "release/network/genesis.json".to_string(),
        ),
        (
            format!("{runtime}.yaml"),
            format!("release/network/{runtime}.yaml"),
        ),
        (
            format!("{runtime}_runtime.compact.compressed.wasm"),
            format!("release/network/{runtime}_runtime.compact.compressed.wasm"),
        ),
        (
            format!("{runtime}_runtime.compact.wasm"),
            format!("release/network/{runtime}_runtime.compact.wasm"),
        ),
        (
            format!("{runtime}.json"),
            format!("release/network/{runtime}.json"),
        ),
    ];

    for (asset_name, file_path) in &asset_files {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(anyhow!("Le fichier d'asset n'existe pas: {}", file_path));
        }

        println!("📤 Upload de {asset_name}...");
        let asset_url =
            crate::gitlab::upload_file(project_id.clone(), path, asset_name.clone()).await?;

        println!("📎 Création du lien d'asset: {asset_name} -> {asset_url}");
        crate::gitlab::create_asset_link(network.clone(), asset_name.clone(), asset_url).await?;
    }

    // Fichiers historiques pour Squid (indexeur) — compressés car trop volumineux pour GitLab
    let squid_files = vec!["block_hist.json", "cert_hist.json", "tx_hist.json"];
    for filename in &squid_files {
        let src = format!("release/network/{filename}");
        if !Path::new(&src).exists() {
            return Err(anyhow!("Le fichier Squid n'existe pas: {}", src));
        }
        let gz_path = format!("{src}.gz");
        let gz_name = format!("{filename}.gz");
        println!("🗜️  Compression de {filename}...");
        let status = Command::new("gzip").args(["-k", "-f", &src]).status()?;
        if !status.success() {
            return Err(anyhow!("Échec de la compression de {}", src));
        }
        println!("📤 Upload de {gz_name}...");
        let asset_url =
            crate::gitlab::upload_file(project_id.clone(), Path::new(&gz_path), gz_name.clone())
                .await?;
        println!("📎 Création du lien d'asset: {gz_name} -> {asset_url}");
        crate::gitlab::create_asset_link(network.clone(), gz_name, asset_url).await?;
    }

    println!("✅ Release réseau créée avec succès pour: {network}");
    println!("📋 Résumé:");
    println!("   - Réseau: {network}");
    println!("   - Runtime: {runtime}");
    println!("   - Branche: {branch}");
    println!("   - Assets: {} fichiers", asset_files.len());

    Ok(())
}
