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

/// Crée une release runtime sur GitLab avec les assets nécessaires.
/// Cette fonction reproduit l'étape de CI create_runtime_release qui :
/// 1. Crée la release GitLab avec le milestone
/// 2. Upload le fichier WASM runtime
/// 3. Crée le lien d'asset pour la release
/// # Arguments
/// * `runtime` - Le runtime à publier (gdev, gtest, g1)
/// * `branch` - La branche Git à utiliser
pub async fn create_runtime_release(runtime: String, branch: String) -> Result<()> {
    println!("🚀 Création de la release runtime pour: {}", runtime);

    // Vérifier que le runtime est valide
    if !["gdev", "gtest", "g1"].contains(&runtime.as_str()) {
        return Err(anyhow!(
            "Runtime invalide: {}. Les runtimes supportés sont gdev, gtest, g1.",
            runtime
        ));
    }

    // Calculer les versions et noms comme dans la CI
    let runtime_version = get_runtime_version(&runtime)?;
    let runtime_milestone = format!("runtime-{}", runtime_version);

    println!("📦 Version runtime: {}", runtime_version);
    println!("🏷️  Milestone: {}", runtime_milestone);

    // Vérifier que le fichier WASM existe
    let wasm_file = format!("release/{}_runtime.compact.compressed.wasm", runtime);
    if !Path::new(&wasm_file).exists() {
        return Err(anyhow!(
            "Le fichier WASM n'existe pas: {}. Exécutez d'abord 'cargo xtask release runtime build {}' pour générer le runtime.",
            wasm_file,
            runtime
        ));
    }
    println!("✅ Fichier WASM trouvé: {}", wasm_file);

    // Vérifier la présence des fichiers d'historique (optionnels pour runtime upgrade)
    // Ces fichiers sont nécessaires uniquement pour le bootstrap d'un nouveau réseau
    let history_files = vec![
        "release/network/genesis.json",
        "release/network/block_hist.json",
        "release/network/cert_hist.json",
        "release/network/tx_hist.json",
    ];

    let mut available_history_files = Vec::new();
    let mut has_all_history_files = true;

    for file in &history_files {
        if Path::new(file).exists() {
            println!("✅ Fichier d'historique trouvé: {}", file);
            available_history_files.push(file.to_string());
        } else {
            has_all_history_files = false;
        }
    }

    if !has_all_history_files {
        println!("⚠️  Fichiers d'historique G1 non trouvés (optionnel pour runtime upgrade)");
        println!("   Pour un bootstrap réseau, exécutez: cargo xtask release network g1-data");
    }

    // Étape 1: Créer la release runtime via GitLab
    println!("🌐 Création de la release runtime GitLab...");
    crate::gitlab::release_runtime(
        runtime_milestone.clone(),
        runtime.clone(),
        branch.clone(),
        runtime_milestone.clone(),
    )
    .await?;

    // Étape 2: Uploader les fichiers (WASM + historiques si disponibles) et créer les liens d'assets
    println!("📤 Upload des fichiers vers GitLab...");

    // ID du projet GitLab (nodes/rust/duniter-v2s)
    let project_id = "nodes%2Frust%2Fduniter-v2s".to_string();

    // Liste des assets à uploader (nom dans la release, chemin du fichier)
    let mut asset_files = vec![(
        format!("{}_runtime.compact.compressed.wasm", runtime),
        wasm_file.clone(),
    )];

    // Ajouter les fichiers d'historique seulement s'ils sont disponibles
    if has_all_history_files {
        asset_files.extend(vec![
            (
                "genesis.json".to_string(),
                "release/network/genesis.json".to_string(),
            ),
            (
                "block_hist.json".to_string(),
                "release/network/block_hist.json".to_string(),
            ),
            (
                "cert_hist.json".to_string(),
                "release/network/cert_hist.json".to_string(),
            ),
            (
                "tx_hist.json".to_string(),
                "release/network/tx_hist.json".to_string(),
            ),
        ]);
    }

    for (asset_name, file_path) in &asset_files {
        let path = Path::new(file_path);

        println!("📤 Upload de {}...", asset_name);
        let asset_url =
            crate::gitlab::upload_file(project_id.clone(), path, asset_name.clone()).await?;

        println!(
            "📎 Création du lien d'asset: {} -> {}",
            asset_name, asset_url
        );
        // Créer le lien d'asset via GitLab
        crate::gitlab::create_asset_link(runtime_milestone.clone(), asset_name.clone(), asset_url)
            .await?;
    }

    println!("✅ Release runtime créée avec succès!");
    println!("📋 Résumé:");
    println!("   - Runtime: {}", runtime);
    println!("   - Version: {}", runtime_version);
    println!("   - Branche: {}", branch);
    println!("   - Release: {}", runtime_milestone);
    println!("   - Assets uploadés:");
    for (asset_name, _) in &asset_files {
        println!("     • {}", asset_name);
    }

    Ok(())
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

    println!("📦 Version runtime détectée: {}", version);
    Ok(version.to_string())
}
