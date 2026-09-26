import unittest

from vendor import identity, resolve_dependency


class DependencyIdentityTests(unittest.TestCase):
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
