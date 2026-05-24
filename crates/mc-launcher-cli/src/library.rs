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
    libs_base: &Path,
    natives_dir: &Path,
) -> Result<(Vec<ResolvedLibrary>, PathBuf)> {
    let natives_key = manifest::get_natives_key();
    let mut resolved = Vec::new();

    let natives_suffix = format!(":natives-{}", std::env::consts::OS);

    // Check once if natives dir already has .so files
    let has_natives = std::fs::read_dir(natives_dir).ok()
        .map(|mut e| e.any(|e| e.as_ref().is_ok_and(|e| e.path().extension().is_some_and(|x| x == "so"))))
        .unwrap_or(false);

    for lib in &metadata.libraries {
        if let Some(rules) = &lib.rules {
            if !manifest::rule_matches(rules) {
                continue;
            }
        }

        let is_native_classifier = lib.name.ends_with(&natives_suffix);

        // Resolve artifact path: use metadata `path` field if available, else maven_path
        let artifact_path = |artifact: &manifest::Artifact, name: &str| -> PathBuf {
            artifact.path.as_ref()
                .map(|p| PathBuf::from(p))
                .unwrap_or_else(|| manifest::maven_path(name))
        };

        // Main artifact – save to libraries/
        if let Some(artifact) = &lib.downloads.artifact {
            let rel_path = artifact_path(artifact, &lib.name);
            let jar_path = libs_base.join(&rel_path);
            cache
                .download_to(&artifact.url, &jar_path, Some(&artifact.sha1))
                .await?;
            resolved.push(ResolvedLibrary {
                path: jar_path,
                is_native: false,
            });
        }

        // Separate native classifier entries (e.g. org.lwjgl:lwjgl:3.3.3:natives-linux)
        if is_native_classifier {
            if let Some(artifact) = &lib.downloads.artifact {
                let jar_path = libs_base.join(&artifact_path(artifact, &lib.name));
                if !has_natives {
                    cache
                        .download_to(&artifact.url, &jar_path, Some(&artifact.sha1))
                        .await?;
                    extract_natives(&jar_path, natives_dir, lib.extract.as_ref()).await?;
                }
            }
        // Embedded natives (lib.natives field + classifiers)
        } else if let Some(natives_key) = lib.natives.as_ref().and_then(|n| n.get(natives_key)) {
            if let Some(classifiers) = &lib.downloads.classifiers {
                if let Some(native_artifact) = classifiers.get(natives_key) {
                    let rel_path = native_artifact.path.as_ref()
                        .map(|p| PathBuf::from(p))
                        .unwrap_or_else(|| manifest::maven_classifier_path(&lib.name, natives_key));
                    let jar_path = libs_base.join(&rel_path);
                    if !has_natives {
                        cache
                            .download_to(&native_artifact.url, &jar_path, Some(&native_artifact.sha1))
                            .await?;
                        extract_natives(&jar_path, natives_dir, lib.extract.as_ref()).await?;
                    }
                }
            }
        }
    }

    Ok((resolved, natives_dir.to_path_buf()))
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

        if entry_path.ends_with(".class")
            || entry_path.ends_with(".jar")
            || entry_path.starts_with("META-INF/")
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
