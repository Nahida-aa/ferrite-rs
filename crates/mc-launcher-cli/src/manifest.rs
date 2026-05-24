use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<VersionEntry>,
}

#[derive(Deserialize, Clone)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Clone)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Deserialize, Clone)]
pub struct VersionMetadata {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    #[serde(default, rename = "assetIndex")]
    pub asset_index: AssetIndexRef,
    pub assets: String,
    pub downloads: Downloads,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Deserialize, Clone)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

#[derive(Deserialize, Clone)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    pub url: String,
}

impl Default for AssetIndexRef {
    fn default() -> Self {
        Self {
            id: String::new(),
            sha1: String::new(),
            size: 0,
            total_size: 0,
            url: String::new(),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Downloads {
    pub client: Artifact,
    #[serde(default)]
    pub client_mappings: Option<Artifact>,
    pub server: Artifact,
    #[serde(default)]
    pub server_mappings: Option<Artifact>,
}

#[derive(Deserialize, Clone)]
pub struct Artifact {
    pub url: String,
    #[serde(default)]
    pub sha1: String,
    #[serde(default)]
    pub size: i64,
}

impl Default for Artifact {
    fn default() -> Self {
        Self {
            url: String::new(),
            sha1: String::new(),
            size: 0,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Argument {
    Simple(String),
    Rule(RuleArgument),
}

#[derive(Deserialize, Clone)]
pub struct RuleArgument {
    pub rules: Vec<Rule>,
    pub value: ArgumentValue,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum ArgumentValue {
    String(String),
    List(Vec<String>),
}

#[derive(Deserialize, Clone)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<OsRestriction>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Deserialize, Clone)]
pub struct OsRestriction {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: LibraryDownloads,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub extract: Option<ExtractRules>,
}

#[derive(Deserialize, Clone, Default)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Deserialize, Clone)]
pub struct ExtractRules {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetEntry>,
    #[serde(default)]
    pub virtual_: Option<bool>,
}

#[derive(Deserialize, Clone)]
pub struct AssetEntry {
    pub hash: String,
    pub size: i64,
}

pub fn rule_matches(rules: &[Rule]) -> bool {
    let mut allowed = false;
    for rule in rules {
        let matches = rule_matches_single(rule);
        match rule.action.as_str() {
            "allow" if matches => allowed = true,
            "disallow" if matches => allowed = false,
            _ => {}
        }
    }
    allowed
}

fn rule_matches_single(rule: &Rule) -> bool {
    let os_ok = match &rule.os {
        None => true,
        Some(os) => {
            let name_ok = match &os.name {
                None => true,
                Some(n) => n == std::env::consts::OS,
            };
            let arch_ok = match &os.arch {
                None => true,
                Some(a) => a == std::env::consts::ARCH,
            };
            name_ok && arch_ok
        }
    };
    let features_ok = match &rule.features {
        None => true,
        Some(features) => features.iter().all(|(_, v)| *v),
    };
    os_ok && features_ok
}

/// Convert Maven coordinate `group:artifact:version` to relative path.
/// e.g. `"org.lwjgl:lwjgl:3.3.4"` → `"org/lwjgl/lwjgl/3.3.4/lwjgl-3.3.4.jar"`
pub fn maven_path(name: &str) -> PathBuf {
    let parts: Vec<&str> = name.splitn(3, ':').collect();
    if parts.len() != 3 {
        return PathBuf::from(name.replace(':', ".") + ".jar");
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    PathBuf::from_iter([&group, artifact, version, &format!("{}-{}.jar", artifact, version)])
}

/// Same as `maven_path` but for classifier jars (e.g. natives).
pub fn maven_classifier_path(name: &str, classifier: &str) -> PathBuf {
    let parts: Vec<&str> = name.splitn(3, ':').collect();
    if parts.len() != 3 {
        return PathBuf::from(name.replace(':', ".") + "-" + classifier + ".jar");
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    PathBuf::from_iter([
        &group,
        artifact,
        version,
        &format!("{}-{}-{}.jar", artifact, version, classifier),
    ])
}

pub fn get_natives_key() -> &'static str {
    match std::env::consts::OS {
        "linux" => "natives-linux",
        "macos" => "natives-macos",
        "windows" => "natives-windows",
        _ => "natives-linux",
    }
}
