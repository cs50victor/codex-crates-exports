#!/usr/bin/env python3
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import tarfile
import tempfile
import subprocess
import tomllib
from urllib.request import urlopen

ROOT = Path(__file__).resolve().parents[1]


def identity(package):
    return (package["name"], package["version"], package.get("source", ""))


def resolve_dependency(specification, packages):
    parts = specification.split(maxsplit=2)
    matches = [package for package in packages if package["name"] == parts[0]]
    if len(parts) > 1:
        matches = [package for package in matches if package["version"] == parts[1]]
    if len(parts) > 2:
        matches = [package for package in matches if package.get("source", "") == parts[2].strip("()")]
    if len(matches) != 1:
        raise RuntimeError(f"ambiguous Cargo.lock dependency: {specification}")
    return identity(matches[0])


def prepare():
    provenance = json.loads((ROOT / "_exports/upstream.json").read_text())
    revision = os.environ.get("CODEX_EXPORT_UPSTREAM", provenance["revision"])
    lock = subprocess.check_output(["git", "-C", str(ROOT), "show", f"{revision}:codex-rs/Cargo.lock"], text=True)
    manifest = subprocess.check_output(["git", "-C", str(ROOT), "show", f"{revision}:codex-rs/Cargo.toml"], text=True)
    packages = tomllib.loads(lock)["package"]
    patches = tomllib.loads(manifest).get("patch", {}).get("crates-io", {})
    affected = set()
    overrides = []
    for name, source in patches.items():
        expected = "git+" + source["git"].removesuffix(".git")
        matches = [package for package in packages if package["name"] == name and package.get("source", "").split("?")[0].removesuffix(".git") == expected]
        if len(matches) != 1:
            raise RuntimeError(f"patch does not identify exactly one locked package: {name}")
        package = matches[0]
        affected.add(identity(package))
        overrides.append({"name": name, "version": package["version"], "source": source})
    initial = set(affected)
    while True:
        additions = {
            identity(package) for package in packages
            if package.get("source", "").startswith("registry+")
            and any(resolve_dependency(dependency, packages) in affected for dependency in package.get("dependencies", []))
        } - affected
        if not additions:
            break
        affected.update(additions)
    selected = sorted((package for package in packages if identity(package) in affected - initial), key=identity)
    destination = ROOT / "_exports/vendor"
    records = []
    with tempfile.TemporaryDirectory(prefix="codex-crate-vendor-") as directory:
        staging = Path(directory)
        for package in selected:
            name, version = package["name"], package["version"]
            if not package.get("source", "").startswith("registry+"):
                raise RuntimeError(f"unsupported non-registry dependency ancestor: {name}")
            archive_name = f"{name}-{version}"
            cache = list((Path.home() / ".cargo/registry/cache").glob(f"*/{archive_name}.crate"))
            if cache:
                data = cache[0].read_bytes()
            else:
                with urlopen(f"https://static.crates.io/crates/{name}/{archive_name}.crate", timeout=120) as response:
                    data = response.read()
            if hashlib.sha256(data).hexdigest() != package["checksum"]:
                raise RuntimeError(f"upstream Cargo.lock checksum mismatch: {archive_name}")
            with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
                if any(not member.name.startswith(archive_name + "/") for member in archive.getmembers()):
                    raise RuntimeError(f"unexpected registry archive layout: {archive_name}")
                archive.extractall(staging, filter="data")
            records.append({"name": name, "version": version, "checksum": package["checksum"], "path": f"_exports/vendor/{archive_name}"})
            overrides.append({"name": name, "version": version, "source": {"path": f"_exports/vendor/{archive_name}"}})
        if destination.exists():
            shutil.rmtree(destination)
        shutil.copytree(staging, destination)
    (ROOT / "_exports/vendor.json").write_text(json.dumps(records, indent=2) + "\n")
    (ROOT / "_exports/dependency-sources.json").write_text(json.dumps(overrides, indent=2) + "\n")
    print(f"Prepared {len(records)} checksum-verified dependencies to propagate upstream patches")


if __name__ == "__main__":
    prepare()
