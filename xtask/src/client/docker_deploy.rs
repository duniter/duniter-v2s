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

/// Déploie une image Docker multi-architecture pour un réseau donné.
/// Cette fonction reproduit l'étape de CI docker_deploy qui :
/// 1. Se connecte à Docker Hub avec podman
/// 2. Construit une image multi-architecture (amd64, arm64)
/// 3. Pousse l'image vers Docker Hub avec les tags appropriés
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
pub fn docker_deploy(network: String) -> Result<()> {
    println!("🐳 Déploiement Docker pour le réseau: {}", network);

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

    // Vérifier que podman est disponible
    if Command::new("podman").arg("--version").status().is_err() {
        return Err(anyhow!(
            "podman n'est pas installé. Veuillez installer podman pour continuer.\n\
            Sur Ubuntu/Debian: sudo apt-get install podman\n\
            Sur macOS: brew install podman"
        ));
    }

    // Vérifier que les variables d'environnement nécessaires sont présentes
    let docker_password = std::env::var("DUNITERTEAM_PASSWD")
        .map_err(|_| anyhow!("Variable d'environnement DUNITERTEAM_PASSWD manquante"))?;

    // Calculer les variables comme dans la CI
    let client_version = get_client_version()?;
    let runtime_version = get_runtime_version(runtime)?;
    let docker_tag = format!("{}-{}", runtime_version, client_version);
    let image_name = format!("duniter/duniter-v2s-{}", network);
    let manifest = format!("localhost/manifest-{}:{}", image_name, docker_tag);

    println!("🏷️  Tag Docker: {}", docker_tag);
    println!("📦 Nom de l'image: {}", image_name);
    println!("📋 Manifest: {}", manifest);

    // Étape 1: Se connecter à Docker Hub
    println!("🔐 Connexion à Docker Hub...");
    exec_should_success(Command::new("podman").args([
        "login",
        "-u",
        "duniterteam",
        "-p",
        &docker_password,
        "docker.io",
    ]))?;

    // Étape 2: Nettoyer le manifest existant s'il existe
    println!("🧹 Nettoyage du manifest existant...");
    let _ = Command::new("podman")
        .args(["manifest", "rm", &manifest])
        .status();

    // Étape 3: Construire l'image multi-architecture
    println!("🔨 Construction de l'image multi-architecture...");
    exec_should_success(Command::new("podman").args([
        "build",
        "--layers",
        "--platform",
        "linux/amd64,linux/arm64",
        "--manifest",
        &manifest,
        "-f",
        "docker/Dockerfile",
        "--build-arg",
        &format!("chain={}", runtime),
        ".",
    ]))?;

    // Étape 4: Pousser l'image avec le tag spécifique
    println!("📤 Poussée de l'image avec le tag spécifique...");
    exec_should_success(Command::new("podman").args([
        "manifest",
        "push",
        "--all",
        &manifest,
        &format!("docker://docker.io/{}:{}", image_name, docker_tag),
    ]))?;

    // Étape 5: Pousser l'image avec le tag latest
    println!("📤 Poussée de l'image avec le tag latest...");
    exec_should_success(Command::new("podman").args([
        "manifest",
        "push",
        "--all",
        &manifest,
        &format!("docker://docker.io/{}:latest", image_name),
    ]))?;

    // Étape 6: Nettoyer le manifest local
    println!("🧹 Nettoyage du manifest local...");
    let _ = Command::new("podman")
        .args(["manifest", "rm", &manifest])
        .status();

    println!("✅ Déploiement Docker terminé avec succès!");
    println!("📋 Résumé:");
    println!("   - Réseau: {}", network);
    println!("   - Runtime: {}", runtime);
    println!("   - Image: {}:{}", image_name, docker_tag);
    println!("   - Image latest: {}:latest", image_name);

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

    println!("📦 Version client détectée: {}", version);
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

    println!("📦 Version runtime détectée: {}", version);
    Ok(version.to_string())
}

fn exec_should_success(command: &mut Command) -> Result<()> {
    if !command.status()?.success() {
        std::process::exit(1);
    } else {
        Ok(())
    }
}
