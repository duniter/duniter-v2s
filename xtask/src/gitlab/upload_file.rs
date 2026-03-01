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

/// Upload un fichier vers GitLab et retourne l'URL de téléchargement
pub(super) async fn upload_file(
    gitlab_token: String,
    project_id: String,
    file_path: &Path,
    filename: String,
) -> Result<String> {
    let file_content = std::fs::read(file_path)?;

    let client = reqwest::Client::new();

    // Utiliser l'API REST de GitLab pour uploader un fichier
    let form = reqwest::multipart::Form::new()
        .text("file", filename.clone())
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_content).file_name(filename.clone()),
        );

    let response = client
        .post(format!(
            "https://git.duniter.org/api/v4/projects/{project_id}/uploads"
        ))
        .header("PRIVATE-TOKEN", gitlab_token)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        let upload_response: serde_json::Value = response.json().await?;

        if let Some(full_path) = upload_response.get("full_path") {
            return Ok(format!(
                "https://git.duniter.org{}",
                full_path.as_str().unwrap()
            ));
        }

        Err(anyhow!("Réponse d'upload invalide: {}", upload_response))
    } else {
        let error_text = response.text().await?;
        Err(anyhow!("Erreur lors de l'upload: {}", error_text))
    }
}
