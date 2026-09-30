#!/usr/bin/env python3

"""Validate and publish a versioned Skyline plugin release."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
VERSION_FILE = ROOT / "VERSION"
CARGO_FILE = ROOT / "Cargo.toml"
NRO = "target/aarch64-skyline-switch/release/libfeth_better_durability.nro"


def run(*args: str, capture: bool = False) -> str:
  result = subprocess.run(
    args,
    cwd=ROOT,
    check=True,
    text=True,
    capture_output=capture,
  )
  return result.stdout.strip() if capture else ""


def ensure_clean_main() -> None:
  if run("git", "rev-parse", "--show-toplevel", capture=True) != str(ROOT):
    raise ValueError("run this script from its own repository")
  if run("git", "branch", "--show-current", capture=True) != "main":
    raise ValueError("release from main")
  if run("git", "status", "--porcelain", capture=True):
    raise ValueError("commit or remove working-tree changes before releasing")

  run("git", "fetch", "origin", "main", "--tags")
  if run("git", "rev-parse", "HEAD", capture=True) != run(
    "git", "rev-parse", "origin/main", capture=True
  ):
    raise ValueError("local main is not synchronized with origin/main")


def current_version() -> str:
  version = VERSION_FILE.read_text(encoding="utf-8").strip()
  if not re.fullmatch(r"\d+\.\d+\.\d+", version):
    raise ValueError(f"VERSION is not a stable semantic version: {version}")
  cargo_text = CARGO_FILE.read_text(encoding="utf-8")
  package = re.search(r"(?ms)^\[package\]\n(.*?)(?=^\[|\Z)", cargo_text)
  cargo_version = (
    re.search(r'(?m)^version = "(\d+\.\d+\.\d+)"$', package.group(1))
    if package
    else None
  )
  if cargo_version is None or cargo_version.group(1) != version:
    raise ValueError("Cargo.toml and VERSION do not agree")
  return version


def bump_version(version: str, part: str) -> str:
  major, minor, patch = (int(value) for value in version.split("."))
  if part == "major":
    return f"{major + 1}.0.0"
  if part == "minor":
    return f"{major}.{minor + 1}.0"
  return f"{major}.{minor}.{patch + 1}"


def write_version(version: str) -> None:
  old_cargo = CARGO_FILE.read_text(encoding="utf-8")
  new_cargo, count = re.subn(
    r'(?m)^version = "\d+\.\d+\.\d+"$',
    f'version = "{version}"',
    old_cargo,
    count=1,
  )
  if count != 1:
    raise ValueError("could not update Cargo.toml package version")
  VERSION_FILE.write_text(f"{version}\n", encoding="utf-8")
  CARGO_FILE.write_text(new_cargo, encoding="utf-8")


def verify_build() -> None:
  run("cargo", "fmt", "--check")
  run("cargo", "test", "--locked")
  run("cargo", "skyline", "check")
  run("cargo", "skyline", "build", "--release")
  run("python3", "tools/verify_nro.py", NRO)


def main() -> None:
  parser = argparse.ArgumentParser(description=__doc__)
  selection = parser.add_mutually_exclusive_group(required=True)
  selection.add_argument("--current", action="store_true", help="release VERSION as-is")
  selection.add_argument("--bump", choices=("patch", "minor", "major"))
  parser.add_argument("--yes", action="store_true", help="skip interactive confirmation")
  args = parser.parse_args()

  ensure_clean_main()
  old_version = current_version()
  version = old_version if args.current else bump_version(old_version, args.bump)
  tag = f"v{version}"
  if subprocess.run(
    ("git", "rev-parse", "--verify", "--quiet", f"refs/tags/{tag}"),
    cwd=ROOT,
    check=False,
    capture_output=True,
  ).returncode == 0:
    raise ValueError(f"tag already exists: {tag}")

  print(f"Release {tag} from {ROOT}", flush=True)
  if not args.yes and input("Build, commit, tag, and push? [y/N] ").lower() not in (
    "y", "yes"
  ):
    print("Release cancelled.")
    return

  verify_build()
  if args.current:
    # The first release already has its intended package version. An empty
    # commit records the release event without inventing a source change.
    run("git", "commit", "--allow-empty", "-m", f"chore: release {tag}")
  else:
    write_version(version)
    run("cargo", "check")
    run("git", "add", "VERSION", "Cargo.toml", "Cargo.lock")
    run("git", "commit", "-m", f"chore: release {tag}")

  run("git", "push", "origin", "main")
  run("git", "tag", "-a", tag, "-m", tag)
  run("git", "push", "origin", tag)
  print(f"Published {tag}. Check the release workflow before using its assets.")


if __name__ == "__main__":
  main()
