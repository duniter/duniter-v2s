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
use chrono::{NaiveDateTime, NaiveTime, Utc};
use std::{process::Command, time::Instant};

pub async fn g1_data(dump_url: Option<String>) -> Result<()> {
    println!("🚀 Génération des données G1 avec Docker...");

    // Générer l'URL du dump si elle n'est pas fournie
    let dump_url = match dump_url {
        Some(url) => url,
        None => {
            let now = Utc::now();
            let today = now.date_naive();
            let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
            let today_midnight = NaiveDateTime::new(today, midnight);
            let date_str = today_midnight.format("%Y-%m-%d_%H-%M").to_string();
            format!(
                "https://dl.cgeek.fr/public/auto-backup-g1-duniter-1.8.7_{}.tgz",
                date_str
            )
        }
    };

    // Vérifier que Docker est disponible
    if !Command::new("docker").arg("--version").status()?.success() {
        return Err(anyhow::anyhow!(
            "Docker n'est pas installé ou n'est pas accessible"
        ));
    }

    // Utiliser le répertoire courant
    let current_dir = std::env::current_dir()?;
    let work_dir = current_dir.join("release/network");
    std::fs::create_dir_all(&work_dir)?;

    // Vérifier si le fichier existe déjà
    let dump_file_path = work_dir.join("g1-dump.tgz");
    if dump_file_path.exists() {
        println!("📁 Fichier existant trouvé: {}", dump_file_path.display());
        println!("⏭️  Utilisation du fichier existant, téléchargement ignoré.");
    } else {
        // Télécharger le dump G1 localement
        println!("📥 Téléchargement du dump G1 depuis: {}", dump_url);

        // Télécharger avec wget dans un conteneur Alpine
        println!("📥 Téléchargement avec wget dans un conteneur Alpine...");
        let start_time = Instant::now();

        let download_result = download_with_wget(&dump_url, &dump_file_path)?;
        let download_time = start_time.elapsed();

        if !download_result.success() {
            eprintln!("❌ Erreur lors du téléchargement avec wget:");
            eprintln!("💡 Conseil: Vérifiez votre connexion internet et réessayez");
            return Err(anyhow::anyhow!("Échec du téléchargement avec wget"));
        }

        // Calculer et afficher les statistiques de téléchargement
        let file_size = std::fs::metadata(&dump_file_path)?.len();
        let file_size_mb = file_size as f64 / (1024.0 * 1024.0);
        let download_speed = if download_time.as_secs() > 0 {
            file_size as f64 / download_time.as_secs() as f64
        } else {
            file_size as f64
        };
        let speed_mbps = download_speed / (1024.0 * 1024.0);

        println!("\n✅ Téléchargement terminé: {}", dump_file_path.display());
        println!("📏 Taille du fichier: {:.2} MB", file_size_mb);
        println!(
            "⏱️  Temps de téléchargement: {:.2}s",
            download_time.as_secs_f64()
        );
        println!("🚀 Débit moyen: {:.2} MB/s", speed_mbps);
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
        mv tmp/* duniter_default
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
    let docker_args = vec![
        "run",
        "--rm",
        "-v",
        &dump_volume,
        "-v",
        &output_volume,
        "-e",
        "LEVELDB_PATH=/dump/duniter_default/data/duniter_default/data/leveldb",
        "registry.duniter.org/tools/py-g1-migrator:latest",
        "sh",
        "-c",
        &script_content,
    ];

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

/// Télécharge un fichier avec wget dans un conteneur Alpine
fn download_with_wget(
    url: &str,
    output_path: &std::path::Path,
) -> Result<std::process::ExitStatus> {
    let output_dir = output_path.parent().unwrap();
    let filename = output_path.file_name().unwrap().to_string_lossy();

    let mut docker_cmd = Command::new("docker");
    docker_cmd.args([
        "run",
        "--rm",
        "-v",
        &format!("{}:/download", output_dir.to_string_lossy()),
        "alpine:latest",
        "wget",
        format!("--output-document=/download/{}", filename.as_ref()).as_str(),
        url,
    ]);

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

    Ok(status)
}
