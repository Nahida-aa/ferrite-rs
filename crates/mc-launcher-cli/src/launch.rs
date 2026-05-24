use crate::library::ResolvedLibrary;
use crate::manifest::{self, Argument, VersionMetadata};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Child, Command};

pub struct LaunchConfig {
    pub java_path: PathBuf,
    pub version: String,
    pub server: Option<String>,
    pub port: Option<u16>,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
    pub game_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub assets_root: PathBuf,
    pub libraries: Vec<ResolvedLibrary>,
    pub natives_dir: PathBuf,
    pub metadata: VersionMetadata,
    pub client_jar: PathBuf,
    pub no_assets: bool,
}

pub fn launch(config: LaunchConfig) -> Result<Child> {
    let classpath = build_classpath(&config);
    let natives = config.natives_dir.to_string_lossy().to_string();

    // JVM args
    let mut jvm_args: Vec<String> = Vec::new();

    // Standard JVM args
    jvm_args.push("-Xmx2G".to_string());
    jvm_args.push("-Xms512M".to_string());
    jvm_args.push(format!("-Djava.library.path={}", natives));
    jvm_args.push(format!(
        "-Dminecraft.client.jar={}",
        config.client_jar.display()
    ));

    // Parse version JSON jvm arguments
    if let Some(ref args) = config.metadata.arguments {
        for arg in &args.jvm {
            match arg {
                Argument::Simple(s) => {
                    let sub = substitute(s, &config);
                    jvm_args.push(sub);
                }
                Argument::Rule(rule) => {
                    if manifest::rule_matches(&rule.rules) {
                        let values = match &rule.value {
                            manifest::ArgumentValue::String(s) => vec![substitute(s, &config)],
                            manifest::ArgumentValue::List(l) => {
                                l.iter().map(|s| substitute(s, &config)).collect()
                            }
                        };
                        jvm_args.extend(values);
                    }
                }
            }
        }
    }

    // Classpath
    jvm_args.push("-cp".to_string());
    jvm_args.push(classpath);

    // Main class
    jvm_args.push(config.metadata.main_class.clone());

    // Game args
    let mut has_username = false;
    if let Some(ref args) = config.metadata.arguments {
        for arg in &args.game {
            match arg {
                Argument::Simple(s) => {
                    let sub = substitute(s, &config);
                    if sub == "--username" {
                        has_username = true;
                    }
                    jvm_args.push(sub);
                }
                Argument::Rule(rule) => {
                    if manifest::rule_matches(&rule.rules) {
                        let values = match &rule.value {
                            manifest::ArgumentValue::String(s) => vec![substitute(s, &config)],
                            manifest::ArgumentValue::List(l) => {
                                l.iter().map(|s| substitute(s, &config)).collect()
                            }
                        };
                        for v in &values {
                            if v == "--username" {
                                has_username = true;
                            }
                        }
                        jvm_args.extend(values);
                    }
                }
            }
        }
    } else if let Some(ref legacy_args) = config.metadata.minecraft_arguments {
        let sub = substitute(legacy_args, &config);
        for token in shlex::split(&sub).unwrap_or_default() {
            if token == "--username" {
                has_username = true;
            }
            jvm_args.push(token);
        }
    }

    // Ensure username is set
    if !has_username {
        jvm_args.push("--username".to_string());
        jvm_args.push(config.username.clone());
    }

    // Add server args if specified
    if let Some(ref server) = config.server {
        jvm_args.push("--server".to_string());
        jvm_args.push(server.clone());
        jvm_args.push("--port".to_string());
        jvm_args.push(config.port.unwrap_or(25565).to_string());
    }

    tracing::info!("Launching: {} {}", config.java_path.display(), jvm_args[0]);
    tracing::debug!("Full args: {:?}", jvm_args);

    let child = Command::new(&config.java_path)
        .args(&jvm_args)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .context("Failed to launch Java")?;

    Ok(child)
}

fn build_classpath(config: &LaunchConfig) -> String {
    let separator = if cfg!(windows) { ";" } else { ":" };
    let mut entries: Vec<String> = Vec::new();

    entries.push(config.client_jar.to_string_lossy().to_string());

    for lib in &config.libraries {
        if !lib.is_native {
            entries.push(lib.path.to_string_lossy().to_string());
        }
    }

    entries.join(separator)
}

fn substitute(template: &str, config: &LaunchConfig) -> String {
    let mut result = template.to_string();

    let subs: Vec<(&str, String)> = vec![
        ("${natives_directory}", config.natives_dir.to_string_lossy().to_string()),
        ("${library_directory}", config.game_dir.join("libraries").to_string_lossy().to_string()),
        ("${classpath_separator}", if cfg!(windows) { ";" } else { ":" }.to_string()),
        ("${launcher_name}", "mc-launcher-cli".to_string()),
        ("${launcher_version}", "0.1.0".to_string()),
        ("${version_name}", config.version.clone()),
        ("${assets_root}", config.assets_root.to_string_lossy().to_string()),
        ("${game_assets}", config.assets_dir.to_string_lossy().to_string()),
        ("${assets_index_name}", config.metadata.assets.clone()),
        ("${auth_player_name}", config.username.clone()),
        ("${auth_access_token}", config.access_token.clone()),
        ("${auth_uuid}", config.uuid.clone()),
        ("${user_properties}", "{}".to_string()),
        ("${user_type}", config.user_type.clone()),
        ("${version_type}", config.metadata.kind.clone()),
    ];

    for (key, val) in subs {
        result = result.replace(key, &val);
    }

    result
}
