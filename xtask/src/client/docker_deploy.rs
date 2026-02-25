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

/// Déploie une image Docker pour un réseau donné.
/// Cette fonction reproduit l'étape de CI docker_deploy qui :
/// 1. Se connecte à Docker Hub avec podman/docker
/// 2. Construit une image pour l'architecture spécifiée (ou multi-arch si None)
/// 3. Pousse l'image vers Docker Hub avec les tags appropriés
/// # Arguments
/// * `network` - Le nom du réseau (ex: gtest-1000, g1-1000, gdev-1000)
/// * `arch` - L'architecture cible (amd64, arm64) ou None pour multi-arch
pub fn docker_deploy(network: String, arch: Option<String>) -> Result<()> {
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

    // Étape 0: S'assurer que le fichier raw spec existe dans le contexte Docker
    // (sera copié dans le container via COPY . . dans le Dockerfile)
    super::ensure_raw_spec::ensure_raw_spec(&network)?;

    // Detect which container tool is available (podman or docker)
    let container_tool = if Command::new("podman").arg("--version").status().is_ok() {
        "podman"
    } else if Command::new("docker").arg("--version").status().is_ok() {
        "docker"
    } else {
        return Err(anyhow!(
            "Neither podman nor docker is installed. Please install one of them:\n\
            - Podman: sudo apt-get install podman (Ubuntu/Debian) or brew install podman (macOS)\n\
            - Docker: https://docs.docker.com/get-docker/"
        ));
    };

    println!("🔧 Using container tool: {}", container_tool);

    // Vérifier que les variables d'environnement nécessaires sont présentes
    let docker_password = std::env::var("DUNITERTEAM_PASSWD")
        .map_err(|_| anyhow!("Variable d'environnement DUNITERTEAM_PASSWD manquante"))?;

    // Calculer les variables comme dans la CI
    let client_version = get_client_version()?;
    let runtime_version = get_runtime_version(runtime)?;

    // Add architecture suffix to tag if building for specific arch
    let docker_tag = if let Some(ref arch) = arch {
        format!("{}-{}-{}", runtime_version, client_version, arch)
    } else {
        format!("{}-{}", runtime_version, client_version)
    };

    let image_name = format!("duniter/duniter-v2s-{}", network);
    let manifest = format!("localhost/manifest-{}:{}", image_name, docker_tag);

    println!("🏷️  Tag Docker: {}", docker_tag);
    println!("📦 Nom de l'image: {}", image_name);
    println!("📋 Manifest: {}", manifest);
    if let Some(ref arch) = arch {
        println!("🏗️  Architecture: {}", arch);
    } else {
        println!("🏗️  Architecture: multi-arch (amd64, arm64)");
    }

    // Étape 1: Se connecter à Docker Hub
    println!("🔐 Connexion à Docker Hub...");
    exec_should_success(Command::new(container_tool).args([
        "login",
        "-u",
        "duniterteam",
        "-p",
        &docker_password,
        "docker.io",
    ]))?;

    // Étape 2: Nettoyer le manifest existant s'il existe
    println!("🧹 Nettoyage du manifest existant...");
    let _ = Command::new(container_tool)
        .args(["manifest", "rm", &manifest])
        .status();

    // Étape 3: Construire l'image (single-arch ou multi-arch)
    if container_tool == "docker" {
        // Docker buildx approach
        if let Some(ref arch) = arch {
            println!("🔨 Construction de l'image pour architecture {}...", arch);
            let image_tag = format!("{}:{}", image_name, docker_tag);

            // Use classic docker build (not buildx) for single-arch to avoid manifest creation
            // Build the image
            exec_should_success(Command::new("docker").args([
                "build",
                "--platform",
                &format!("linux/{}", arch),
                "--tag",
                &image_tag,
                "-f",
                "docker/Dockerfile",
                "--build-arg",
                &format!("chain={}", runtime),
                ".",
            ]))?;

            // Push the image
            println!("📤 Pushing image {}...", image_tag);
            exec_should_success(Command::new("docker").args(["push", &image_tag]))?;
        } else {
            println!("🔨 Construction de l'image multi-architecture...");
            exec_should_success(Command::new("docker").args([
                "buildx",
                "build",
                "--platform",
                "linux/amd64,linux/arm64",
                "--tag",
                &format!("{}:{}", image_name, docker_tag),
                "--push",
                "-f",
                "docker/Dockerfile",
                "--build-arg",
                &format!("chain={}", runtime),
                ".",
            ]))?;
        }
    } else {
        // Podman approach with manifest
        if let Some(ref arch) = arch {
            println!("🔨 Construction de l'image pour architecture {}...", arch);
            exec_should_success(Command::new(container_tool).args([
                "build",
                "--layers",
                "--platform",
                &format!("linux/{}", arch),
                "--manifest",
                &manifest,
                "-f",
                "docker/Dockerfile",
                "--build-arg",
                &format!("chain={}", runtime),
                ".",
            ]))?;
        } else {
            println!("🔨 Construction de l'image multi-architecture...");
            exec_should_success(Command::new(container_tool).args([
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
        }
    }

    // Étape 4: Pousser l'image (seulement pour Podman, Docker a déjà push avec --push)
    if container_tool == "podman" {
        // Podman: utiliser manifest push
        println!("📤 Poussée de l'image avec le tag spécifique...");
        exec_should_success(Command::new(container_tool).args([
            "manifest",
            "push",
            "--all",
            &manifest,
            &format!("docker://docker.io/{}:{}", image_name, docker_tag),
        ]))?;

        // Étape 5: Pousser l'image avec le tag latest (only for multi-arch builds)
        if arch.is_none() {
            println!("📤 Poussée de l'image avec le tag latest...");
            exec_should_success(Command::new(container_tool).args([
                "manifest",
                "push",
                "--all",
                &manifest,
                &format!("docker://docker.io/{}:latest", image_name),
            ]))?;
        }

        // Étape 6: Nettoyer le manifest local
        println!("🧹 Nettoyage du manifest local...");
        let _ = Command::new(container_tool)
            .args(["manifest", "rm", &manifest])
            .status();
    }

    println!("✅ Déploiement Docker terminé avec succès!");
    println!("📋 Résumé:");
    println!("   - Réseau: {}", network);
    println!("   - Runtime: {}", runtime);
    if let Some(ref arch_val) = arch {
        println!("   - Architecture: {}", arch_val);
        println!("   - Image: {}:{}-{}", image_name, docker_tag, arch_val);
    } else {
        println!("   - Architecture: multi-arch (amd64, arm64)");
        println!("   - Image: {}:{}", image_name, docker_tag);
        println!("   - Image latest: {}:latest", image_name);
    }

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
