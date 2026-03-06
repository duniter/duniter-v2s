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
    println!("🚀 Construction du runtime réseau pour le runtime: {runtime}");

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

    // Récupérer l'UID:GID de l'utilisateur hôte
    let uid_output = Command::new("id").arg("-u").output()?;
    let gid_output = Command::new("id").arg("-g").output()?;
    let host_uid = String::from_utf8_lossy(&uid_output.stdout)
        .trim()
        .to_string();
    let host_gid = String::from_utf8_lossy(&gid_output.stdout)
        .trim()
        .to_string();

    // Préparer les arguments Docker pour srtool.
    // On exécute directement le conteneur avec l'UID/GID hôte pour éviter les
    // fichiers root sur le volume monté. Le groupe builder (1001) est ajouté
    // pour conserver l'accès en lecture à la toolchain préinstallée dans /home/builder.
    let script_content = format!(
        r#"
        set -e
        export HOME=/home/builder
        echo "🚀 Démarrage de srtool..."
        echo "📁 Répertoire de travail: /build"
        echo "🔧 Runtime: {runtime}"
        echo "📄 Sortie: {srtool_output_filename}"
        cd /build
        echo "🔨 Construction du runtime avec srtool..."
        /srtool/build --app --json -cM | tee -a release/network/{srtool_output_filename}
        echo "📦 Déplacement du fichier WASM..."
        mv /build/runtime/{runtime}/target/srtool/release/wbuild/{runtime}-runtime/{runtime}_runtime.compact.compressed.wasm /build/release/network/
        mv /build/runtime/{runtime}/target/srtool/release/wbuild/{runtime}-runtime/{runtime}_runtime.compact.wasm /build/release/network/
        echo "✅ Construction du runtime terminée!"
        "#
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

    // Ajouter :z pour la compatibilité SELinux (Fedora, RHEL, CentOS…)
    // Inoffensif sur les systèmes sans SELinux (macOS, Ubuntu, etc.)
    let build_volume = format!("{}:/build:z", current_dir.to_string_lossy());
    let builder_gid = std::env::var("SRTOOL_BUILDER_GID").unwrap_or_else(|_| "1001".to_string());
    let user_mapping = format!("{host_uid}:{host_gid}");
    let package = format!("PACKAGE={runtime}-runtime");
    let runtime_dir = format!("RUNTIME_DIR=runtime/{runtime}");
    let mut docker_args = vec!["run", "--rm"];
    // Forcer la plateforme amd64 pour que Docker utilise l'émulation sur ARM
    if is_arm {
        docker_args.extend_from_slice(&["--platform", "linux/amd64"]);
    }
    docker_args.extend_from_slice(&[
        "-u",
        user_mapping.as_str(),
        "--group-add",
        builder_gid.as_str(),
        "-v",
        &build_volume,
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
                println!("{line}");
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
                eprintln!("{line}");
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
    let wasm_file = format!("release/network/{runtime}_runtime.compact.compressed.wasm");
    if !std::path::Path::new(&wasm_file).exists() {
        return Err(anyhow::anyhow!(
            "Le fichier WASM n'a pas été généré: {}",
            wasm_file
        ));
    }

    println!("✅ Runtime réseau généré avec succès!");
    println!("📁 Fichiers disponibles dans le répertoire 'release/network':");
    println!("   - {wasm_file}");
    if srtool_output.exists() {
        println!("   - {}", srtool_output.to_string_lossy());
    }

    Ok(())
}
