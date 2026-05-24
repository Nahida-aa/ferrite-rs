#!/usr/bin/env python3
"""
Fetch Minecraft Java Edition source code for protocol reference.

Usage:
    python3 tools/mc-src/fetch.py                  # fetch configured versions
    python3 tools/mc-src/fetch.py --versions 1.21.8 26.1.2  # specific versions
"""

import argparse
import json
import os
import subprocess
import sys
import urllib.request
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
CACHE_DIR = SCRIPT_DIR / ".cache"
TOOLS_DIR = CACHE_DIR / "tools"
JARS_DIR = CACHE_DIR / "jars"
OUTPUT_DIR = SCRIPT_DIR.parent.parent / "mc-src"

CFR_URL = (
    "https://repo1.maven.org/maven2/org/benf/cfr/"
    "0.152/cfr-0.152.jar"
)
MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"


def ensure_dir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def download(url: str, dest: Path) -> None:
    if dest.exists():
        return
    print(f"  \u2193 {dest.name}")
    urllib.request.urlretrieve(url, dest)


def get_cfr() -> Path:
    path = TOOLS_DIR / "cfr.jar"
    if not path.exists():
        print("  Downloading CFR decompiler...")
        ensure_dir(TOOLS_DIR)
        urllib.request.urlretrieve(CFR_URL, path)
    return path


def load_config() -> dict:
    config_path = SCRIPT_DIR / "config.json"
    if config_path.exists():
        with open(config_path) as f:
            return json.load(f)
    return {"versions": ["1.21.8", "latest"]}


def process_version(manifest: dict, version_id: str) -> bool:
    print(f"\n{'='*60}")
    print(f"  Version: {version_id}")
    print(f"{'='*60}")

    if version_id == "latest":
        version_id = manifest["latest"]["release"]

    entries = [v for v in manifest["versions"] if v["id"] == version_id]
    if not entries:
        print(f"  \u2717 Version '{version_id}' not found in manifest")
        return False

    vm = json.loads(urllib.request.urlopen(entries[0]["url"]).read())
    src_dir = OUTPUT_DIR / version_id

    if src_dir.exists():
        n = len(list(src_dir.rglob("*.java")))
        print(f"  \u2192 Already exists ({n} files), skipping")
        return True

    jar_url = vm["downloads"]["client"]["url"]
    jar_path = JARS_DIR / f"{version_id}.jar"
    ensure_dir(JARS_DIR)
    download(jar_url, jar_path)

    cfr = get_cfr()
    cmd = [
        "java", "-jar", str(cfr),
        str(jar_path),
        "--outputdir", str(src_dir),
        "--silent", "true",
    ]

    has_mappings = "client_mappings" in vm["downloads"]
    if has_mappings:
        print(f"  Status: obfuscated (CFR + Mojang mappings)")
        mappings_url = vm["downloads"]["client_mappings"]["url"]
        mappings_path = JARS_DIR / f"{version_id}-mappings.txt"
        download(mappings_url, mappings_path)
        cmd += ["--obfuscationpath", str(mappings_path)]
    else:
        print(f"  Status: unobfuscated (CFR only)")

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    if result.returncode != 0:
        print(f"  \u2717 Failed: {result.stderr[:500]}")
        return False

    n = len(list(src_dir.rglob("*.java")))
    print(f"  \u2713 Done! {n} source files -> {src_dir}")
    return True


def main():
    parser = argparse.ArgumentParser(description="Fetch Minecraft source code")
    parser.add_argument("--versions", nargs="*", help="Versions to fetch")
    args = parser.parse_args()

    repo_root = SCRIPT_DIR.parent.parent
    os.chdir(repo_root)

    config = load_config()
    versions = args.versions or config["versions"]

    print("Fetching version manifest...")
    manifest = json.loads(urllib.request.urlopen(MANIFEST_URL).read())
    print(f"  Latest release: {manifest['latest']['release']}")
    print(f"  Latest snapshot: {manifest['latest']['snapshot']}")

    success = 0
    for v in versions:
        try:
            if process_version(manifest, v):
                success += 1
        except Exception as e:
            print(f"  \u2717 Error: {e}")

    print(f"\n{'='*60}")
    print(f"  Done. {success}/{len(versions)} versions processed.")
    print(f"  Output: {OUTPUT_DIR}/")
    if success < len(versions):
        sys.exit(1)


if __name__ == "__main__":
    main()
