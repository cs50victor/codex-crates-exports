#!/usr/bin/env python3
import argparse
import datetime
import errno
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
UPSTREAM = "https://github.com/openai/codex.git"
PROTECTED = ("_exports/", ".github/workflows/exports.yml", ".github/workflows/freshness.yml", "README.txt")


def run(*args, **kwargs):
    return subprocess.run(args, cwd=ROOT, check=True, **kwargs)


def replace_upstream(archive):
    with tempfile.TemporaryDirectory(prefix="codex-upstream-") as directory:
        staging = Path(directory).resolve()
        with tarfile.open(fileobj=io.BytesIO(archive)) as source:
            source.extractall(staging, filter="data")
        if (staging / "_exports").exists() or (staging / "README.txt").exists():
            raise RuntimeError("upstream now owns a reserved exporter path")
        for source in staging.rglob("*"):
            if source.is_symlink():
                target = source.resolve().relative_to(staging).as_posix()
                if target.startswith((*PROTECTED, ".git/", ".jj/")):
                    raise RuntimeError(f"upstream symlink targets protected path: {source}")
        tracked = run("git", "ls-files", "-z", capture_output=True).stdout.decode().split("\0")
        directories = set()
        for name in filter(None, tracked):
            if not name.startswith(PROTECTED):
                path = ROOT / name
                path.unlink(missing_ok=True)
                directories.update(parent for parent in path.parents if parent != ROOT and ROOT in parent.parents)
        for directory in sorted(directories, key=lambda path: len(path.parts), reverse=True):
            try:
                directory.rmdir()
            except FileNotFoundError:
                pass
            except OSError as error:
                if error.errno not in (errno.ENOTEMPTY, errno.EEXIST):
                    raise
        for source in staging.rglob("*"):
            relative = source.relative_to(staging)
            name = relative.as_posix()
            if name.startswith(".github/workflows/") or name == "README.md":
                continue
            destination = ROOT / relative
            if source.is_symlink():
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.symlink_to(source.readlink())
            elif source.is_file():
                destination.parent.mkdir(parents=True, exist_ok=True)
                if destination.is_dir():
                    raise RuntimeError(f"untracked directory conflicts with upstream file: {destination}")
                shutil.copy2(source, destination)


def refresh():
    run("git", "fetch", "--no-tags", "--depth=1", UPSTREAM, "main")
    revision = run("git", "rev-parse", "FETCH_HEAD", capture_output=True, text=True).stdout.strip()
    archive = run("git", "archive", revision, capture_output=True).stdout
    replace_upstream(archive)
    with (ROOT / "codex-rs/rust-toolchain.toml").open("rb") as file:
        toolchain = tomllib.load(file)["toolchain"]["channel"]
    run("rustup", "toolchain", "install", toolchain, "--profile", "minimal")
    os.environ["RUSTUP_TOOLCHAIN"] = toolchain
    if environment_file := os.environ.get("GITHUB_ENV"):
        with open(environment_file, "a") as file:
            file.write(f"RUSTUP_TOOLCHAIN={toolchain}\n")
    run("cargo", "run", "--locked", "--manifest-path", "_exports/exporter/Cargo.toml", "--", str(ROOT))
    run("cargo", "update", "--workspace", "--manifest-path", "codex-rs/Cargo.toml")
    provenance = {
        "repository": UPSTREAM,
        "revision": revision,
        "checked_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    }
    (ROOT / "_exports/upstream.json").write_text(json.dumps(provenance, indent=2) + "\n")
    return revision


def freshness(hours):
    provenance = json.loads((ROOT / "_exports/upstream.json").read_text())
    checked = datetime.datetime.fromisoformat(provenance["checked_at"])
    age = datetime.datetime.now(datetime.timezone.utc) - checked
    if age > datetime.timedelta(hours=hours) or age < datetime.timedelta(minutes=-5):
        raise RuntimeError(f"last successful upstream check is {age}; allowed age is {hours} hours")
    print(f"Upstream check age: {age}; revision: {provenance['revision']}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--freshness-hours", type=int)
    arguments = parser.parse_args()
    if arguments.freshness_hours is not None:
        freshness(arguments.freshness_hours)
    else:
        print(refresh())
