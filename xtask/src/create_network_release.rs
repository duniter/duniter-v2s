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
use std::path::Path;

/// Crée une release réseau sur GitLab avec les assets nécessaires.
/// Une release réseau = produire et publier le genesis.json et le runtime wasm.
/// Ne livre pas le client, c'est une autre étape à réaliser ensuite.
/// Les livraisons de tous les futurs clients feront référence à cette release réseau
/// pour construire les spécifications du client (raw specs).
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
/// * `branch` - La branche GitLab à partir de laquelle créer la release (ex: network/gtest-1000).
///              Doit exister sur GitLab. Peut-être n'importe quelle branche, mais préférer créer
///              une branche dédiée par réseau comme `network/gtest-1000`.
pub async fn create_network_release(network: String, branch: String) -> Result<()> {
    println!("🚀 Création de la release réseau pour: {}", network);

    // Déterminer le runtime basé sur le réseau
    let runtime = if network.starts_with("gdev-") {
        "gdev"
    } else if network.contains("gtest-") {
        "gtest"
    } else if network.contains("g1-") {
        "g1"
    } else {
        return Err(anyhow::anyhow!(
            "Impossible de déterminer le runtime pour le réseau: {}. Préfixez le nom de release par gdev-, gtest- ou g1-. Ex. : gtest-1000",
            network
        ));
    };

    println!("📦 Runtime détecté: {}", runtime);

    // Vérifier que les fichiers nécessaires existent
    let required_files = vec![
        format!("release/genesis.json"),
        format!("release/{}.yaml", runtime),
        format!("release/{}.json", runtime),
        format!("release/{}_runtime.compact.compressed.wasm", runtime),
        format!("release/{}_runtime.compact.wasm", runtime),
    ];

    for file in &required_files {
        if !Path::new(file).exists() {
            return Err(anyhow::anyhow!(
                "Le fichier requis n'existe pas: {}. Assurez-vous d'avoir exécuté les étapes de build précédentes.",
                file
            ));
        }
        println!("✅ Fichier trouvé: {}", file);
    }

    // Créer la release réseau via GitLab
    println!("🌐 Création de la release GitLab...");
    crate::gitlab::release_network(network.clone(), branch.clone()).await?;

    // Créer les liens d'assets comme dans la CI
    println!("🔗 Création des liens d'assets...");

    // Simuler les URLs d'assets comme dans la CI
    // Dans un environnement réel, ces URLs seraient générées par GitLab CI
    let base_url =
        "https://nodes.pages.duniter.org/-/rust/duniter-v2s/-/jobs/ARTIFACT_JOB_ID/artifacts";

    let asset_links = vec![
        (
            "g1-data.json".to_string(),
            format!("{}/release/genesis.json", base_url),
        ),
        (
            format!("{}.yaml", runtime),
            format!("{}/release/{}.yaml", base_url, runtime),
        ),
        (
            format!("{}_runtime.compact.compressed.wasm", runtime),
            format!(
                "{}/release/{}_runtime.compact.compressed.wasm",
                base_url, runtime
            ),
        ),
        (
            format!("{}_runtime.compact.wasm", runtime),
            format!("{}/release/{}_runtime.compact.wasm", base_url, runtime),
        ),
        (
            format!("{}.json", runtime),
            format!("{}/release/{}.json", base_url, runtime),
        ),
    ];

    for (asset_name, asset_url) in &asset_links {
        println!(
            "📎 Création du lien d'asset: {} -> {}",
            asset_name, asset_url
        );
        // Créer le lien d'asset via GitLab
        crate::gitlab::create_asset_link(network.clone(), asset_name.clone(), asset_url.clone())
            .await?;
    }

    println!("✅ Release réseau créée avec succès pour: {}", network);
    println!("📋 Résumé:");
    println!("   - Réseau: {}", network);
    println!("   - Runtime: {}", runtime);
    println!("   - Branche: {}", branch);
    println!("   - Assets: {} fichiers", asset_links.len());

    Ok(())
}
