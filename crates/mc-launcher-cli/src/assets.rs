use crate::download::Cache;
use crate::manifest::VersionMetadata;
use anyhow::Result;
use std::path::Path;
use tokio::fs;

pub async fn ensure_assets(
    cache: &Cache,
    metadata: &VersionMetadata,
    assets_dir: &Path,
) -> Result<()> {
    // Download asset index
    let asset_index = metadata.asset_index.clone();
    let index_path = cache
        .get_jar(&asset_index.url, &format!("{}.json", asset_index.id), Some(&asset_index.sha1))
        .await?;

    let index_data = fs::read_to_string(&index_path).await?;
    let index: crate::manifest::AssetIndex = serde_json::from_str(&index_data)?;

    let objects_dir = assets_dir.join("objects");
    fs::create_dir_all(&objects_dir).await?;

    let total = index.objects.len();
    let mut done = 0;

    for (name, entry) in &index.objects {
        let hash = &entry.hash;
        let sub_dir = &hash[..2];
        let obj_path = objects_dir.join(sub_dir).join(hash);

        if !obj_path.exists() {
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                sub_dir, hash
            );

            let obj_dir = objects_dir.join(sub_dir);
            fs::create_dir_all(&obj_dir).await?;

            let response = cache.get(&url, Some(hash)).await;
            match response {
                Ok(p) => {
                    // Move to correct location
                    let target = obj_dir.join(hash);
                    if p != target {
                        fs::copy(&p, &target).await?;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to download asset {}: {}", name, e);
                }
            }
        }

        done += 1;
        if done % 500 == 0 || done == total {
            tracing::info!("  Assets: {}/{}", done, total);
        }
    }

    tracing::info!("  Assets complete: {}/{}", done, total);
    Ok(())
}
