import unittest
import json
import os
from pathlib import Path
import tempfile
from unittest.mock import patch

import vendor
from vendor import identity, resolve_dependency


class DependencyIdentityTests(unittest.TestCase):
    def test_regeneration_uses_committed_inputs_without_upstream_git_objects(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "codex-rs").mkdir()
            (root / "_exports").mkdir()
            (root / "codex-rs/Cargo.lock").write_text('[[package]]\nname="leaf"\nversion="1.0.0"\nsource="git+https://example.test/leaf?rev=abc#abc"\n')
            (root / "codex-rs/Cargo.toml").write_text('[patch.crates-io]\nleaf={git="https://example.test/leaf",rev="abc"}\n')
            (root / "_exports/upstream.json").write_text(json.dumps({"revision": "abc"}))
            vendor.snapshot_dependencies(root, "abc")
            (root / "codex-rs/Cargo.lock").write_text("changed after export")
            with patch.object(vendor, "ROOT", root), patch.dict(os.environ, {"CODEX_EXPORT_UPSTREAM": "abc"}):
                vendor.prepare()
            overrides = json.loads((root / "_exports/dependency-sources.json").read_text())
            self.assertEqual(overrides[0]["source"]["rev"], "abc")
            self.assertFalse((root / ".git").exists())

    def test_resolves_version_and_source_without_merging_distinct_packages(self):
        registry = {"name": "leaf", "version": "1.0.0", "source": "registry+https://example.test"}
        patched = {"name": "leaf", "version": "1.0.0", "source": "git+https://example.test/leaf#abc"}
        older = {"name": "leaf", "version": "0.9.0", "source": "registry+https://example.test"}
        packages = [registry, patched, older]
        self.assertEqual(resolve_dependency("leaf 1.0.0 (git+https://example.test/leaf#abc)", packages), identity(patched))
        self.assertEqual(resolve_dependency("leaf 0.9.0", packages), identity(older))
        with self.assertRaises(RuntimeError):
            resolve_dependency("leaf", packages)
        with self.assertRaises(RuntimeError):
            resolve_dependency("leaf 1.0.0", packages)


if __name__ == "__main__":
    unittest.main()
