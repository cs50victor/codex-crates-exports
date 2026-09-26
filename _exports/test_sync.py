import datetime
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import sync


def archive(files, symlinks=None):
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w") as tar:
        for name, content in files.items():
            entry = tarfile.TarInfo(name)
            body = content.encode()
            entry.size = len(body)
            tar.addfile(entry, io.BytesIO(body))
        for name, target in (symlinks or {}).items():
            entry = tarfile.TarInfo(name)
            entry.type = tarfile.SYMTYPE
            entry.linkname = target
            tar.addfile(entry)
    return buffer.getvalue()


class SyncTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.patch = patch.object(sync, "ROOT", self.root)
        self.patch.start()
        self.addCleanup(self.patch.stop)
        subprocess.run(["git", "init", "--quiet", str(self.root)], check=True)
        self.write("_exports/exporter/src/main.rs", "exporter")
        self.write(".github/workflows/exports.yml", "our workflow")
        self.write("README.txt", "our readme")
        self.write("old.txt", "old upstream")
        subprocess.run(["git", "-C", str(self.root), "add", "."], check=True)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def test_replaces_renamed_and_deleted_files_and_preserves_overlay(self):
        sync.replace_upstream(archive({"new.txt": "upstream", "README.md": "upstream readme", ".github/workflows/unwanted.yml": "upstream workflow"}))
        self.assertFalse((self.root / "old.txt").exists())
        self.assertEqual((self.root / "new.txt").read_text(), "upstream")
        self.assertEqual((self.root / "README.txt").read_text(), "our readme")
        self.assertEqual((self.root / "_exports/exporter/src/main.rs").read_text(), "exporter")
        self.assertEqual((self.root / ".github/workflows/exports.yml").read_text(), "our workflow")
        self.assertFalse((self.root / ".github/workflows/unwanted.yml").exists())
        self.assertFalse((self.root / "README.md").exists())

    def test_preserves_safe_relative_symlinks(self):
        sync.replace_upstream(archive({"target.txt": "content"}, {"nested/link.txt": "../target.txt"}))
        self.assertTrue((self.root / "nested/link.txt").is_symlink())
        self.assertEqual((self.root / "nested/link.txt").read_text(), "content")

    def test_replaces_directory_with_file_and_file_with_directory(self):
        self.write("directory/old.txt", "old")
        self.write("file", "old")
        subprocess.run(["git", "-C", str(self.root), "add", "."], check=True)
        sync.replace_upstream(archive({"directory": "now file", "file/new.txt": "now directory"}))
        self.assertEqual((self.root / "directory").read_text(), "now file")
        self.assertEqual((self.root / "file/new.txt").read_text(), "now directory")

    def test_replaces_directory_with_symlink(self):
        self.write("directory/old.txt", "old")
        subprocess.run(["git", "-C", str(self.root), "add", "."], check=True)
        sync.replace_upstream(archive({"target.txt": "new"}, {"directory": "target.txt"}))
        self.assertTrue((self.root / "directory").is_symlink())
        self.assertEqual((self.root / "directory").read_text(), "new")

    def test_rejects_symlinks_into_exporter_before_modifying_checkout(self):
        with self.assertRaises(RuntimeError):
            sync.replace_upstream(archive({}, {"link": "_exports/exporter"}))
        self.assertTrue((self.root / "old.txt").exists())

    def test_rejects_reserved_paths_before_modifying_checkout(self):
        with self.assertRaises(RuntimeError):
            sync.replace_upstream(archive({"_exports/collision.txt": "bad"}))
        self.assertEqual((self.root / "old.txt").read_text(), "old upstream")

    def test_rejects_archive_path_escape_before_modifying_checkout(self):
        with self.assertRaises(tarfile.FilterError):
            sync.replace_upstream(archive({"../escape.txt": "bad"}))
        self.assertTrue((self.root / "old.txt").exists())

    def test_network_failure_propagates(self):
        failure = subprocess.CalledProcessError(1, "git fetch")
        with patch.object(sync, "run", side_effect=failure), self.assertRaises(subprocess.CalledProcessError):
            sync.refresh()
        self.assertFalse((self.root / "_exports/upstream.json").exists())

    def test_freshness_accepts_recent_and_rejects_stale_or_future(self):
        now = datetime.datetime.now(datetime.timezone.utc)
        for hours in (1, 49, -1):
            self.write("_exports/upstream.json", json.dumps({"revision": "abc", "checked_at": (now - datetime.timedelta(hours=hours)).isoformat()}))
            if hours == 1:
                sync.freshness(48)
            else:
                with self.assertRaises(RuntimeError):
                    sync.freshness(48)


if __name__ == "__main__":
    unittest.main()
