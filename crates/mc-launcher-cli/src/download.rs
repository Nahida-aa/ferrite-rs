use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct Cache {
    client: Client,
    pub base: PathBuf,
}

impl Cache {
    pub fn new(base: PathBuf) -> Self {
        Self {
            client: Client::builder()
                .user_agent("mc-launcher-cli")
                .build()
                .expect("reqwest client"),
            base,
        }
    }

    pub async fn get(&self, url: &str, sha1: Option<&str>) -> Result<PathBuf> {
        let hash = hex::encode(Sha1::digest(url.as_bytes()));
        let dir = self.base.join("objects");
        let path = dir.join(&hash);

        if path.exists() {
            if let Some(expected) = sha1 {
                if self.verify_sha1(&path, expected).await {
                    return Ok(path);
                }
                tracing::debug!("SHA1 mismatch, re-downloading {}", url);
            } else {
                return Ok(path);
            }
        }

        fs::create_dir_all(&dir).await?;

        let temp_path = dir.join(format!("{}.tmp", hash));
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("download {}", url))?;

        let total = response.content_length().unwrap_or(0);
        let mut file = fs::File::create(&temp_path).await?;
        download_stream(response, &mut file, total, url).await?;

        file.flush().await?;
        fs::rename(&temp_path, &path).await?;

        if let Some(expected) = sha1 {
            if !self.verify_sha1(&path, expected).await {
                fs::remove_file(&path).await?;
                anyhow::bail!("SHA1 mismatch for {}", url);
            }
        }

        Ok(path)
    }

    /// Download a URL directly to a specific file path.
    /// Creates parent directories as needed. Verifies SHA1 if provided.
    pub async fn download_to(
        &self,
        url: &str,
        dest: &Path,
        sha1: Option<&str>,
    ) -> Result<()> {
        if dest.exists() {
            if let Some(expected) = sha1 {
                if self.verify_sha1(&dest.to_path_buf(), expected).await {
                    return Ok(());
                }
                tracing::debug!("SHA1 mismatch, re-downloading {}", dest.display());
            } else {
                return Ok(());
            }
        }

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).await?;
        }

        let temp_path = dest.with_extension("tmp");
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("download {}", dest.display()))?;

        let total = response.content_length().unwrap_or(0);
        let mut file = fs::File::create(&temp_path).await?;
        let display_name = url.split('/').last().unwrap_or(url);
        download_stream(response, &mut file, total, display_name).await?;

        file.flush().await?;
        fs::rename(&temp_path, dest).await?;

        if let Some(expected) = sha1 {
            if !self.verify_sha1(&dest.to_path_buf(), expected).await {
                fs::remove_file(dest).await?;
                anyhow::bail!("SHA1 mismatch for {}", url);
            }
        }

        Ok(())
    }

    pub async fn get_jar(
        &self,
        url: &str,
        name: &str,
        sha1: Option<&str>,
    ) -> Result<PathBuf> {
        let dir = self.base.join("jars");
        fs::create_dir_all(&dir).await?;
        let path = dir.join(name);

        if path.exists() {
            if let Some(expected) = sha1 {
                if self.verify_sha1(&path, expected).await {
                    return Ok(path);
                }
                tracing::debug!("SHA1 mismatch, re-downloading {}", name);
            } else {
                return Ok(path);
            }
        }

        let temp_path = dir.join(format!("{}.tmp", name));
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("download {}", name))?;

        let total = response.content_length().unwrap_or(0);
        let mut file = fs::File::create(&temp_path).await?;
        download_stream(response, &mut file, total, name).await?;

        file.flush().await?;
        fs::rename(&temp_path, &path).await?;

        if let Some(expected) = sha1 {
            if !self.verify_sha1(&path, expected).await {
                fs::remove_file(&path).await?;
                anyhow::bail!("SHA1 mismatch for {}", name);
            }
        }

        Ok(path)
    }

    async fn verify_sha1(&self, path: &PathBuf, expected: &str) -> bool {
        let data = fs::read(path).await;
        match data {
            Ok(bytes) => {
                let hash = hex::encode(Sha1::digest(&bytes));
                hash == expected
            }
            Err(_) => false,
        }
    }
}

async fn download_stream(
    response: reqwest::Response,
    file: &mut fs::File,
    total: u64,
    label: &str,
) -> Result<()> {
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_pct: u64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        if total > 0 {
            let pct = downloaded * 100 / total;
            if pct >= last_pct + 25 || downloaded == total {
                tracing::info!("  {} {}%", label, pct);
                last_pct = pct;
            }
        }
    }

    Ok(())
}
