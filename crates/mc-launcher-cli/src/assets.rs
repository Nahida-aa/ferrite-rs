use crate::download::Cache;
use crate::manifest::VersionMetadata;
use anyhow::Result;
use std::path::Path;
use tokio::fs;

pub async fn ensure_assets(
    cache: &Cache,
    metadata: &VersionMetadata,
    assets_root: &Path,
) -> Result<()> {
    let asset_index = metadata.asset_index.clone();
    let indexes_dir = assets_root.join("indexes");
    fs::create_dir_all(&indexes_dir).await?;
    let index_path = indexes_dir.join(format!("{}.json", asset_index.id));

    // Download asset index to assets/indexes/<id>.json
    cache
        .download_to(&asset_index.url, &index_path, Some(&asset_index.sha1))
        .await?;

    let index_data = fs::read_to_string(&index_path).await?;
    let index: crate::manifest::AssetIndex = serde_json::from_str(&index_data)?;

    let objects_dir = assets_root.join("objects");
    fs::create_dir_all(&objects_dir).await?;

    let total = index.objects.len();
    let mut done = 0;

    for (_name, entry) in &index.objects {
        let hash = &entry.hash;
        let sub_dir = &hash[..2];
        let obj_path = objects_dir.join(sub_dir).join(hash);

        if !obj_path.exists() {
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                sub_dir, hash
            );
            cache.download_to(&url, &obj_path, Some(hash)).await?;
        }

        done += 1;
        if done % 500 == 0 || done == total {
            tracing::info!("  Assets: {}/{}", done, total);
        }
    }

    tracing::info!("  Assets complete: {}/{}", done, total);
    Ok(())
}
