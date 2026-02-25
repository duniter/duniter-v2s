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

pub fn build_network_runtime(runtime: String) -> Result<()> {
    println!(
        "🚀 Construction du runtime réseau pour le runtime: {}",
        runtime
    );

    // Vérifier que Docker est disponible
    if !Command::new("docker").arg("--version").status()?.success() {
        return Err(anyhow::anyhow!(
            "Docker n'est pas installé ou n'est pas accessible. srtool nécessite Docker."
        ));
    }

    // Répertoire de travail
    let current_dir = std::env::current_dir()?;
    let work_dir = current_dir.join("release/network");

    // Créer le répertoire release s'il n'existe pas
    std::fs::create_dir_all(work_dir.clone())?;

    // Définir les variables comme dans la CI
    let srtool_output = work_dir.join("network_srtool_output.json");
    let srtool_output_filename = srtool_output.file_name().unwrap().to_string_lossy();
    println!("📄 SRTOOL_OUTPUT = {}", srtool_output.to_string_lossy());

    // Supprimer le fichier network_srtool_output.json s'il existe
    if srtool_output.exists() {
        std::fs::remove_file(srtool_output.clone())?;
        println!("🗑️  Fichier {} supprimé", srtool_output.to_string_lossy());
    }

    // Préparer les arguments Docker pour srtool
    let script_content = format!(
        r#"
        set -e
        echo "🚀 Démarrage de srtool..."
        echo "📁 Répertoire de travail: /build"
        echo "🔧 Runtime: {}"
        echo "📄 Sortie: {}"
        cd /build
        # Construire le runtime avec srtool
        echo "🔨 Construction du runtime avec srtool..."
        /srtool/build --app --json -cM | tee -a release/network/{}
        # Déplacer le fichier WASM généré
        echo "📦 Déplacement du fichier WASM..."
        mv /build/runtime/{}/target/srtool/release/wbuild/{}-runtime/{}_runtime.compact.compressed.wasm /build/release/network/
        mv /build/runtime/{}/target/srtool/release/wbuild/{}-runtime/{}_runtime.compact.wasm /build/release/network/
        echo "✅ Construction du runtime terminée!"
        "#,
        runtime,
        srtool_output_filename,
        srtool_output_filename,
        runtime,
        runtime,
        runtime,
        runtime,
        runtime,
        runtime
    );

    // Exécuter le conteneur Docker avec srtool
    // srtool n'est disponible qu'en amd64 : forcer la plateforme pour compatibilité ARM (Mac M1/M2/M3/M4)
    let is_arm = std::env::consts::ARCH == "aarch64";
    if is_arm {
        eprintln!("⚠️  Architecture ARM détectée. L'image srtool est amd64 uniquement.");
        eprintln!("   Le build tournera sous émulation (Rosetta/QEMU) et sera très lent.");
        eprintln!("   Assurez-vous que Docker Desktop a au moins 16 Go de RAM allouée.");
        eprintln!("   Pour un build plus rapide, utilisez une machine Linux x86_64.");
    }

    let build_volume = format!("{}:/build", current_dir.to_string_lossy());
    let package = format!("PACKAGE={}-runtime", runtime);
    let runtime_dir = format!("RUNTIME_DIR=runtime/{}", runtime);
    // Volume Docker nommé pour le cache srtool : évite les problèmes de permissions
    // VirtioFS (UID mapping) tout en persistant le cache entre les runs
    let cache_volume_name = format!("srtool-cache-{}", runtime);
    let cache_volume = format!(
        "{}:/build/runtime/{}/target/srtool",
        cache_volume_name, runtime
    );
    // Initialiser les permissions du volume (créé root:root par défaut)
    // pour que l'utilisateur builder (1001) puisse écrire dedans
    let init_volume = format!("{}:/srtool-cache", cache_volume_name);
    let init_status = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &init_volume,
            "alpine",
            "chown",
            "1001:1001",
            "/srtool-cache",
        ])
        .status()?;
    if !init_status.success() {
        eprintln!("⚠️  Impossible d'initialiser les permissions du volume cache");
    }
    let mut docker_args = vec!["run", "--rm"];
    // Forcer la plateforme amd64 pour que Docker utilise l'émulation sur ARM
    if is_arm {
        docker_args.extend_from_slice(&["--platform", "linux/amd64"]);
    }
    docker_args.extend_from_slice(&[
        "-v",
        &build_volume,
        "-v",
        &cache_volume,
        "-e",
        runtime_dir.as_str(),
        "-e",
        package.as_str(),
        "paritytech/srtool:1.88.0",
        "sh",
        "-c",
        &script_content,
    ]);

    println!("🐳 Lancement du conteneur srtool...");
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
        eprintln!("❌ Erreur lors de l'exécution de srtool");
        return Err(anyhow::anyhow!("Échec de l'exécution de srtool"));
    }

    // Vérifier que le fichier WASM a été généré
    let wasm_file = format!(
        "release/network/{}_runtime.compact.compressed.wasm",
        runtime
    );
    if !std::path::Path::new(&wasm_file).exists() {
        return Err(anyhow::anyhow!(
            "Le fichier WASM n'a pas été généré: {}",
            wasm_file
        ));
    }

    println!("✅ Runtime réseau généré avec succès!");
    println!("📁 Fichiers disponibles dans le répertoire 'release/network':");
    println!("   - {}", wasm_file);
    if srtool_output.exists() {
        println!("   - {}", srtool_output.to_string_lossy());
    }

    Ok(())
}
