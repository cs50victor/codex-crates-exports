#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from native_env import native_environment

ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


def digest():
    value = hashlib.sha256()
    value.update((ROOT / "_exports/crates.json").read_bytes())
    for directory in (ROOT / "codex-rs",):
        for path in sorted(directory.rglob("*")):
            if path.is_file() and "target" not in path.parts and "__pycache__" not in path.parts:
                value.update(path.relative_to(ROOT).as_posix().encode())
                value.update(path.read_bytes())
    return value.digest()


def invariants():
    catalog = json.loads((ROOT / "_exports/crates.json").read_text())
    metadata = json.loads(run("cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", str(ROOT / "codex-rs/Cargo.toml"), capture_output=True, text=True).stdout)
    packages = [package for package in metadata["packages"] if package["id"] in metadata["workspace_members"]]
    assert {package["name"] for package in packages} == {package["name"] for package in catalog}
    for package in packages:
        assert any(set(target["kind"]) & {"lib", "rlib", "proc-macro", "cdylib", "staticlib"} for target in package["targets"]), package["name"]
    before = digest()
    run("cargo", "run", "--locked", "--manifest-path", str(ROOT / "_exports/exporter/Cargo.toml"), "--", str(ROOT))
    assert before == digest(), "export is not idempotent"
    print(f"Validated all {len(packages)} library targets and deterministic exports")


def consumer(all_crates, git_source):
    catalog = json.loads((ROOT / "_exports/crates.json").read_text())
    names = {"codex-utils-redacted-string", "codex-websocket-client", "codex-api"}
    selected = catalog if all_crates else [entry for entry in catalog if entry["name"] in names]
    revision = run("git", "-C", str(ROOT), "rev-parse", "HEAD", capture_output=True, text=True).stdout.strip()
    with tempfile.TemporaryDirectory(prefix="codex-consumer-") as directory:
        project = Path(directory)
        manifest = '[package]\nname = "external-codex-consumer"\nversion = "0.0.0"\nedition = "2024"\n\n[workspace]\n\n[dependencies]\n'
        for entry in selected:
            if git_source:
                location = f'git = {json.dumps(ROOT.as_uri())}, rev = "{revision}"'
            else:
                location = f'path = {json.dumps(str(ROOT / entry["path"]))}'
            manifest += f'{entry["name"]} = {{ {location} }}\n'
        (project / "Cargo.toml").write_text(manifest)
        (project / "src").mkdir()
        (project / "src/main.rs").write_text('''fn main() {
    let secret = codex_utils_redacted_string::RedactedString::from("secret");
    assert_eq!(format!("{secret:?}"), "<redacted>");
    let _: Option<codex_websocket_client::WebSocketConnector> = None;
    let _: Option<codex_api::auth::AuthHeaderTelemetry> = None;
}
''')
        environment = os.environ.copy()
        environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target/exports-consumer"))
        if all_crates:
            environment.update(native_environment())
        run("cargo", "check", "--manifest-path", str(project / "Cargo.toml"), env=environment)
        if not all_crates:
            run("cargo", "run", "--manifest-path", str(project / "Cargo.toml"), env=environment)
        print(f"Checked {len(selected)} external {'Git' if git_source else 'path'} dependencies without consumer patches")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--invariants", action="store_true")
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--git", action="store_true")
    arguments = parser.parse_args()
    if arguments.invariants:
        invariants()
    else:
        consumer(arguments.all, arguments.git)
