#!/usr/bin/env python3
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def native_environment():
    os.environ["CODEX_REPO_ROOT"] = str(ROOT)
    sys.path.insert(0, str(ROOT / "scripts"))
    from codex_package.targets import TARGET_SPECS
    from codex_package.v8 import resolve_codex_v8_cargo_env

    version = subprocess.check_output(["rustc", "-vV"], text=True)
    host = next(line.removeprefix("host: ") for line in version.splitlines() if line.startswith("host: "))
    target = os.environ.get("CARGO_BUILD_TARGET", host)
    if target not in TARGET_SPECS:
        raise RuntimeError(f"upstream has no native V8 artifact configuration for {target}")
    return resolve_codex_v8_cargo_env(TARGET_SPECS[target])


if __name__ == "__main__":
    environment = os.environ.copy()
    environment.update(native_environment())
    if len(sys.argv) < 2:
        raise SystemExit("usage: python3 _exports/native_env.py cargo <arguments>")
    raise SystemExit(subprocess.run(sys.argv[1:], env=environment).returncode)
