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
use chrono::Utc;
use std::{process::Command, time::Instant};

pub async fn g1_data(dump_url: Option<String>) -> Result<()> {
    println!("🚀 Génération des données G1 avec Docker...");

    // Générer l'URL du dump si elle n'est pas fournie
    // Le backup cgeek est généré chaque jour à 23h00 UTC
    // On essaie d'abord la date du jour, puis la veille si le dump n'est pas encore disponible
    let dump_url = match dump_url {
        Some(url) => url,
        None => {
            let today = Utc::now().date_naive();
            let today_url = format!(
                "https://dl.cgeek.fr/public/auto-backup-g1-duniter-1.8.7_{}_23-00.tgz",
                today.format("%Y-%m-%d")
            );
            if url_exists(&today_url) {
                today_url
            } else {
                let yesterday = today - chrono::Duration::days(1);
                let yesterday_url = format!(
                    "https://dl.cgeek.fr/public/auto-backup-g1-duniter-1.8.7_{}_23-00.tgz",
                    yesterday.format("%Y-%m-%d")
                );
                println!(
                    "⚠️  Dump du jour non disponible, utilisation de la veille ({})",
                    yesterday.format("%Y-%m-%d")
                );
                yesterday_url
            }
        }
    };

    // Vérifier que Docker est disponible
    if !Command::new("docker").arg("--version").status()?.success() {
        return Err(anyhow::anyhow!(
            "Docker n'est pas installé ou n'est pas accessible"
        ));
    }

    // Vérifier que curl est disponible (pour le téléchargement avec reprise)
    if !Command::new("curl").arg("--version").status()?.success() {
        return Err(anyhow::anyhow!(
            "curl n'est pas installé. Veuillez installer curl pour continuer."
        ));
    }

    // Utiliser le répertoire courant
    let current_dir = std::env::current_dir()?;
    let work_dir = current_dir.join("release/network");
    std::fs::create_dir_all(&work_dir)?;

    // Vérifier si le fichier existe déjà et est complet
    let dump_file_path = work_dir.join("g1-dump.tgz");
    let need_download = if dump_file_path.exists() {
        // Vérifier la taille attendue via HTTP HEAD
        let expected_size = get_remote_file_size(&dump_url);
        let local_size = std::fs::metadata(&dump_file_path)?.len();

        match expected_size {
            Some(expected) if local_size == expected => {
                println!(
                    "📁 Fichier complet trouvé: {} ({:.0} Mo)",
                    dump_file_path.display(),
                    local_size as f64 / (1024.0 * 1024.0)
                );
                println!("⏭️  Utilisation du fichier existant, téléchargement ignoré.");
                false
            }
            Some(expected) => {
                println!(
                    "⚠️  Fichier incomplet trouvé: {:.0} Mo / {:.0} Mo attendus",
                    local_size as f64 / (1024.0 * 1024.0),
                    expected as f64 / (1024.0 * 1024.0)
                );
                println!("📥 Reprise du téléchargement...");
                true
            }
            None => {
                println!(
                    "📁 Fichier trouvé: {} ({:.0} Mo), impossible de vérifier la taille distante",
                    dump_file_path.display(),
                    local_size as f64 / (1024.0 * 1024.0)
                );
                println!("⏭️  Utilisation du fichier existant.");
                false
            }
        }
    } else {
        true
    };

    if need_download {
        println!("📥 Téléchargement du dump G1 depuis: {}", dump_url);
        let start_time = Instant::now();

        // Télécharger avec curl directement sur le host (supporte la reprise avec -C -)
        let status = Command::new("curl")
            .args([
                "--fail",
                "--location",
                "--continue-at",
                "-",
                "--output",
                &dump_file_path.to_string_lossy(),
                &dump_url,
            ])
            .status()?;

        let download_time = start_time.elapsed();

        if !status.success() {
            // Supprimer le fichier partiel si curl a échoué complètement
            if dump_file_path.exists() {
                let size = std::fs::metadata(&dump_file_path)?.len();
                if size == 0 {
                    std::fs::remove_file(&dump_file_path)?;
                }
            }
            return Err(anyhow::anyhow!(
                "Échec du téléchargement. Vérifiez l'URL et votre connexion.\n\
                URL: {}\n\
                💡 Relancez la commande pour reprendre le téléchargement.",
                dump_url
            ));
        }

        let file_size = std::fs::metadata(&dump_file_path)?.len();
        let file_size_mb = file_size as f64 / (1024.0 * 1024.0);
        let speed_mbps = if download_time.as_secs() > 0 {
            file_size_mb / download_time.as_secs_f64()
        } else {
            0.0
        };

        println!("✅ Téléchargement terminé: {}", dump_file_path.display());
        println!("📏 Taille du fichier: {:.0} Mo", file_size_mb);
        println!(
            "⏱️  Temps de téléchargement: {:.0}s",
            download_time.as_secs_f64()
        );
        println!("🚀 Débit moyen: {:.1} Mo/s", speed_mbps);
    }

    // Préparer les arguments Docker avec des variables pour éviter les problèmes de durée de vie
    let dump_file_str = work_dir.join("g1-dump.tgz").to_string_lossy().to_string();
    let output_dir_str = work_dir.to_string_lossy().to_string();
    let script_content = r#"
        set -e
        echo "📦 Extraction du dump..."
        mkdir /dump
        cd /dump
        cp /g1-dump.tgz /dump
        tar xvzf g1-dump.tgz
        echo "🔄 Conversion avec py-g1-migrator..."
        cd /py-g1-migrator
        echo "🔧 Génération main (1/4)..."
        ./main.py
        echo "🔧 Génération squid-block (2/4)..."
        ./squid-block.py
        echo "🔧 Génération squid-cert (3/4)..."
        ./squid-cert.py
        echo "🔧 Génération squid-tx (4/4)..."
        ./squid-tx.py
        echo "✅ Génération terminée!"
    "#;

    // Préparer les arguments de volume Docker
    let dump_volume = format!("{}:/g1-dump.tgz", dump_file_str);
    let output_volume = format!("{}:/py-g1-migrator/output", output_dir_str);

    // Exécuter le conteneur Docker avec py-g1-migrator
    // L'image est amd64 uniquement : forcer la plateforme pour compatibilité ARM
    let mut docker_args = vec!["run", "--rm"];
    if std::env::consts::ARCH == "aarch64" {
        docker_args.extend_from_slice(&["--platform", "linux/amd64"]);
    }
    docker_args.extend_from_slice(&[
        "-v",
        &dump_volume,
        "-v",
        &output_volume,
        "-e",
        "LEVELDB_PATH=/dump/duniter_default/data/leveldb",
        "registry.duniter.org/tools/py-g1-migrator:latest",
        "sh",
        "-c",
        script_content,
    ]);

    println!("🐳 Lancement du conteneur Docker...");
    let mut docker_cmd = Command::new("docker");
    docker_cmd.args(&docker_args);
    docker_cmd.stdout(std::process::Stdio::piped());
    docker_cmd.stderr(std::process::Stdio::piped());

    let mut child = docker_cmd.spawn()?;

    // Lire stdout et stderr en parallèle avec des threads
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stdout_handle = if let Some(stdout) = stdout {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                println!("{}", line);
            }
        })
    } else {
        std::thread::spawn(|| {})
    };

    let stderr_handle = if let Some(stderr) = stderr {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                eprintln!("{}", line);
            }
        })
    } else {
        std::thread::spawn(|| {})
    };

    // Attendre que le processus se termine
    let status = child.wait()?;

    // Attendre que les threads de lecture se terminent
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    if !status.success() {
        eprintln!("❌ Erreur lors de l'exécution Docker");
        return Err(anyhow::anyhow!("Échec de l'exécution Docker"));
    }

    // Copier les fichiers générés vers le répertoire courant
    let expected_files = vec![
        "genesis.json",
        "block_hist.json",
        "cert_hist.json",
        "tx_hist.json",
    ];

    for src in expected_files {
        let src_path = work_dir.join(src);
        if src_path.exists() {
            println!("📄 Généré: {} -> {}", src, src_path.display());
        } else {
            println!("⚠️ Fichier non trouvé: {}", src);
        }
    }

    println!("✅ Génération des données G1 terminée avec succès!");
    println!("📁 Fichiers disponibles dans: {}", work_dir.display());

    Ok(())
}

/// Vérifie qu'une URL distante existe via HTTP HEAD (code 200)
fn url_exists(url: &str) -> bool {
    Command::new("curl")
        .args([
            "--silent",
            "--head",
            "--fail",
            "--location",
            "--output",
            "/dev/null",
            url,
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Récupère la taille d'un fichier distant via HTTP HEAD
fn get_remote_file_size(url: &str) -> Option<u64> {
    let output = Command::new("curl")
        .args(["--silent", "--head", "--location", url])
        .output()
        .ok()?;

    let headers = String::from_utf8_lossy(&output.stdout);
    for line in headers.lines() {
        if let Some(value) = line.strip_prefix("content-length:") {
            return value.trim().parse().ok();
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            return value.trim().parse().ok();
        }
    }
    None
}
