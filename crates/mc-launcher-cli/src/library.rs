use crate::download::Cache;
use crate::manifest::{self};
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct ResolvedLibrary {
    pub path: PathBuf,
    pub is_native: bool,
}

pub async fn resolve_libraries(
    cache: &Cache,
    metadata: &manifest::VersionMetadata,
) -> Result<(Vec<ResolvedLibrary>, PathBuf)> {
    let natives_dir = cache.base.join("natives");
    fs::create_dir_all(&natives_dir).await?;

    let natives_key = manifest::get_natives_key();
    let mut resolved = Vec::new();

    for lib in &metadata.libraries {
        // Check rules
        if let Some(rules) = &lib.rules {
            if !manifest::rule_matches(rules) {
                continue;
            }
        }

        // Main artifact
        if let Some(artifact) = &lib.downloads.artifact {
            let name = lib.name.replace(':', ".") + ".jar";
            let path = cache
                .get_jar(&artifact.url, &name, Some(&artifact.sha1))
                .await?;
            resolved.push(ResolvedLibrary {
                path,
                is_native: false,
            });
        }

        // Native classifiers
        if let Some(natives_key) = lib.natives.as_ref().and_then(|n| n.get(natives_key)) {
            if let Some(classifiers) = &lib.downloads.classifiers {
                if let Some(native_artifact) = classifiers.get(natives_key) {
                    let name = lib.name.replace(':', ".") + "-" + natives_key + ".jar";
                    let jar_path = cache
                        .get_jar(&native_artifact.url, &name, Some(&native_artifact.sha1))
                        .await?;
                    extract_natives(&jar_path, &natives_dir, lib.extract.as_ref()).await?;
                    resolved.push(ResolvedLibrary {
                        path: jar_path,
                        is_native: true,
                    });
                }
            }
        }
    }

    Ok((resolved, natives_dir))
}

pub async fn extract_natives(
    jar_path: &Path,
    natives_dir: &Path,
    extract_rules: Option<&manifest::ExtractRules>,
) -> Result<()> {
    let file = fs::read(jar_path).await?;
    let reader = std::io::Cursor::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;

    let exclude_prefixes = extract_rules
        .map(|r| r.exclude.clone())
        .unwrap_or_default();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = entry.name().to_string();

        if entry.is_dir() {
            continue;
        }

        if !entry_path.contains('/')
            || entry_path.ends_with(".class")
            || entry_path.ends_with(".jar")
        {
            continue;
        }

        if exclude_prefixes
            .iter()
            .any(|p| entry_path.starts_with(p))
        {
            continue;
        }

        let out_path = natives_dir.join(&entry_path);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut data = Vec::with_capacity(entry.size() as usize);
        use std::io::Read;
        entry.read_to_end(&mut data)?;

        fs::write(&out_path, data).await?;
    }

    Ok(())
}
