#!/usr/bin/env python3
"""
Decompile Minecraft jar to readable Java source for protocol reference.

Downloads the jar via `mc-launcher-cli`, then decompiles with CFR.
Mappings are handled separately (launcher doesn't need them).

Usage:
    python3 tools/mc-src/fetch.py
    python3 tools/mc-src/fetch.py --versions 1.21.8 26.1.2
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
SRC_DIR = SCRIPT_DIR.parent.parent / "mc-src"
MINECRAFT_DIR = SCRIPT_DIR.parent.parent / ".minecraft"
LAUNCHER_PKG = "mc-launcher-cli"

CFR_URL = (
    "https://repo1.maven.org/maven2/org/benf/cfr/"
    "0.152/cfr-0.152.jar"
)
MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"


def get_cfr() -> Path:
    path = TOOLS_DIR / "cfr.jar"
    if not path.exists():
        print("  Downloading CFR decompiler...")
        TOOLS_DIR.mkdir(parents=True, exist_ok=True)
        urllib.request.urlretrieve(CFR_URL, path)
    return path


def load_config() -> dict:
    config_path = SCRIPT_DIR / "config.json"
    if config_path.exists():
        with open(config_path) as f:
            return json.load(f)
    return {"versions": ["1.21.8", "latest"]}


def ensure_jar(version_id: str) -> Path:
    """Get jar, downloading via mc-launcher-cli if missing."""
    jar = MINECRAFT_DIR / "versions" / version_id / f"{version_id}.jar"
    if jar.exists():
        return jar

    print("  Jar not found. Running mc-launcher-cli to download...")
    result = subprocess.run(
        ["cargo", "run", "--package", LAUNCHER_PKG, "--",
         "--version", version_id, "--no-assets", "--no-launch"],
        capture_output=True, text=True, timeout=300,
    )
    if result.returncode != 0:
        print(f"  mc-launcher-cli failed: {result.stderr[:500]}")
        print("  Falling back to direct download...")
        vm = json.loads(urllib.request.urlopen(
            f"{MANIFEST_URL}"
        ).read())
        # Need to find version URL from manifest
        manifest = json.loads(urllib.request.urlopen(MANIFEST_URL).read())
        entry = [v for v in manifest["versions"] if v["id"] == version_id]
        if not entry:
            raise RuntimeError(f"Version {version_id} not found")
        vm = json.loads(urllib.request.urlopen(entry[0]["url"]).read())
        jar_url = vm["downloads"]["client"]["url"]
        jar.parent.mkdir(parents=True, exist_ok=True)
        print(f"  Downloading {jar.name}...")
        urllib.request.urlretrieve(jar_url, jar)

    if not jar.exists():
        raise RuntimeError(f"Failed to get jar for {version_id}")
    return jar


def decompile(version_id: str, jar_path: Path) -> bool:
    src_dir = SRC_DIR / version_id
    if src_dir.exists():
        n = len(list(src_dir.rglob("*.java")))
        print(f"  Already decompiled ({n} files), skipping")
        return True

    # Fetch version metadata to check for mappings
    manifest = json.loads(urllib.request.urlopen(MANIFEST_URL).read())
    entries = [v for v in manifest["versions"] if v["id"] == version_id]
    if not entries:
        print(f"  Version '{version_id}' not found in manifest")
        return False
    vm = json.loads(urllib.request.urlopen(entries[0]["url"]).read())

    cfr = get_cfr()
    cmd = [
        "java", "-jar", str(cfr),
        str(jar_path),
        "--outputdir", str(src_dir),
        "--silent", "true",
    ]

    has_mappings = "client_mappings" in vm["downloads"]
    if has_mappings:
        mappings_url = vm["downloads"]["client_mappings"]["url"]
        mappings_path = CACHE_DIR / "jars" / f"{version_id}-mappings.txt"
        mappings_path.parent.mkdir(parents=True, exist_ok=True)
        if not mappings_path.exists():
            print(f"  Downloading mappings...")
            urllib.request.urlretrieve(mappings_url, mappings_path)
        cmd += ["--obfuscationpath", str(mappings_path)]

    action = "obfuscated + mappings" if has_mappings else "unobfuscated"
    print(f"  Status: {action}, decompiling...")

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    if result.returncode != 0:
        print(f"  Failed: {result.stderr[:500]}")
        return False

    n = len(list(src_dir.rglob("*.java")))
    print(f"  Done! {n} source files -> {src_dir}")
    return True


def main():
    parser = argparse.ArgumentParser(description="Decompile Minecraft jar to source")
    parser.add_argument("--versions", nargs="*", help="Versions to decompile")
    args = parser.parse_args()

    config = load_config()
    versions = args.versions or config["versions"]

    manifest = json.loads(urllib.request.urlopen(MANIFEST_URL).read())
    print(f"Latest release: {manifest['latest']['release']}")

    success = 0
    for v in versions:
        if v == "latest":
            v = manifest["latest"]["release"]
        print(f"\n{'='*60}")
        print(f"  Version: {v}")
        print(f"{'='*60}")
        try:
            jar = ensure_jar(v)
            if decompile(v, jar):
                success += 1
        except Exception as e:
            print(f"  Error: {e}")

    print(f"\nDone. {success}/{len(versions)} versions processed.")
    print(f"Output: {SRC_DIR}/")


if __name__ == "__main__":
    main()
